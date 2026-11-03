---
id: S-0209
title: Cell occupancy and crowding (capacity, entry outcomes, waiting, re-routing, scatter requests)
topic: movement
revision: 55
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-04
measurements: []
requests: [Q-0019, Q-0020, Q-0240, Q-0310, Q-0311, Q-0356, Q-0560, Q-2396]
supersedes: []
---

# S-0209: Cell occupancy and crowding (capacity, entry outcomes, waiting, re-routing, scatter requests)

## Summary

What a cell records about the objects in it, how many infantry fit, which *entry outcome* a moving
unit (infantry, ground vehicle or aircraft) gets for a neighbouring cell, what a walking unit does
with each outcome, when a cell's units obey a request to make room, and in which order a cell lists
its objects. Path costs built on these outcomes belong to S-0212, standing positions within a cell
and scatter destinations to S-0208, and crushing to S-0210. Skirmish only; network games and
campaigns are out of scope. References to their original rules are context, not implementation
requirements.

## Terms

Cited specs: S-0003, S-0004, S-0006, S-0009 (map file semantics), S-0105, S-0107, S-0203 (tick order), S-0204 (coordinates and
facing), S-0205 (number domains), S-0207 (ground vehicle movement), S-0208 (infantry movement),
S-0210 (crushing), S-0211 (land types and passability), S-0212 (pathfinding), S-0213 (human
orders), S-0214 (order targeting rules) and S-0215 (behaviours) are approved specs of the index in
`spec/README.md`. Claim, formula, table and row labels refer to the current coordinated texts,
including the latest drafts in this correction batch. Hand off their coordinated texts and this one
together for the teleporting-vehicle, aircraft and infantry reservation lifecycles,
shared quirks and vehicle entry-outcome summary. S-0212 claim 22's immediate fresh search and
same-tick follow-up remain coordinated with S-0207 claim 15 and scenario D17(d3). S-0208 and S-0212 use their current correction-batch texts where revised.

The glossary cited is the current approved glossary. Its terms are used as defined
there, in particular *cell*, *lepton*, *tick*, *infantry*, *vehicle*, *building*, *aircraft*,
*overlay*, *house*, *human-controlled*, *bridge deck*, *scatter*, *disguise*, *height level*,
*locomotor*, *recorder*, *computer-controlled* and *movement zone*. Three of the terms this spec
relies on first appeared in glossary revision 6: *AI house* and *house skill level*, which record the
project lead's decision that AI houses follow the unit rules of human-controlled houses (both are
repeated in the list below), and the *measurement setup* with its five allowances, among them
named test files and scripted input. Earlier revisions of the glossary
lack them. Terms from S-0208:
*standing position* (centre, top, right, left, bottom, numbered 0 to 4 in that order in its table
P), *position class*, *side positions*, *walking target*, *scatter request* (with its *threat point*
and its *forced* and *move-anywhere* options), *enterable*, *tether link*, *target*,
*destination* and *locomotor destination*. From S-0107: *has movement in progress* and *fraidycat*. The veterancy ranks are those of the
glossary (*veterancy rank*). From
S-0207: *track*, *track target*, *two-cell curve*, a curve's *chain point* and *entry point* (its
claim 8), and a vehicle type's *passive flag*. From S-0204: an object's *position* (its claims 1 and 3; a building's position is the centre of
the top-left cell of its foundation) and a building's *reference point* (its claim 4). From
S-0213 and S-0215: *behaviour*, the behaviour names (*area guard*, *enter* and so on), *guard
spot* and *contact* (the one pairing a mobile unit can hold with another object while the two carry
out an exchange; "in contact with nothing" means it holds none). From S-0214: *being erased*.
From S-0211: *usable area*, *land type*, the *passability category*, the *occupation pattern* of a
terrain-object type, the *laser-fence flag* and *firestorm-wall flag* of a building type, and a
house's *firestorm*, the passability categories (*wall-like* among them) and the movement zones
(*infantry* and *infantry-destroyer* among them) of its claims 11 and 12. From S-0212: *train* (a type marked as a train), *search layer* (its Terms: the ground or the deck of a bridge cell, as a place for the path search), and from its claim 27 what *counts as moving* for an occupant (it has a destination, is turning, or its locomotor reports it moving; a building never counts as moving) and its *close case*. From S-0210 (claim 1): *crusher* and *crushable* by a crusher. From S-0004: an overlay type's *registration index*. Terms used by this spec:

- **AI house, house skill level.** These two terms are those of the glossary (revision 10; first given in revision 6), repeated here
  in the form this spec uses them, so that the spec can be read with an earlier glossary
  revision. Where the two texts differ, the glossary entries are meant, with one exception: for
  the skill level of the neutral and special houses, claim 11 is meant, and the glossary's "every
  computer-controlled house" gets the maximum reads as "the house of every computer player".
  - An *AI house* is a house of our engine that our own AI plays, in place of the original's
    computer player. By decision of the project lead (recorded in the glossary entry *AI house*),
    an AI house has the same unit rules and the same order legality as a human-controlled house.
    A rule that the original applies only to units of a computer-controlled owner does not apply
    to its units, and our AI may reproduce the original's computer behaviour by giving ordinary
    orders. This is a rule of our engine, not a claim about the original, and it holds in every
    spec. It is not a quirk switch and not a departure, and this spec defines no switch for it.
  - The ruling covers player houses only. The *neutral house* and the *special house* (glossary
    *house*), which every skirmish creates after the players, are not AI houses. They keep the
    original's rules for owners that are not human-controlled (the project lead's decision of
    2026-10-10; glossary *AI house*, *Player houses only*). In this spec that means row WO1 of claim
    4 (a crate blocks their walkers) and their house skill level in claim 11.
  - The *house skill level* is an integer per house that gates some automatic behaviours of its
    units by comparison with game-wide thresholds from the rules data (claim 11). In a skirmish or
    network game of the original, the house of every computer player gets the rules' maximum skill
    level when the houses are set up. Every human-controlled house, and the neutral and special
    houses, keep the level 0 that every house starts with (claim 11). In our engine an AI house in
    a skirmish or network game has the level 0 of a human-controlled house, by the rule above, and
    the neutral and special houses keep the original's level 0.
  - In this spec the rule changes the outcome for our engine in row WO1 of claim 4 (and row VO1 of
    claim 5, campaign games only) and in the last condition of claim 11. Those claims state the
    original's behaviour, which scenarios C6 and C7 measure; the AI house's behaviour needs no
    measurement.
- **Cell layer.** A cell has a ground layer and, when it carries a high bridge, a bridge-deck layer.
  These are its cell layers. Every object in a cell belongs to one of them: an object on a bridge
  deck to the deck layer, every other object to the ground layer. Claim 15 gives when an ordinary
  walker changes this membership; target reservation uses a separate surface choice. This classification does not
  mean every object is listed there: claim 13 excludes ordinary aircraft at positive listing
  height and gives jumpjets their separate flight-state listing rules. It is
  neither a rules layer nor a render layer.

  *The cell layer that counts* for a neighbouring cell that is asked about depends on a *reference
  height level*, which goes with the mover and the situation:
  - a walker, when it asks for the next cell of its path on its asking tick (claims 6 to 10): its
    own height level, that is, the height level of its cell, plus 4 while it is on a bridge deck,
  - a ground vehicle, in the check it makes as it begins a movement step (claim 5): its own height
    level, counted in the same way,
  - a walker or a ground vehicle in a path search: the height level of the pair being expanded,
    its cell's height level plus 4 on its deck search layer, with the start-pair adjustment for
    trains as S-0212 claim 7 gives it. The step starts from that pair's cell, not necessarily the
    cell the mover currently stands in. S-0212 claims 7 and 10 own the search and its height rule,
  - for the first landing-point entry check of teleporting infantry (S-0208 claim 19): initially
    −1 height levels, with no connecting-step direction. This is not an ordinary walking ask.
    The check has no connecting-step height restriction. Unless the negative-level exception
    below applies, a high-bridge destination uses its terrain height level plus 4 as the
    reference for its occupancy record; both objects and record then come from the deck.
    A destination without a high bridge uses ground objects and record. These choices are
    independent of the standing-position deck mode used to choose the landing point.
    S-0208 claim 19 owns this use, with `measurable` backing (its scenario W11) and medium
    confidence for the bridge consequences,
  - for the entry check of the *obstructed* class of a group destination look (S-0213 claim 7):
    the common cell's height level, plus 4 when the common cell is on a high-bridge deck, with
    no connecting-step direction, so no connecting-step height restriction applies. S-0213
    claim 7 owns this use, `unmeasured` and rated medium there. Which cell layer of a high-bridge
    asked cell counts for this check is not established (Open questions),
  - in other order targeting (S-0213, S-0214) and other situations not listed above: not
    established (Open questions).

  **Landing-only negative-level exception.** For that teleporting-infantry entry check, a
  destination at terrain height level −6 or lower gives *clear* without testing terrain or
  occupants: the initial reference −1 is more than 4 height levels above it. These are integer
  comparisons with no rounding. This does not bypass a refusal of the preceding standing-position
  choice. S-0208 claim 19 owns the exception, its scenario W11(g) and
  `quirks.teleport_infantry_negative_level_entry` (default original; off, omit this early bypass
  and apply the ordinary landing entry check). It does not change ordinary walking entry rules.
  This pointer inherits that claim's `measurable` backing and high confidence for the comparison.

  For the walking asks, vehicle step checks and path searches above, an asked cell that carries
  a high bridge uses these rules:
  - the objects listed are those of the deck layer when the reference height level differs from
    the asked cell's height level by more than 1, and those of the ground layer otherwise,
  - the occupancy record read is the deck layer's when the reference height level is exactly 4
    above the asked cell's height level, and the ground layer's otherwise.

  For every other cell both come from the ground layer. The two rules agree for a
  walker on a deck over ground of the asked cell's level, and for a walker stepping onto the deck
  from ground 4 height levels above the asked cell (both deck), and for a walker on the ground
  under the bridge (both ground). They can disagree on a deck that spans uneven ground: when the
  ground under the next deck cell is 1 height level above or below the ground under the walker's
  own deck cell, the objects come from the deck and the record from the ground (Open questions; a possible
  switch is among the Questions for the project lead). A step between two cells whose height levels differ by 2, 3 or more than 4 is
  *blocked* by the height rule of S-0212 (claim 10) before the cell layer matters.
  - Confidence: medium for the two rules and for the walker's height level on its asking tick,
    medium for the vehicle's at the start of a step, high for the path search's reference level
    and start cell, low for the uneven-ground case.
  - Backing: `measurable` (scenario C22, parts a and b, for a deck over level ground; part c uses
    the vehicle's case; S-0212 scenario P4, maps K and K2, for the path-search reference). `unmeasured` for the uneven-ground case: which ground a high bridge may span
    belongs to the bridge specs, and whether such a bridge can be built at all is not established,
    so no map is known that shows the case.
- **Occupancy record.** For each cell layer, it holds:
  - which of the five standing positions are taken,
  - a *vehicle-present* flag,
  - a *building-present* flag,
  - an *object-present* flag, set by objects of other kinds, for example an aircraft on the ground,
  - the *infantry house*: the house of the last infantry unit to mark a position there, or none
    (claim 1). It need not be the house of any infantry unit still there (Edge cases).

  A mover is *excused from the vehicle-present flag* of a cell layer when a row of its occupant
  table says so: an enslaved walker asking about its master's unloading cell while its master is
  listed there (row WP2 of claim 4), and a vehicle asking about the cell layer in which it is
  listed itself (row VP1 of claim 5). For a mover that is excused, the record tables of claims 4 and 5 read the flag as clear.
  These two rows carry the confidence and backing of this rule.

  The record holds state only; claim 1 says which objects set each part. The building-present flag
  plays no part in this spec's entry outcomes (claim 3). It counts only where another spec asks
  whether the record is empty as a whole, as the free-cell test does: S-0212 claim 25 owns that
  test (its nearby-cell search), and S-0208 claim 13 uses it for a scatter without a threat point.
- **Listed, listing order.** The objects of a cell layer form an ordered list (claim 13). "Listed
  first" means at the front of that list.
- **Allied.** A house is allied with itself and with the houses it counts as its allies in the
  game. In the original each house keeps its own set of allies, so the relation need not be
  symmetric. Each test below is made from one side, its *view*:
  - the mover's view (the walker's or the vehicle's house is allied with the other house): the
    occupant rows of claims 4 and 5, the wall rows of both claims (the wall's owner), the
    infantry-house rows WR2, WR3 and VR6 (the recorded house), and the attack test of claim 9.
    Unless a claim says otherwise, "allied" in this spec means this view,
  - the gate owner's view (the gate's owner is allied with the mover's house): the closed-gate rows
    of claims 4 and 5.

  An object without an owner, such as a terrain object, is allied with no house, in either view.
  A wall's owner is the house that S-0210 (claim 14) gives it; a wall without an owner, which
  includes every map-placed wall with no qualifying building, counts as owned by another house
  in the wall rows.
  So a terrain object always falls under the enemy rows of claims 4 and 5, and the attack test of
  claim 9 finds it not allied.
  - Confidence: high for each of these views, and for objects without an owner.
  - Backing: the views differ only under a one-sided alliance. The teams chosen when the game is
    set up, and the map-trigger actions that make or end an alliance, act on both houses at once.
    The alliance command a player gives during play makes only the player's house an ally of the
    other house (medium confidence), so it can set up a one-sided alliance where the game settings
    allow alliances to change during play. `measurable` (scenario C25) for the walker's occupant,
    infantry-house, gate and wall rows, the attack test of claim 9 (part a), and the vehicle's
    occupant, gate and wall rows. `measurable` (scenario C26, part e) for the vehicle's
    infantry-house row (row VR6), which decides only when no occupant gives a value: C26 (e) asks
    while a walker of the other house has chosen its walking target in the cell and is not yet
    listed there (claim 4, last consequence). For objects without an owner, `measurable` (scenario
    C16, part c2, for the attack test of claim 9; scenario C20, parts k1 and k2, for a vehicle). The
    other scenarios use the same house, or a pair of houses that are allied, or hostile, both ways.
- **Blocking cell of a building.** A foundation cell of a building is one of its *blocking cells*
  when the building's type has no impassable-rows value (Import mapping), when the type is a bunker
  and a unit sits in it, or when the cell lies fewer than *k* cells along x from the building's
  top-left cell, *k* being the type's impassable-rows value. A cell outside the foundation is not a
  blocking cell. So a value of 0 makes no cell a blocking cell (unless the bunker case applies), and
  a value of at least the foundation's width along x makes every foundation cell one. Used only by
  claim 5.
  - Confidence: medium that a type without the value, and an occupied bunker, make every foundation
    cell a blocking cell; low that the count runs along x from the top-left cell.
  - Backing: `measurable` (scenario C23, parts c and d, through the repair and bunker rows of
    claim 5).
- **Armed.** This spec's test of *armed* is the one below: for a walker the
  truncated mean damage of its weapons, not their plain sum, and for a vehicle a primary weapon or
  the train flag. A walker is *armed* when its *mean damage* is positive. Let *W* be the set of weapons
  the walker has in its primary and secondary slots at its current veterancy rank (an elite unit has
  the elite weapon of a slot when its type gives one, otherwise the ordinary one; glossary *veterancy
rank* and *elite weapon*), *n*
  the number of weapons in *W* (0, 1 or 2), and *d*(*w*) and *a*(*w*) the damage field and the
  ambient-damage field of a weapon *w*:

  > mean damage = trunc(Σ<sub>*w* ∈ *W*</sub> (*d*(*w*) + *a*(*w*)) / *n*) for *n* ≥ 1, and 0 for
  > *n* = 0. Integers, exact integer arithmetic, and trunc rounds toward zero. Unit: that of the
  > damage field.

  So a walker with only a secondary weapon can be armed, and a primary weapon of damage 1 beside a
  secondary weapon of damage 0 (no ambient damage) gives trunc(1 / 2) = 0: unarmed. The value does
  not depend on the occupant in question. For a walker of a *multi-turret type* (S-0105 Terms,
  applied to the walker's type), the mean damage is that of its currently selected weapon alone, and
  which weapon that is belongs to the combat specs. Some rows of claim 4 ask something else, and say
  so: the wall rows ask whether the walker *has a primary weapon* and whether that primary weapon
  *can damage walls*, and the closed-gate rows whether it has a primary weapon.
  - Confidence: high that the test asks for a positive mean damage over both weapon slots, and that
    a walker without a primary weapon but with a damaging secondary one is armed. Medium for the
    truncated mean, the ambient-damage term and the elite weapons. Low for the multi-turret case.
  - Backing: `measurable` (scenario C3, parts a2 to a6). `unmeasured` for the multi-turret case:
    which weapon such a walker has selected belongs to the combat specs, and no setup is known that
    fixes it for a walker.

  A ground vehicle is *armed*, in the rows of claim 5, when it has a primary weapon (below) or its
  type is a train (S-0212 Terms). So a train without any weapon is armed. The rows of claim 5 that
  say *has a primary weapon* ask for the primary weapon alone, without the train case.
  - Confidence: high that a vehicle's test asks for a primary weapon and does not look at the
    damage. Medium that a train counts as armed without one.
  - Backing: `measurable` (scenario C20, parts e1 and e2, for the primary weapon; part e4 for the
    damage playing no part; part e3 for a train without a weapon).

  A walker or a vehicle *has a primary weapon* when it has a weapon in its primary slot at its
  current veterancy rank: an elite unit has the elite primary weapon when its type gives one, and
  otherwise the ordinary one, as for the walker's mean damage above. So an elite unit whose type
  gives only an elite primary weapon has a primary weapon, and the same unit below elite rank has
  none. For a multi-turret type the weapon of its currently selected turret is asked instead, and
  which one that is belongs to the combat specs. Rows WO2 to WO4 of claim 4 and rows VO4 to VO6 and
  VR10 of claim 5, which ask whether the primary weapon *can damage walls*, *can damage wood* or
  *can hit ground targets*, ask that of this weapon (Import mapping). Row VP29 of claim 5 asks
  instead about the weapon the vehicle would use against the terrain object.
  - Confidence: medium for the elite rule. Low for the multi-turret case.
  - Backing: `measurable` (scenario C20, part e5, for a vehicle; scenario C8, part c2, for a
    walker). `unmeasured` for the multi-turret case: which turret weapon a walker or a vehicle has
    selected at a given moment belongs to the combat specs, which do not yet say how to fix it in a
    setup, so no scenario can tell whose weapon the test asked about.
- **Cloaked.** An object whose cloak state is fully cloaked. Whether a house can see it anyway, for
  example because a sensor of that house covers its cell, plays no part here: the cloaked rows of
  claims 4 and 5 apply to an object in that state even when the mover's house can see it.
  - Confidence: high that the rows read the cloak state alone. Medium that a sensor leaves the cloak
    state as it is (the visibility specs own sensors).
  - Backing: `measurable` (scenario C20, part g3, for the vehicle's row; scenario C3, part f2, for
    the walker's).
- **Entry outcome.** The answer a moving unit gets for a neighbouring cell. It is one of eight
  values, in order of increasing severity:

  | Value | Name (ours) | Meaning |
  |---|---|---|
  | 0 | clear | the unit may enter |
  | 1 | cloaked blocker | a cloaked enemy stands there |
  | 2 | moving blocker | friendly units that are on the move occupy it, or the vehicle-present flag is set (a vehicle stands there or has started a step into it) |
  | 3 | closed friendly gate | an allied gate that is closed |
  | 4 | friendly obstacle | an allied wall that could be destroyed |
  | 5 | enemy obstacle | an enemy object the unit could destroy |
  | 6 | stationary friendly | friendly units that stand still occupy it |
  | 7 | blocked | the unit may not enter |

  The numbers are ours, for ordering only. Claim 4 says how the contents of a cell combine into one
  value.
- **Mission target.** The object or cell that a unit's current behaviour aims to enter, capture,
  sabotage or otherwise reach. Here it is the unit's destination or target, while the unit is in a
  behaviour that has a mission target. The behaviour specs own which behaviours have one (Open
  questions).
- **Gate.** A building type that units can pass through while it is open. When a gate is open or
  closed, how it starts (closed) and how it opens and closes are S-0215's (claim 35); the
  *closed friendly gate* outcome of this spec is what makes it open.
- **Close-enough distance.** A game-wide imported distance in leptons (Import mapping). A unit
  within it of its destination counts as having arrived when the last cells are occupied.
- **Blockage delay.** A game-wide imported number of ticks (Import mapping). A unit waits this long
  behind moving blockers before it searches a path that avoids them.
- **Path retry timer.** A per-unit timer that delays the next path search after a failed one. Its
  length belongs to S-0212.
- **Elite.** A unit whose veterancy rank is the highest (glossary *veterancy rank*).
- **Scatter veterancy ability.** An ability that a unit type can be given for a veterancy rank in the
  imported data (Import mapping). A unit has it when the data gives it to the unit's type at the
  unit's current veterancy rank; the spec that glossary *veterancy ability* names owns the
  abilities and which rank grants each.
  This spec uses only this one (claim 11).
- **Invulnerable.** Protected by an effect that makes an object take no damage for a time: the
  glossary's *temporarily invulnerable*. How the effects start and end belongs to
  the superweapon specs.
- **Parasite, victim.** A unit whose weapon attaches it to another unit, which it then damages from
  inside, is a *parasite*; the unit it is attached to is its *victim* or *host* (S-0614 claims 1 to 5).
  Either an infantry mover or a vehicle mover can be the parasite in the entry tables below.
  The overrides in claim 4 row WP1 and claim 5 row VP2 apply to entering a cell while moving,
  not to the parasite's exit test. S-0614 claim 13 owns that separate test: the host blocks its own
  cell, so a non-naval parasite must search for another exit cell. This pointer carries that
  claim's medium confidence and `measurable` backing (its scenarios P4 and P9).
  Ordinary return placement uses the ground occupants and ground occupancy record, without
  an approach direction or source elevation. Only *clear* accepts placement under the entry
  tables of claims 4 and 5; every other outcome refuses immediately, without waiting,
  re-routing, scatter requests, gate opening, uncloaking or attacking an obstruction.
  S-0614 claim 20 owns this single placement attempt, its silent removal on failure and the
  additional infantry standing-position choice. By contrast, S-0614 claim 23's host-loss
  fallback chooses a standing position using ground occupancy even when the accepted point
  will finally be placed on a bridge deck. Its final forced placement bypasses entry legality.
  It is not an ordinary movement ask. That claim owns the fallback's failures and bridge
  surface rule. These placement pointers have `measurable` backing (S-0614 scenario P10),
  high confidence for clear-only acceptance and no movement reactions, and medium confidence
  for uncommon entry-table exceptions and fallback bridge consequences.
- **Enslaved unit, master, master's unloading cell.** An *enslaved unit* is a unit that works for
  another object, its *master* (for example the slaves of a slave-miner harvester); the economy specs
  own them. The *master's unloading cell* is the cell the master stands in, or, for a master that is
  a building with a foundation of *W* × *H* cells, the foundation cell *W* − 1 cells along x and
  ⌊*H* / 2⌋ cells along y from its top-left cell (glossary *foundation*; integer division). That
  this is counted from the top-left cell is medium confidence; scenario C13 (b) records the cell.
  Returning slaves are sent to that cell with a plain *move* (the economy specs own when).
- **Guarding a vehicle.** A walker whose behaviour is *area guard* and whose guard spot is a vehicle
  guards that vehicle. An order to area-guard an object makes that object both the guard spot and
  the destination (S-0213).
- **Between steps.** A vehicle is *between steps* from the moment it completes a movement step (it
  reaches the target of its track, S-0207) until it next advances along a track. A vehicle that has
  not advanced along a track since it was created, or since it was put down somewhere else (for
  example by a teleport), is between steps too. This definition applies to vehicles that drive on
  tracks. For a hovering vehicle, S-0503 claim 13 owns the state: it starts between steps, each
  arrival at its heading point puts it between steps again, and the next positive-distance travel
  with a heading point ends that state, including travel on the same arrival tick. Choosing a
  heading point does not itself end the state; losing one after refusal does not itself set it.
  This pointer inherits that claim's confidence and `measurable` backing (S-0503 scenario H11).
  That walking vehicles are between steps at the end of each step, and otherwise not, is medium
  confidence. Vehicles that move in other ways (for example by jumping or flying) are not known
  ever to leave the state (low). Otherwise this definition has the confidence and backing of the
  rows that use it: rows WP17 and WP18 of claim 4 and rows VP22 and VP23 of claim 5. U6 (claim 4's
  list) still applies to isolating the split of those rows, not to the hover state itself.
- **At a chain point.** A vehicle that drives on tracks is *at a chain point* while its position is
  the chain point of its current two-cell curve, the step after that curve turns again, and the next
  pair of steps has a curve with an entry point (S-0207 claim 8). Whether the vehicle's type has the
  passive flag (S-0207) plays no part: a vehicle without it still counts while it passes that point,
  although under S-0207's default it never chains the curve. A vehicle that does not drive on tracks
  (for example a hovering vehicle) is never at a chain point. This definition has the confidence
  and backing of the rows that use it: rows WP17 and WP18 of claim 4 and rows VP22 and VP23 of
  claim 5.
- **Side-position pattern.** The occupation pattern of a terrain-object type for the map's theater
  (the snow pattern in the snow theater, the temperate pattern in every other, as S-0211 picks it).
  For a pattern value *p*, any integer that the importer accepts (S-0003), negative values included:

  > right position marked ⇔ ⌊*p* / 1⌋ mod 2 = 1, left position marked ⇔ ⌊*p* / 2⌋ mod 2 = 1,
  > bottom position marked ⇔ ⌊*p* / 4⌋ mod 2 = 1. Exact integer arithmetic. ⌊ ⌋ rounds toward minus
  > infinity and mod gives 0 or 1. So only *p* mod 8 (taken from 0 to 7) has an effect.

  So the pattern 7 marks all three side positions, the pattern 4 the bottom one only, the pattern −1
  all three (−1 mod 8 = 7) and the pattern −5 the right and left ones (−5 mod 8 = 3). Confidence high,
  backing `measurable` (scenario C16, parts a and a′).
- **Asking tick.** A walker that stands on a standing position with a path step ahead asks, on one
  tick, for the entry outcome of the next cell of its path. That tick is its *asking tick*. When the
  next cell is *clear* and a position is free, it is the tick on which the walker chooses its walking
  target there (S-0208 claims 3 and 6). Which tick that is, counted from the walker's arrival on its
  standing position, belongs to S-0208. This spec states its reactions relative to the asking tick.
  A walker that does not choose a walking target on its asking tick, because it waits or reacts to
  an obstacle, asks again on every later tick on which it still stands there with a path step
  ahead, and each such tick is an asking tick too (claim 7, later ticks).
  The walker does not move on the tick on which it chooses a walking target, and takes its first
  step towards it on the next tick (S-0208 claim 6). So at a free cell boundary the asking tick is
  the last tick before the walker's position changes again.

  *First ask, repeated ask.* On one asking tick the entry outcomes of up to two cells decide what
  the walker does: the next cell of the path it holds when the tick begins (its *first ask*), and,
  when the first ask gives *cloaked blocker*, *stationary friendly*, *friendly obstacle*, *enemy
  obstacle* or *blocked*, the first cell of the new path it searches on that same tick (its
  *repeated ask*, claim 7). The two names say which cell's outcome decides which reaction. They fix
  no internal order: what is observable is the walker's state at the end of the asking tick (its
  position, path, target and walking target, and the requests it has sent), as claims 7 to 9 and
  State and ordering give it (scenario C19).

## Behaviour

### The occupancy record and capacity

1. **What an infantry unit and a vehicle record.**
   - *Infantry.* An infantry unit ordinarily marks one standing position in the occupancy record of a cell
     layer, and sets that cell's infantry house to its own house. Own-locomotor teleporting
     movement has the separate lifecycle below. When several units mark, the most
     recent one's house wins. The position marked is the position class (S-0208 claim 2) of one
     point: its walking target while it has one, otherwise its own position.
   - *Reservation.* When a walker chooses its walking target in the next cell (S-0208 claim 3), its
     old mark is cleared and the new target is marked at once, with the walker's house as the next
     cell's infantry house. The position is reserved before the walker gets there, and its old
     position is free from that tick. Claim 15 gives the reserved surface, independently of the
     entry check's record and the walker's listing layer. The walker is listed in the next cell only
     from the tick on which its position moves into it (claim 13), so for a few ticks the next cell
     shows the taken position and the walker's house while the walker is still listed in the cell
     it is leaving.
   - *Teleporting infantry.* S-0208 claim 19 owns the chosen-point reservation and
     its post-jump lifecycle (Q-0311). The chosen point's height determines whether
     its standing position and infantry house are marked in ground or deck. Landing
     itself neither transfers that reservation nor adds a mark at the actual position.
     A ground-height choice followed by a high-bridge landing therefore retains its
     ground mark and house, lists the infantry only on deck, and adds no deck
     standing-position mark. A choice made at deck height keeps its earlier deck
     reservation instead.
     After a completed jump, a later destination choice clears by actual position
     and height, not by the former chosen ground point. On deck this clears the
     corresponding deck position, even if another infantry marked it, while leaving
     ground taken. Success marks the next landing point; refusal marks the current
     deck position. After a successful next jump, the old ground mark can remain
     without infantry listed on either surface. Ground position choice still counts
     it as taken. This differs from the vehicle and aircraft placement marks below
     and shares `quirks.teleport_infantry_ground_reservation` (Edge cases).
     This pointer has S-0208 claim 19's medium confidence and `measurable` backing
     (its scenario W11, part i).
   - *Clearing the house after an infantry mark.* When an infantry unit clears its mark, the
     infantry house goes back to none only when none of the three side positions is taken
     afterwards, by an infantry unit or by a terrain object's mark. Otherwise it stays as it was.
     This house-clearing rule applies only to clearing an infantry mark; clearing a terrain
     object's marks leaves the house unchanged, even when it frees the last side position.
   - *Driving vehicles.* A driving vehicle sets the vehicle-present flag of the cell layer its
     position is in. Driving movement has the reservations below; hovering and teleporting
     movement have their separate lifecycles described next. When it starts a movement step into a neighbouring cell, it also sets that cell's flag at once, before
     its position is in it. When the step is a two-cell curve (S-0207 claim 7), it sets the flag of
     both cells of the curve at once. A vehicle that drives on tracks clears a flag at three
     moments, each time in the cell layer of its position at that moment:
     - when it passes the first point of a track (S-0207 claim 3) while it is between steps (Terms),
       it clears the flag of the cell it stands in, which is the cell it is leaving. Its position is
       then still in that cell, and stays there for about half a step,
     - on a two-cell curve, when it reaches the curve's release point (S-0207 table T1), it clears
       the flag of the curve's first cell, while its position is still in that cell,
     - when it is taken out of a cell's list while it is between steps (for example when it leaves
       the map), it clears the flag of that cell.

     So a vehicle at rest keeps the flag of its cell, the target cell of its last track keeps the
     flag until the vehicle makes progress on its next track, and a cell it is leaving loses the
     flag before the vehicle's position leaves it. While the vehicle is still listed there, the
     occupant rows of claims 4 and 5 (row VP1 for the vehicle itself) still see it. Moving into a
     new cell during a track clears nothing. When ships and walking vehicles clear their flags
     is not established (Open questions).
   - *Seized vehicles with a ground stand-in.* S-0607 claim 23 owns the exception
     to ordinary resting occupancy for a vehicle seized with a driving, walking or ship
     stand-in locomotor. With `quirks.lift_standin_unmarked_cell` on (default original),
     the seize clears its vehicle-present flag and installing the stand-in marks no cell.
     It holds no vehicle-present flag until that stand-in's movement sets one; a vehicle
     that never moves remains unmarked even after its grip ends. It stays listed, so the
     missing flag does not by itself bypass the occupant rows of claim 5. With the switch
     off, the vehicle keeps its own cell's flag at the seize, and subsequent movement
     clears and marks normally. The teleporting stand-in instead reserves its landing
     point as S-0607 claim 23 and S-0207 claim 20 specify. This pointer inherits medium
     confidence for the marks, low confidence for the composed movement consequences,
     and `measurable` backing (S-0607 scenario L13, part f).
   - *Hovering vehicles.* S-0503 claims 13 and 15 own their reservations, between-steps state
     and arrival outcomes. A clear heading-point choice reserves that point's cell before
     physical entry. Replacing or abandoning the point clears its old reservation, except for
     the no-choice idle arrival below. The positive advance with a heading point that ends the
     between-steps state clears the flag at the actual position before movement, even when the
     vehicle remains in that cell. While not between steps, cell-edge crossings move the listing
     without setting or clearing flags, so cells merely swung through at a turn are not reserved.
     A crossing while still between steps clears the old actual-position flag and marks the new
     actual position. These are flags, not per-vehicle counts.

     Setting a flag uses deck occupancy only when the marked point's cell has a high bridge and
     the point is at least 416 leptons above terrain surface there; otherwise it uses ground.
     Clearing uses the point's height alone: at least 416 leptons above that surface clears deck,
     even without a high bridge, and below that clears ground. These are integer comparisons,
     with no rounding. Neither operation uses the listing layer. S-0503 claim 13 owns
     `quirks.hover_flag_clear_height_only` (default original); off, clearing requires the same
     high-bridge condition as setting.

     For a surviving on-map vehicle, an arrival-time choice that does not refuse entry and
     leaves no heading point marks the actual position and produces S-0503 claim 15's
     cell-arrival update. This includes a timer-deferred search with the locomotor destination
     retained, a failed search during choice even when it clears that destination, and an absent
     locomotor destination with contact in the vehicle's present cell, provided the vehicle is
     not held. Search success is not required. A refused choice, including a held vehicle,
     supplies neither restoration nor that update. If the initial arrival path update has already
     cleared the locomotor destination, choice requires contact in the present cell. Without
     that contact, a failure there that cleared the old heading reservation supplies no
     replacement mark or update. An ordinary no-destination/no-contact arrival with no stored
     step clears the heading point without releasing its old reservation, absent another clearing
     event. S-0503 claim 15 owns `quirks.hover_idle_arrival_reservation` (default original).
     Off, release that reservation and mark the actual position, still without a cell-arrival
     update. These tests use the locomotor destination, not the order-level destination.
     Any already-computed final travel can still follow these arrival outcomes; a boundary
     crossing while between steps transfers an actual-position mark to the post-travel cell.
     Crossing itself is not a cell-arrival update.
     These pointers inherit high confidence for reservation and first-advance release, medium
     confidence for exceptional crossings and restoration, and `measurable` backing
     (S-0503 scenarios H11 and H15).
   - *Teleporting vehicles.* S-0207 claim 20 a, e and k owns their reservation lifecycle.
     Fixing a landing point reserves its ground record, even for a high-bridge cell and even
     when the nearby search uses deck mode. Replacing a remembered landing point clears that
     point's ground reservation, including after a completed jump. Before any landing point
     has been remembered, clearing uses the vehicle's actual position and height.
     A jump onto a high bridge lists the vehicle on the deck and marks the deck record,
     without clearing or transferring the ground reservation: immediately after an uninterrupted
     landing both records have the vehicle-present flag. A later accepted destination clears
     the remembered ground reservation and reserves the new landing cell's ground record.
     The old deck mark stays until the next jump removes the vehicle from that cell.
     The ground reservation is not a listed ground occupant, and reservation layer need not
     match listing layer. This uses the shared `quirks.teleport_vehicle_ground_reservation`
     (Edge cases). Removal clears the actual cell's flag at actual height, not a separate
     remembered or pending ground reservation. After deck landing without replacement,
     removal therefore leaves ground marked and deck clear. With an accepted replacement
     not yet jumped to, old ground is already clear, removal clears old deck, and the new
     ground reservation survives. Off a high bridge, removal clears the ground flag when
     actual occupancy and reservation coincide. A distinct pending reservation also survives
     removal before the first jump. These results assume no other flag writer intervenes.
     These are pointers with S-0207 claim 20's medium confidence and `measurable`
     backing (its scenario D22, runs 13 and 15), not additional driving release rules.
   - *Temporary driving after a teleport.* S-0207 claim 21 owns this conditional case.
     Installing temporary driving alone clears neither mark of a previous two-record deck
     landing. Ordinary driving departure clears actual deck occupancy but leaves the separate
     remembered ground reservation. Driving marks its current cell at actual height.
     Restoration relocates no mark and does not recreate the former deck mark. This assumes
     no earlier replacement destination or other flag writer changed those records.
     Confidence: medium. Backing: `unmeasured` for this previous-deck state, because no
     established stock factory exit starts in it; S-0207 scenario D23 gives a conditional
     observation protocol, not permission to manufacture the state.
   - *Bridge-collapse drops.* S-0511 claim 9 owns the immediate occupancy changes when
     an object drops from a lost deck. A stationary infantry unit's standing-position mark
     or stationary vehicle's present flag clears on deck and is marked on ground at once,
     before S-0511 claim 11's check for units heading onto the lost deck. The transfer is
     not deferred until that check or landing. Movement reservations clear at the locomotor's
     heading point, when present, using that point's stored height, in addition to ordinary
     listing-change marks. Clearing uses the ordinary position and flag rules above, not
     ownership counts. No blanket occupancy-record reset follows the attempt, landing or
     repair; unrelated marks keep their ordinary clearing events. With no other mark writer,
     a stationary dropped unit therefore leaves no phantom deck mark after repair.
     This pointer inherits S-0511 claim 9's medium confidence and `measurable` backing
     (its scenarios B3, turn-order and marks variant, and B6).
   - *Jumpjets.* S-0502 claim 4 owns actual-position marking during jumpjet flight,
     separately from claim 13's listing, S-0502 claim 15's landing reservation and
     S-0502 claim 21's hover slot. Grounded, ascending, descending, crashing and
     crashed physics can supply ordinary kind-specific marks when the resulting
     placement qualifies for claim 13's exact jumpjet listing predicate, including its
     separate surface-height and adjusted-zero tests over high bridges. A vehicle can therefore mark
     during low positive-height ascent or descent and near crash end. The first rise
     clears the preceding occupancy and ends its on-deck state, but does not forbid
     later low-flight marking. Descent can supply both an actual-position mark and a
     landing reservation. Hovering and cruising physics create neither a new listing
     nor a new actual-position mark; do not infer a blanket erasure of retained
     low-flight placement from this absence of new marks. These are pointers with
     S-0502 claim 4's high confidence for placement eligibility, medium confidence for
     kind-specific marking consequences and `measurable` backing (S-0502 scenario J3).
   - *Buildings.* A building sets the building-present flag of its foundation cells.
   - *Terrain objects.* A terrain object marks the side positions that its type's side-position
     pattern (Terms) gives, in the ground layer of its cell. It never sets the infantry house, and it
     sets none of the three flags. Its marks count wherever this spec counts taken positions: for a
     full cell (claim 2), for the record table of claim 4 (part 4d), and for the clearing rule above. When the
     terrain object is removed, the same positions are cleared, and the infantry house is left as
     it is.
   - *Other objects.* An ordinary landed aircraft sets the object-present flag. S-0501
     claim 44 owns its ordinary marking and departure: rest on terrain marks ground,
     while rest on a high-bridge deck marks only deck, not both records. At each marking
     or clearing event, deck is used when a high bridge is present and actual world
     height is at least terrain surface height plus 416 leptons; otherwise ground is
     used. These are signed integer comparisons in leptons, with no rounding. Neither
     listing nor the aircraft's own deck mark substitutes for this surface test.
     Normal deck take-off clears the deck flag in the first climbing movement tick,
     without transferring it to ground when the aircraft's own deck mark clears.
     Accepting a take-off destination alone does not release it. By the end of that
     climbing tick the aircraft is absent from both occupant lists (claim 13).
     These results assume initially clear records and no other flag writer; the flags
     are not counts of objects. This ordinary-flight pointer has high confidence for
     marking and clearing, medium confidence for the combined take-off consequence,
     and `measurable` backing (S-0501 scenario AC27, occupancy variant). It does not
     govern the separate teleport reservations below.
     A rocket missile using rocket-flight movement sets no object-present flag, even during
     pause, tilt or raise at its launch point (S-0506 claim 23). This exception has high
     confidence and `measurable` backing (S-0506 scenario R17). Which other kinds set the
     flag is not known.
   - *Teleporting aircraft.* S-0207 claim 20 a and e owns this lifecycle. Fixing a
     landing point reserves its ground object-present flag, even on a high bridge.
     Deck placement also marks the deck object-present flag without clearing ground:
     immediately after an uninterrupted landing both records are marked, but only the
     deck lists the aircraft. The next accepted destination clears the remembered ground
     reservation; the next jump's departure clears the actual deck mark. Destruction and
     removal while the aircraft remains on deck clear only actual deck occupancy, not a
     separate remembered or pending reservation (S-0207 claim 20 k). Without replacement,
     ground stays marked. With a replacement accepted but not jumped to, old ground is
     already clear and the new ground reservation survives removal. Before the first jump,
     a distinct pending ground reservation likewise survives actual-cell clearing. Off a
     high bridge, removal clears the coincident actual/reservation ground flag.
     These flags are not counts of objects: another object's marking or clearing of the
     same flag can overwrite that result. The ground mark is neither a listed aircraft
     nor a vehicle-present flag. This shares `quirks.teleport_aircraft_ground_reservation`
     (Edge cases), with medium confidence and `measurable` backing through S-0207
     scenario D22, runs 14 and 15, and scenario C30 here.

   Consequence of the clearing rule: in a cell whose terrain object marks a side position, the
   infantry house never goes back to none while the terrain object stands, once an infantry unit has
   marked there. The house of the last infantry unit to mark stays in force after that unit has left
   (Edge cases). Removing the terrain object does not clear it either; only the next clearing of an
   infantry mark there, with no side position taken afterwards, does.
   - Backing: `measurable` for the infantry part (scenarios C1 and C12, for the clearing rule and
     for "the most recent one's house wins"; scenario C26, parts b to f, for a reserved position and
     its house before the walker is listed), for the vehicle flag (scenario C2, parts c and d; scenario C26, parts a1 and a2, for the flag set
     at the start of a step), for
     the building flag (scenario C10, part c, through the free-cell test of S-0212 claim 25), for the
     moments at which a driving vehicle clears its flag (scenario C28), and for
     the terrain marks (scenario C16), and for the object-present flag's isolated effect
     through a teleporting aircraft's retained ground reservation (scenario C30 and
     S-0207 scenario D22, run 14).
   - Confidence: high for the infantry part and the terrain marks. Medium for the vehicle, building
     and object-present flags, and for the three moments at which a driving vehicle clears its flag
     (inferred, not observed). Low for the clearing when a vehicle is taken out of a cell's list.

2. **Three side positions make a cell full for infantry.** A cell layer counts as full for an
   arriving infantry unit when its right, left and bottom positions are all taken. The centre and the
   top positions do not count.
   - Arriving walkers only ever take side positions (S-0208 claim 3), so a cell normally holds at most
     three walkers.
   - A unit standing at the centre or the top position does not reduce that room. Such a unit can be
     pre-placed by the map (S-0208 claim 5), or have walked in towards its mission target (S-0208
     claim 4).
   - So a cell can hold five infantry at distinct standing positions: three on the side positions,
     one at the centre and one at the top. This is not a separate five-object limit on map placement.
     A unit at the top position is recorded as taking the centre (S-0208 claim 2).
   - Side positions marked by a terrain object (claim 1) count as taken, so a cell holding one has
     room for fewer arriving walkers, and none when the object's pattern marks all three.
   - Backing: `measurable` (scenario C1, S-0208 scenario W2, and scenario C16 for the terrain marks)
   - Confidence: high

3. **Vehicles and infantry do not share a cell; buildings keep walkers out through claim 4.**
   - An infantry unit cannot take a standing position in a cell layer whose vehicle-present flag is
     set.
   - It cannot take one in a cell whose object-present flag is set either, unless an open gate stands
     in that cell (S-0208 claim 3).
   - Neither flag applies where the position is chosen without regard to the occupants (S-0208
     claim 4): in the cell that holds the walker's mission target, and, for an enslaved walker, in its
     master's unloading cell while the master is there.
   - A walker asking to enter a cell whose vehicle-present flag is set, while the cell's listed
     contents give *clear*, gets *moving blocker* (claim 4, row WR1). That
     covers a vehicle that has started a step into the cell but whose position is not yet in it
     (claim 1), and an idle vehicle that the walker's claim 4 rows pass over.
   - The building-present flag does not stop a walker from taking a standing position. Buildings keep
     walkers out only through their entry outcomes (claim 4). So a walker can walk into, and stand in,
     the cells of a building that claim 4 passes over, for example a building type that is invisible
     in play.
   - Backing: `measurable` (scenario C2, parts a to d, scenario C10, parts a and b, and scenario
     C13 for the enslaved walker), and scenario C30 for the object-present flag's isolated refusal.
     `unmeasured` for the open-gate exception to that refusal: C30 has no open gate, and no
     scenario here isolates a retained object-present flag together with an open gate.
   - Confidence: high for the vehicle-present flag. Medium for the object-present flag, the
     building-present flag and the vehicle that has started a step into the cell.

### Entry outcomes for a walking infantry unit and a ground vehicle

4. **Entry outcome table for an infantry mover.** For a walking infantry unit and a neighbouring
   cell, the outcome follows from the cell's terrain and contents as set out in parts 4a to 4d.
   Every row of the three tables has a label (WO for overlays, WP for occupants, WR for the
   occupancy record), its own confidence and its own backing. A backing of the form U*n* is
   `unmeasured`, and the list after the tables gives its reason.

   **Destruction-escape use.** S-0312 claim 12 also uses these entry outcomes for passengers
   escaping a destroyed vehicle. In those checks the dying vehicle itself contributes neither a
   listed blocker nor its vehicle-present flag. Other occupants still affect entry normally.
   This exclusion is specific to that vehicle's passenger escape: it does not exempt unrelated
   dying objects or apply to ordinary unloading. For comparison, counting an allied vehicle
   with no movement in progress, no destination and no entry override gives *stationary friendly*
   (outcome 6, row WP16) in an otherwise clear cell. That outcome is rejected by S-0312 claim 12's
   escape gate, which accepts only *clear* and *moving blocker*. This conditional comparison does
   not change the dying-vehicle exclusion or prescribe one outcome for every vehicle state.
   - Backing: `measurable` (S-0312 claim 12's scenario D9, variant g, for the scoped exclusion
     and stationary-vehicle comparison)
   - Confidence: high for the scoped exclusion and conditional comparison

   **Garrison exit use.** S-0508 claim 10 uses these entry outcomes to qualify an exit
   candidate for the first occupant, with initial reference level −1 and no connecting-step
   direction. That claim owns boundary qualification, including the inherited outside-area
   rejection and its exceptions, the missing-cell stand-in of S-0009 claim 44, and
   `quirks.garrison_negative_level_exit`. There is no independent missing-cell refusal.
   S-0508 claim 22 owns the subsequent centre-placement conditions: placement does not
   repeat the entry check or test later occupants' terrain permissions, capacity or usable area.
   These are exit-search and placement uses, not ordinary walking asks.
   - Backing: inherits S-0508 claims 10 and 22, including their `measurable` scenarios G7,
     G8, G9 and G16 and their stated `unmeasured` boundary and exceptional-state cases
   - Confidence: high for no independent boundary gate and no later-occupant terrain test,
     medium for outside-area consequences, as in S-0508 claims 10 and 22

   *4a. Terrain checks.* Terrain checks that give *blocked* belong to S-0211 and S-0212 and are not
   repeated here: height steps (S-0212 claim 10), impassable land, a land type whose walker speed
   percentage is zero, and cells outside the usable area for a unit that is inside it. They hold
   whatever the occupants are, except that the zero walker speed check is left out in the cases of
   this table. The first row that applies decides:

   | Case | Zero walker speed check | Confidence | Backing |
   |---|---|---|---|
   | the objects that count are those of the asked cell's bridge-deck layer (Terms *cell layer*), as for a walker on a bridge deck or stepping onto one (the deck rule of S-0211 claim 9) | not applied | high | C22 (d) |
   | the walker holds a tether link | not applied | medium | U1 |
   | the outcome is decided by the parasite override (row WP1) | not applied: *clear* | low | U2 |
   | the outcome is decided by the vehicle-thief override (row WP6) | not applied: *clear* | medium | C14 (c) |
   | the outcome is decided by the tethered-entering override (row WP15) | not applied: *clear* | low | U1 |
   | every other case, among them an outcome decided by the mission-target override (row WP3) or by no override | applied | medium | C14 (c′) |

   The occupants that count are the objects listed in the cell layer that counts (Terms, *cell
   layer*), the walker itself excluded.

   *4b. Overlays.* An overlay in the cell gives a value as follows.

   | Row | Overlay | Value | Confidence | Backing |
   |---|---|---|---|---|
   | WO1 | a crate, the walker owned by a house that is not human-controlled (the original's computer behaviour; see below) | blocked | medium | C6 |
   | WO2 | a wall that is not completely destroyed, the walker without a primary weapon or with one that cannot damage walls | blocked | medium | C3 (c3) |
   | WO3 | a wall that is not completely destroyed, owned by an allied house, the walker's primary weapon can damage walls | friendly obstacle | medium | C3 (c2), (c5) |
   | WO4 | a wall that is not completely destroyed, owned by another house, the walker's primary weapon can damage walls | enemy obstacle | medium | C3 (c4), (c6) |

   Row WO1 is a rule that the original applies to the units of computer-controlled houses only. An
   AI house counts as human-controlled for every unit rule (glossary *AI house*), so in our engine
   a crate gives its walkers no value, as it gives a human player's. This follows from the
   project lead's decision recorded in the glossary entry *AI house*. It is a rule of our engine
   that holds in every spec, not a quirk of this spec, so this spec defines no switch for it. Row
   WO1 states the original's behaviour, which scenario C6 measures. The original's computer
   player thereby keeps its walkers off crates; our AI may choose to do the same through the
   destinations it orders. Walkers of the neutral and special houses keep row WO1 in our engine
   (Terms, the project lead's decision of 2026-10-10).

   A walker reacts to *friendly obstacle* and *enemy obstacle* alike when it reaches the cell
   (claims 7 and 9). C3 (c2) and (c4) show that a wall gives one of the two obstacle values, and not
   *blocked*. Which of the two an allied wall and an enemy wall give shows in the walker's route
   choice, through the step costs of S-0212 (claim 8): C3 (c5) and (c6).

   *4c. Occupants.* Each occupant fits exactly one row of the table below: the first row, from the
   top, whose description it matches. Rows whose value reads "overrides" are explained after the
   table.

   | Row | Occupant | Value | Confidence | Backing |
   |---|---|---|---|---|
   | | **Special relations to the walker** | | | |
   | WP1 | any object in the cell in which the walker's victim stands, when the walker is a parasite attached to that victim and asks to enter while moving, not for exit (S-0614 claim 13) | overrides: clear | low | U2 |
   | WP2 | the walker's master, when the walker is an enslaved unit and the cell is the master's unloading cell | none, and the walker is excused from the cell's vehicle-present flag (Terms *occupancy record*) | medium | C13 |
   | WP3 | the walker's mission target, or an occupant whose own position (for a building, the centre of its top-left foundation cell, S-0204 claim 3) lies in the cell that is its mission target (Terms), unless that occupant is being erased | overrides: clear, or blocked when that occupant is invulnerable | low | C2 (b) and (e); C14 (a) for the being-erased exception |
   | WP4 | any occupant other than the building itself, in a cell whose building is the walker's mission target, or has its reference point (S-0204 claim 4) in the cell that is the walker's mission target, unless the occupant is being erased | none | low | C17 (a); C17 (b) for the being-erased exception |
   | WP5 | a vehicle, of any house, that the walker is guarding (Terms) | blocked | medium | C11 |
   | WP6 | a vehicle that the walker, of a vehicle-thief type, has as its destination, unless that vehicle is a train | overrides: clear | medium | C14 (b); U3 for the train exception |
   | | **Buildings, by type** | | | |
   | WP7 | a building of a type that is invisible in play | none | medium | C10 |
   | WP8 | a building of a type with the laser-fence flag, in two of its states (Open questions) | none | low | U4 |
   | WP9 | a building of a type with the firestorm-wall flag, its owner's firestorm on | blocked | medium | U5 |
   | WP10 | a building of a type with the firestorm-wall flag, its owner's firestorm off | none | medium | C23 (a) |
   | WP11 | a gate that is open | none | medium | C8 (d) |
   | WP12 | a closed gate whose owner is allied with the walker's house (the gate owner's view) | closed friendly gate | medium | C8 (a); C25 (a) for the view |
   | WP13 | any other closed gate, the walker with a primary weapon | enemy obstacle | medium | C8 (b) |
   | WP14 | any other closed gate, the walker without a primary weapon | blocked | medium | C8 (c) |
   | | **Entering under a tether link** | | | |
   | WP15 | the object the walker is entering, when that object is its destination and the walker holds a tether link (S-0208 Terms) | overrides: clear | low | U1 |
   | | **Allied occupants** (the walker's view) | | | |
   | WP16 | allied vehicle with no movement in progress and no destination | stationary friendly | high | C2 (a), C11 (b) |
   | WP17 | allied vehicle that has movement in progress or a destination, and is between steps or at a chain point (Terms) | moving blocker | medium | C5 (a); U6 for the split from row WP18 |
   | WP18 | any other allied vehicle: it has movement in progress or a destination, and is in the middle of a step (neither between steps nor at a chain point) | none (row WR1 then gives moving blocker when *C* is clear) | medium | C5 (b); U6 for the split from row WP17 |
   | WP19 | allied aircraft, or allied building not decided above | blocked | medium | C15 (a), (b) |
   | WP20 | allied infantry | none (it counts through the occupancy record) | high | C1, C4; C25 (b) for the view |
   | | **Enemy occupants** (not allied, and not decided above) | | | |
   | WP21 | cloaked enemy object, armed walker or not | cloaked blocker | medium | C3 (f), (f2) |
   | WP22 | enemy object other than a terrain object, the walker unarmed | blocked | medium | C3 (b), (a3), (a4); C14 (a) |
   | WP23 | enemy building of a type marked as a bridge repair hut, the walker armed | blocked | medium | C15 (c) |
   | WP24 | other enemy building, the walker armed | enemy obstacle | medium | C15 (d) |
   | WP25 | enemy infantry that appears disguised to the walker's house under S-0516 claim 35, the walker armed | stationary friendly | medium | C3 (e); no-house case below |
   | WP26 | other enemy infantry, the walker armed | none (it counts through the occupancy record) | high | C3 (a), (a2) |
   | WP27 | terrain object, such as a tree | none | medium | C16 (a) |
   | WP28 | any other enemy object (vehicles, aircraft), the walker armed | enemy obstacle | medium | C2 (c) |

   Row WP25 includes a disguise with no disguise house when the other conditions of
   S-0516 claim 35 hold. A missing disguise house does not by itself defeat that predicate.
   This no-house infantry case is `unmeasured`: no ordinary skirmish disguise action is
   known to create it. S-0516 claim 35 owns the predicate; its conditional observation
   scenario applies if that state becomes available. Confidence: high for the no-house
   predicate, medium for the entry outcome as in row WP25.

   *Armed walkers and the firing check (Q-2396).* Rows WP13, WP24 and WP28 test only the walker's
   weapons as the rows state (a primary weapon for WP13, *armed* in Terms for WP24 and WP28). None
   of them asks the firing check (S-0301 claim 9) about the occupant. So, where one of these rows
   applies, an armed walker gets
   *enemy obstacle* from an enemy building, a closed enemy gate or another enemy object even when
   the firing check of the weapon it would use against that occupant (S-0308) answers *illegal*,
   for example through a versus value of 0 against every building armour (S-0304 claim 6). The
   path search reads the same entry outcome, so it gives such a step the *enemy obstacle* cost of
   S-0212 claim 8, and a path can run through that cell when every detour costs more. Claim 9 says
   what the walker then does (`quirks.walker_attacks_unhittable_obstacle`, Edge cases).
   - Backing: `measurable` (scenario C15: parts e1 to e3 for row WP24, part e6 for row WP13 with
     a closed enemy gate, part e7 for row WP28 with an enemy vehicle).
   - Confidence: high that rows WP13, WP24 and WP28 ask no firing check. Medium for the entry
     outcome and the path cost of each of the three rows, as for those rows in the table.

   A terrain object has no owner, so it is never allied (Terms *allied*) and always falls under the
   enemy rows. It gives no value of its own (row WP27), but it can still decide the outcome through
   the side positions it marks (claim 1) and through the terrain checks of S-0211.

   For some walkers *area guard* is a behaviour with a mission target (an engineer-like unit, Open
   questions). Such a walker that guards a vehicle, while that vehicle is still its destination, is
   decided by row WP3 (*clear*), not by row WP5.

   *The contents value C.* Write *v*(*o*) for the value that the overlay table or the occupant table
   gives an overlay or occupant *o*, or "none" when its row gives none or no row of either table
   matches it. Severity is the order of the entry outcomes (Terms).

   > *C* = the most severe *v*(*o*) over the overlays and the occupants that are not on an
   > "overrides" row, leaving out every *v*(*o*) that is none. *C* = *clear* when no value is left.

   *Overriding occupants.* Number the occupants *o*₁, …, *o*ₙ in listing order (claim 13), and give
   the overlays index 0, ahead of every occupant. Let *k* be the smallest index from 1 to *n* for which
   *o*ₖ is on an "overrides" row. When such a *k* exists, the outcome is:

   | Condition | Outcome |
   |---|---|
   | At least one overlay or occupant with index less than *k* has value *blocked* | *blocked* |
   | No overlay or occupant with index less than *k* has value *blocked*, and *o*ₖ is on row WP3 and is invulnerable | *blocked* |
   | No overlay or occupant with index less than *k* has value *blocked*, and the preceding WP3-invulnerability condition does not apply | The value of *o*ₖ's row (*clear*) |

   Consequences:
   - In such a cell neither *C* nor the record table plays a part. Values before index *k* that are
     less severe than *blocked*, and every value after index *k*, change nothing (Edge cases,
     `quirks.occupancy_override_hides_later_blockers`; scenario C18).
   - A mission target that is a building is listed after every object that is not a building
     (claim 13), and row WP4 makes the other occupants of its cell give no value. Such a cell is
     *clear* for the walker unless an overlay gives *blocked*, an occupant listed before the
     building that is being erased gives *blocked* through its own row (scenario C17 (b)), the
     building is invulnerable, or the zero walker speed check applies.
   - The override rule itself: confidence medium. Backing: scenarios C18 (a blocker listed after an
     override changes nothing), C17 (b) (a *blocked* listed before an override decides) and C2 (b).

   *4d. The entry outcome.* When no occupant overrides, the outcome follows from *C* and the
   occupancy record of the cell layer that counts (Terms). The first row of this table that applies
   decides:

   | Row | Occupancy record of the cell layer | Applies when *C* is | Entry outcome | Confidence | Backing |
   |---|---|---|---|---|---|
   | WR1 | the vehicle-present flag is set and the mover is not excused from it (Terms *occupancy record*) | clear | moving blocker | high | C2 (c), (d); C5 (b) |
   | WR2 | the infantry house is a house (not none) that the walker's house is not allied with, the walker unarmed | any value | blocked | high | C3 (g), C16 (c) |
   | WR3 | the infantry house is a house (not none) that the walker's house is not allied with, the walker armed | any value | the more severe of *C* and *enemy obstacle* | high | C3 (a), (h); C12; C25 (b) for the view |
   | WR4 | the infantry house is allied, the three side positions are all taken, and exactly three allied infantry listed in the cell layer have no movement in progress (Edge cases, `quirks.stationary_friendly_exactly_three`) | clear or cloaked blocker | stationary friendly | high | C4 (a) to (f) |
   | WR5 | the infantry house is allied, the three side positions are all taken, any other number of them have no movement in progress | clear or cloaked blocker | moving blocker | high | C4 (g), C16 (b) |
   | WR6 | the infantry house is none and the three side positions are all taken | clear | blocked | high | C16 (d) |
   | WR7 | anything else | any value | *C* | high | all scenarios |

   By claim 1, infantry alone never produce the state of row WR6: the infantry house is a house
   whenever an infantry unit holds a side position. A terrain object whose side-position pattern
   marks all three side positions does produce it, in a cell where no infantry has marked since the
   game began. No walker's path leads into such a cell, since the pattern puts it in a passability
   category that the infantry movement zone may not cross (S-0211 claims 11 and 12). The row shows
   where the entry outcome is asked without a path, in the cell scan of a scatter with a threat
   point (S-0208 claim 12), which passes over such cells.

   Consequences:
   - For an armed walker, enemy infantry counts through the record (its house is the infantry house)
     and gives *enemy obstacle*. For an unarmed walker, an enemy unit in the cell gives *blocked*,
     with the one exception of the next bullet.
   - *Cloaked blocker* is the outcome only when *C* is *cloaked blocker* and none of rows WR2 to
     WR5 applies. In practice it comes from a cloaked enemy vehicle, aircraft or building, and then even
     to an unarmed walker. A cloaked enemy *infantry* unit sets the infantry house like any other
     (claim 1). It therefore gives *blocked* to an unarmed walker and *enemy obstacle* to an armed
     one, and it is not revealed by the walker (claim 9). The exception is a cell whose infantry
     house has been forgotten or was last set by an allied unit (Edge cases).
   - When an allied or enemy vehicle has set the vehicle-present flag and the listed contents give
     *clear*, the outcome is *moving blocker*. The infantry house plays no part then, even when it is
     an enemy's.
   - A fourth allied infantry unit at the centre or top position that also has no movement in
     progress makes the count four, and the outcome *moving blocker* (Edge cases,
     `quirks.stationary_friendly_exactly_three`). So does a terrain object's mark on a side
     position, which leaves fewer than three allied infantry to count.
   - The house in the record need not be the house of any infantry still in the cell (claim 1 and
     Edge cases). The rows above use the recorded house.
   - A walker that has chosen its walking target in a cell holds a position there before it is
     listed there (claims 1 and 13). During that time the cell shows a taken side position and the
     walker's house with no listed occupant, so rows WR2 and WR3 apply to it as they would to a unit
     standing there.

   *Slave field qualification.* S-0402 claim 26 owns the slave's field search. Its
   candidate tests require this claim's infantry entry outcome to be *clear* and a
   connection in the slave's own movement zone from its heading-point cell. A partly
   occupied infantry cell can qualify when those tests pass:
   another slave or unit is not a blanket occupancy refusal. This does not replace
   S-0402 claim 26's own-cell tiberium-land shortcut, which bypasses the candidate
   tests.
   - Confidence: medium. Backing: `measurable` (S-0402 claim 26, scenario H21's scan
     variants).

   *Reasons for the `unmeasured` backings of claims 4 and 5.*
   - U1 (tether link): the successful infantry factory or cloning-building release exchange gives
     an infantry unit a tether link (S-0417 claim 10). That identifies a tether situation, but not
     a setup isolating the zero walker speed exception or row WP15's entering override. The
     released unit is leaving its factory, not established to be entering its destination under
     the tether. No setup is established that retains the tether while testing either effect,
     and a tether link is not an imported field that a test file could set. Both effects remain
     `unmeasured`.
   - U2 (parasite): no stock infantry type is a parasite, and an attached parasite rides inside its
     victim instead of walking or driving, so no setup, with or without a test file, has a moving
     parasite with a victim. This uncertainty concerns movement entry only, not the separate
     exit test owned by S-0614 claim 13.
   - U3 (train exception of row WP6): an order on a train gives a vehicle thief *select* (S-0214
     table I, row I14), so no order makes a train a thief's destination.
   - U4 (laser fence): which two states of a laser-fence section give no value is not known (Open
     questions), so no setup can put a section in a chosen one of them.
   - U5 (firestorm on): no event in the original turns a house's firestorm on (S-0211), so the row
     can never apply.
   - U6 (split of rows WP17 and WP18, and of rows VP22 and VP23): the two rows of each pair differ
     only when the vehicle's cell layer also holds an object giving *cloaked blocker*, or when the
     vehicle has not set the vehicle-present flag of the cell it is listed in. The first needs a
     cloaked enemy object and an allied vehicle in one cell layer, which vehicles do not share
     outside a passing moment. Claim 1 does establish release before the vehicle's position leaves
     its cell, but no scenario here isolates that interval together with the between-steps or
     chain-point condition needed to distinguish the rows. Scenario C28 observes flags, not that
     conditional entry outcome.
   - U7 (vehicle crate row VO1): it applies in a campaign game only, and campaign games are outside
     this batch (as for claim 14).

5. **Entry outcome table for a ground vehicle mover.** For a ground vehicle and a neighbouring
   cell, the outcome follows from the cell's terrain and contents as set out in parts 5a to 5f. The
   tables have the same shape as those of claim 4 for walkers, but their rows differ (labels VO, VP
   and VR). S-0212 claim 27 summarises the rows its path costs rely on; this claim owns the full
   table, as decided on 2026-10-10. S-0212's current correction-batch claim 27 agrees with
   rows VP21 to VP23, the first-override precedence, the building exceptions and the wood-wall
   case. Confirm those coordinated texts at hand-off as listed below.

   *5a. Terrain checks.* Terrain checks that give *blocked* belong to S-0211 and S-0212 (claim 10)
   and are not repeated here, among them a land type on which the vehicle's speed percentage is
   zero. That check is left out in the cases of this table; the first row that applies decides, and
   in every other case the check applies:

   | Case | Zero speed check | Confidence | Backing |
   |---|---|---|---|
   | the objects that count are those of the asked cell's bridge-deck layer (Terms *cell layer*; the deck rule of S-0211 claim 9) | not applied | high | C22 (d); S-0211 scenario L6 |
   | the outcome is decided by an override (rows VP2, VP4 to VP7 and VP17) | not applied: *clear* | medium | C24 (d2) |

   *5b. Underground travel.* A vehicle whose locomotor travels underground has one rule more than
   the tables below: where that rule applies, it makes a cell *blocked* whatever the cell's
   overlays and occupants, even when an occupant would override, and where it does not apply it
   leaves the outcome as the tables give it. It never gives any other value. Which cells it makes
   *blocked* (among them cells that hold a building or a terrain object, sloped cells and
   high-bridge cells), in which situations it applies, and the game-wide setting that turns it off
   belong to the specs of underground travel (Open questions). No other locomotor has a rule of this
   kind: for every other vehicle the outcome is the one the tables give.
   - Confidence: high that the underground locomotor is the only one with such a rule, and that the
     rule either gives *blocked* or leaves the outcome alone. Medium that its *blocked* holds even
     when an occupant would override.
   - Backing: `unmeasured`. Stock data gives no unit type this locomotor, and a test file that
     gives a type this locomotor could not tell this rule from the rest of underground travel,
     which no spec states yet. The specs of underground travel own the rule and its measurement.

   The occupants that count are the objects listed in the cell layer that counts (Terms, *cell
   layer*). *Has a primary weapon* is the test of Terms *armed* (the primary weapon at the
   vehicle's current veterancy rank). *Armed* means a vehicle's test of Terms *armed*: a primary
   weapon, or a train.

   *5c. Overlays.*

   | Row | Overlay | Value | Confidence | Backing |
   |---|---|---|---|---|
   | VO1 | a crate, the vehicle owned by a house that is not human-controlled, in a campaign game only (the original's computer behaviour, as row WO1) | blocked | medium | U7 |
   | VO2 | a wall section that the vehicle crushes (its overlay type is crushable and the vehicle is a crusher, or the vehicle's movement zone is *crusher-all*), owned by an allied house | friendly obstacle | medium | C20 (f′) |
   | VO3 | a wall section that the vehicle crushes, owned by another house | none | medium | C20 (f2) |
   | VO4 | any other wall section, the vehicle without a primary weapon | blocked | medium | S-0212 scenario P9 |
   | VO5 | any other wall section, the warhead of the vehicle's primary weapon can damage walls, or can damage wood while the overlay type's armour is wood | friendly obstacle (allied owner), enemy obstacle (other owner) | medium | C20 (j1), (j2); (j4) for the wood case |
   | VO6 | any other wall section | blocked | medium | C20 (j3) |

   *5d. Occupants.* Each occupant fits the first row, from the top, whose description it matches.
   Rows whose value reads "overrides" work as in claim 4.

   | Row | Occupant | Value | Confidence | Backing |
   |---|---|---|---|---|
   | | **Special relations to the vehicle** | | | |
   | VP1 | the vehicle itself | none, and the vehicle is excused from the cell's vehicle-present flag (Terms *occupancy record*) | medium | C21 (b) |
   | VP2 | any object in the cell in which the vehicle's victim stands, when the vehicle is a parasite attached to that victim and asks to enter while moving, not for exit (S-0614 claim 13) | overrides: clear | low | U2 |
   | VP3 | a building the vehicle is in contact with, when the cell is not one of that building's blocking cells (Terms) | none | low | C24 (c1) to (c3) |
   | VP4 | a vehicle of a type with the small-visceroid flag, the mover's type having that flag too, of any house | overrides: clear | medium | C20 (c) |
   | | **Buildings, by type** | | | |
   | VP5 | a building that absorbs units, the vehicle's behaviour *enter* and its destination that building, unless the building is being erased | overrides: clear | low | C24 (b) |
   | VP6 | a building that grinds units, the vehicle's behaviour *eaten* and its destination that building, unless the building is being erased | overrides: clear | low | C24 (a) |
   | VP7 | a building that grinds units and that is also listed in the cell the vehicle stands in | overrides: clear | low | C24 (a2) |
   | VP8 | a gate that is open | none | medium | C8 (d) with a vehicle (scenario C20) |
   | VP9 | a closed gate whose owner is allied with the vehicle's house (the gate owner's view) | closed friendly gate | medium | C8 (a) with a vehicle (scenario C20); C25 (a2) for the view |
   | VP10 | any other closed gate, the vehicle with a primary weapon | enemy obstacle | medium | C20 (h1) |
   | VP11 | any other closed gate, the vehicle without a primary weapon | blocked | medium | C20 (h2) |
   | VP12 | a building of a type on which vehicles are repaired, or of a bunker type, when the cell is not one of its blocking cells | none | low | C23 (c), (d) |
   | VP13 | a building of a type that is invisible in play | none | medium | C10 with a vehicle (scenario C20) |
   | VP14 | a building of a type with the laser-fence flag, in two of its states (Open questions) | none | low | U4 |
   | VP15 | a building of a type with the firestorm-wall flag: its owner's firestorm on / off | blocked / none | medium | on: U5; off: C23 (b) |
   | VP16 | a building of a type with a bib, in a cell of its foundation whose neighbour one step along +x is not a cell of that building's foundation (for a rectangular foundation of *W* × *H* cells, the cells *W* − 1 cells along x from its top-left cell: its last column along x) | none | low | C23 (e) |
   | | **Behaviour relations, any kind of occupant** | | | |
   | VP17 | a vehicle that is the vehicle's destination, the vehicle's behaviour *enter* | overrides: clear | medium | C24 (d) |
   | VP18 | the object that is the guard spot of the vehicle, its behaviour *area guard* | blocked | medium | C24 (e) |
   | | **Allied occupants** (the vehicle's view) | | | |
   | VP19 | allied building that does not count as moving | blocked | medium | S-0212 scenario P7 |
   | VP20 | allied vehicle, infantry unit or aircraft that does not count as moving (S-0212 claim 27) | stationary friendly | medium | S-0212 scenario P7 (vehicles); C20 (a1), (a2) (infantry), (d1), (d2) (aircraft); C25 (c) for the view |
   | VP21 | allied unit that counts as moving, in the close case of S-0212 claim 27 | blocked | medium | S-0212 scenario P14 run 5 |
   | VP22 | allied vehicle (or aircraft, low) that counts as moving and is between steps, or any allied unit that counts as moving and is at a chain point (Terms) | moving blocker | medium | C5 (a) with a vehicle (scenario C20); S-0212 scenario P14; U6 for the split from row VP23 |
   | VP23 | any other allied unit that counts as moving (an infantry unit among them) | none (the record table then decides) | medium | C20 (b); U6 for the split from row VP22 |
   | | **Enemy occupants** (not allied, and not decided above) | | | |
   | VP24 | cloaked enemy object | cloaked blocker | medium | C20 (g1) to (g3) |
   | VP25 | enemy object crushable by the vehicle (the vehicle a crusher) | none, but it counts as a *crushable occupant* in the record table | medium | C3 (d), (d2) |
   | VP26 | enemy object, the vehicle not armed | blocked | medium | C20 (e2); C3 (d); C20 (e3) for a train, which is armed |
   | VP27 | enemy building of a type marked as a bridge repair hut | blocked | medium | C20 (i1) |
   | VP28 | other enemy building | enemy obstacle | medium | C20 (i2) |
   | VP29 | terrain object, such as a tree: the weapon the vehicle would use against it has a warhead that can damage wood / cannot. For a train without a primary weapon this spec states no outcome (Open questions) | enemy obstacle / blocked | medium | C20 (k1), (k2) |
   | VP30 | any other enemy object (vehicles, infantry, aircraft) | enemy obstacle | medium | C20 (e1), (e3), (e4); C3 (d) |

   A terrain object is never allied (Terms *allied*), so for a vehicle it falls under row VP26 when
   the vehicle is not armed, and otherwise under row VP29.

   *C* and the override rule are defined as in claim 4, over these two tables.

   *5e. The entry outcome.* When no occupant overrides, the first row of this table that applies
   decides:

   | Row | Contents, and occupancy record of the cell layer that counts | Entry outcome | Confidence | Backing |
   |---|---|---|---|---|
   | VR1 | *C* is not *clear* | *C* | medium | all rows above |
   | VR2 | some occupant is a crushable occupant, and the vehicle-present flag is set and the mover is not excused from it | *clear* when the first vehicle listed in the cell's **ground** layer is crushable by the vehicle; *moving blocker* otherwise, also when the ground layer lists no vehicle. The ground layer is read even when the deck layer counts (Edge cases, `quirks.crusher_entry_ground_layer`) | medium | C3 (d2); C22 (c) for the ground layer |
   | VR3 | some occupant is a crushable occupant | clear | medium | C3 (d) |
   | VR4 | no standing position is taken and the vehicle-present flag is clear or the mover is excused from it | clear | high | all scenarios |
   | VR5 | the vehicle-present flag is set and the mover is not excused from it | moving blocker | medium | C26 (a1), (a2) |
   | VR6 | the infantry house is a house that the vehicle's house is allied with | moving blocker | medium | C20 (b); C26 (e) for the view |
   | VR7 | otherwise (a standing position is taken; the infantry house is none or not allied): the vehicle is a crusher | clear | medium | C26 (b) |
   | VR8 | otherwise, the vehicle's type is a train | enemy obstacle | medium | C26 (f) |
   | VR9 | otherwise, the vehicle without a primary weapon | blocked | medium | C26 (d) |
   | VR10 | otherwise, the projectile of the vehicle's primary weapon can hit ground targets / cannot | enemy obstacle / blocked | medium | C26 (c), (c2) |

   Rows VR7 to VR10 decide only when *C* is clear while a standing position is taken under a house
   that the vehicle is not allied with, or under none. Since every listed infantry unit, and every
   terrain object, gives a vehicle a value of its own or counts as a crushable occupant, except
   allied infantry that counts as moving (and, for a terrain object, except possibly towards a
   train without a primary weapon, Open questions), these rows are reached in two states: while an infantry
   unit that the vehicle is not allied with has chosen its walking target in the cell and is not yet
   listed there (claim 4, last consequence), and through a stale or forgotten infantry house (Edge
   cases) in a cell that holds only allied infantry that counts as moving. Scenario C26 uses the
   first.

   **Ordinary aircraft on a bridge deck.** The object-present flag alone gives no
   vehicle entry refusal on either cell layer: the record table above does not test it.
   On a high bridge over level traversable ground, an ordinary allied aircraft resting
   on deck without a destination is listed only on deck (claim 13). With no other
   obstacle or exceptional occupant relation, a vehicle approaching below gets *clear*,
   while one approaching on deck gets *stationary friendly* from the listed aircraft
   under row VP20, not from its object-present flag. After ordinary take-off removes
   that listing, deck entry is *clear* with every other condition unchanged. Enemy
   aircraft and moving or special-relation cases retain their occupant-table outcomes.
   S-0501 claim 44 owns this ordinary-flight consequence; teleport reservations are
   separate below, and landing or placement tests keep their own surface rules.
   - Backing: `measurable` (S-0501 scenario AC27, occupancy and vehicle variants)
   - Confidence: high for flag exclusion; medium for the combined bridge entry consequences

   **Aircraft reservation consequence.** An otherwise empty ground layer beneath a
   teleporting aircraft's bridge landing has only its retained object-present flag.
   With no other terrain restriction, occupant, standing-position mark, infantry house
   or vehicle-present flag, ordinary infantry and ground-vehicle entry outcomes are
   both *clear*. That flag alone gives no value in either entry table. Ordinary infantry
   standing-position choice nevertheless refuses it unless an open gate stands there
   (claim 3), so the walker can stop and ask again despite a *clear* outcome (claim 6).
   The same distinction holds after aircraft destruction leaves the ground flag behind.
   Deck entry still sees the listed aircraft through the ordinary occupant tables.
   - Backing: `measurable` (scenario C30; S-0207 scenario D22, run 14)
   - Confidence: medium

   **Teleport reservation consequence.** Before a teleporting vehicle jumps to an otherwise
   empty high-bridge cell over level ground, an approaching ground vehicle reads the ground
   reservation and gets *moving blocker* from row VR5. A vehicle approaching on the deck reads
   the unmarked deck record and gets *clear* from row VR4. These consequences require no other
   terrain restriction, occupant, reservation or exception to decide the outcome. After deck
   placement both records are marked as claim 1 describes; the deck's listed vehicle also
   counts through the occupant table. Reservation lifecycle belongs to S-0207 claim 20,
   while this claim retains ownership of the entry outcomes.
   - Backing: `measurable` (S-0207 scenario D22, run 13)
   - Confidence: medium

   Consequences for infantry in the cell:
   - Allied infantry that does not count as moving gives *stationary friendly* through its own row,
     whatever the occupancy record shows. So a unit standing at the centre or top position still
     keeps an allied vehicle out after the unit it shared its record entry with has left (S-0208
     `quirks.top_position_counts_as_centre` changes the record only, not this row).
   - Allied infantry that counts as moving gives no value itself, outside the close case of S-0212
     claim 27 (where its row gives *blocked*). When nothing else gives one, the record gives *moving
     blocker*, provided the cell's infantry house is allied with the vehicle's house.
   - Enemy infantry gives *clear* to a crusher when every enemy unit in the cell is crushable by it,
     nothing else in the cell gives a value, and either the vehicle-present flag reads as clear for
     the crusher or the first vehicle listed in the cell's ground layer is one it can crush (rows
     VR2 and VR3). With the flag set and no crushable vehicle listed first in the ground layer, it
     gets *moving blocker*, also when the vehicle that set the flag has only started its step into
     the cell. An enemy unit it cannot crush gives
     *enemy obstacle* to an armed vehicle and *blocked* to one that is not armed. A disguise plays no
     part for a vehicle.

   Consequences against claim 4: an allied aircraft resting on the ground that does not count as
   moving gives a vehicle *stationary friendly* (a walker *blocked*); a vehicle's "armed" asks for a
   primary weapon (or a train), a walker's for a positive mean damage (Terms *armed*); a terrain object gives a
   vehicle a value (a train without a primary weapon aside, Open questions) and a walker none; and a vehicle never gets the infantry-house rows when *C* is
   not *clear*.

   *5f. The outcome that governs the start of a step.* When a vehicle whose locomotor drives, moves
   as a ship, or walks begins a movement step, the next cell's entry outcome is checked. The value
   that governs the step is that outcome with the substitutions of this table. What a driving
   vehicle does with that value is S-0207 claim 15. For the ship and walking-vehicle locomotors it
   belongs to their own movement specs, which are not written yet. The substitutions hold for this
   check only: path costs (S-0212) and order targeting (S-0213, S-0214) use the entry outcome
   unchanged.

   | Vehicle | Substitution | Confidence | Backing |
   |---|---|---|---|
   | a train that drives or moves as a ship | every outcome other than *blocked* counts as *clear* | medium | C27 (a) for driving, (b) for a ship |
   | a vehicle whose type has the crusher flag (the crusher veterancy ability does not count here) | *friendly obstacle* and *enemy obstacle* count as *clear* when the cell's overlay is of the overlay type with registration index 0, whatever object or overlay gave the obstacle value (Edge cases, `quirks.crusher_overlay_index0_clear`) | medium for driving and walking, low for a ship | C20 (f) for driving, (f3) for walking. For a ship `unmeasured`: the obstacle value must come from a cell whose overlay is of the type with registration index 0, and no overlay type is known that a map can place in a water cell, where a ship's steps lead |

   The train substitution is not a quirk: it matches the train's path costs (S-0212 claim 8), in
   which every outcome other than *blocked* costs the same.

### What a blocked walker does

6. **Walking on.** When the next cell of its path is *clear*, a walker chooses its walking target
   there on its asking tick (S-0208 claim 3). When it then steps towards it belongs to S-0208
   (claim 6).
   - A *clear* cell can still have no free standing position: its object-present flag
     can be set without any listed occupant giving a value in claim 4, and without an
     open gate (claim 3). A teleporting aircraft's retained ground reservation below a
     bridge supplies this case, even after the aircraft is destroyed. The walker stops
     for that tick and asks again on the next one.
   - Backing: `measurable` for walking on (scenario C1) and for the no-position case
     (scenario C30; S-0207 scenario D22, run 14).
   - Confidence: high for walking on, medium for the no-position case

7. **First reaction to an obstacle: an immediate new path.** When the next cell gives *cloaked
   blocker*, *stationary friendly*, *friendly obstacle*, *enemy obstacle* or *blocked*, the walker
   stops, discards its path and searches a new one on the asking tick itself, whatever its path
   retry timer shows (S-0212 owns the search). For a cloaked blocker, that search sees the
   ground-layer uncloaking caused by claim 9; bridge-deck objects are not uncloaked by it.
   - When the repeated ask (Terms *asking tick*), about the first cell of the new path, gives
     *clear* with a free position, the walker chooses its walking target there on the same asking
     tick (claim 6). So it loses no tick against
     a free cell boundary: from its arrival in front of the obstacle to its first step along the new
     path, it stands still exactly as long as it does at a free boundary.
   - When the repeated ask gives one of these outcomes again, the walker reacts as in claims 8 and
     9, also on the asking tick.
   - When the repeated ask gives *blocked*, the walker keeps its new path and waits for the rest of
     that tick.
   - When the search finds no path, S-0212 (claim 23) says what follows.
   - *Closed friendly gate* and *moving blocker* never cause this immediate new path (claims 9 and 10).
   - *Later ticks.* A walker that still stands on its standing position with a path step ahead and
     no walking target asks again on every later tick, and each of these asks is a first reaction
     of its own (a first ask), not a repeated ask. That holds after a repeated ask that gave
     *blocked* or *cloaked blocker* (claim 9) and after the cell-wide request of claim 8. When the
     next cell gives one of the five outcomes above again, the walker again discards its path and
     searches a new one at once, whatever its path retry timer shows, and makes the repeated ask on
     that tick and reacts to its outcome then. So while the obstacle stays, the walker searches a new path on every tick, and a
     route that has opened is taken from the first asking tick after it opens.
   - Backing: `measurable` (scenario C19, parts a to d, for the asking tick, and part e for later
     ticks; scenarios C3, C4 and C8 for the reactions). `unmeasured` for the possible short-path
     exception of a walking jumpjet type (Open questions): these scenarios use ordinary walking
     types and establish no setup isolating that exception.
   - Confidence: medium. Low for an infantry type whose own locomotor is a jumpjet locomotor (Open
     questions).

8. **Stationary friendly: give up or ask them to move.** The walker stops for good, and its move
   counts as finished, when all of these hold:
   - its locomotor destination (S-0208 Terms; under a move, the position of its destination) is
     less than the close-enough distance away from its own position (Formulas F1),
   - it is in contact with nothing (Terms). A unit entering a transport, for example, may be in
     contact with it,
   - the world height of that locomotor destination differs from the walker's own world height by
     less than 208 leptons, two height levels of 104 leptons: |*z*ᵈ − *z*ʷ| < 208, where *z*ᵈ is
     the locomotor destination's world height and *z*ʷ the walker's, an exact integer comparison in
     leptons. Both are world heights, so a bridge deck counts with its height,
   - its own cell's land type is not *tunnel* (S-0211).

   Otherwise it sends a scatter request to every object listed in the cell layer it tried to enter,
   with no threat point, forced and move-anywhere (claim 11), and waits for the rest of that tick.
   Its asks on later ticks are first reactions again (claim 7). So when the next cell still gives
   *stationary friendly* after the new search of a later tick, the walker sends the request again.
   - Backing: `measurable` (scenario C4, parts a, b and d to f, with part d for the height condition
     and part f for the contact condition, and scenario C19, part c, for the request being sent on
     the asking tick)
   - Confidence: medium. High that the condition is contact of any kind, not only a tether link.
     When an entering unit's contact begins belongs to the specs of that exchange (S-0215).

9. **Obstacles that can be removed.**
   - *Closed friendly gate.* The walker makes the gate open and waits, without searching a new path,
     and walks on once the gate is open.
   - *Friendly obstacle* or *enemy obstacle*, on the repeated ask (Terms *asking tick*; claim 7):
     - Only the first object listed in the cell (claim 13) is considered. The alliance test takes
       the walker's view (Terms): when the walker's house is not allied with that object, the
       walker takes the object as its target and attacks it. An object without an owner, such as a
       terrain object, is never allied (Terms), so a terrain object listed first is taken as the
       target. When the walker's house is allied with the object, nothing is attacked.
     - When no object is listed in the cell and the cell holds a wall, the walker takes the wall as its
       target. That includes an allied wall: no alliance test applies to the wall. Whether it then
       fires on an allied wall belongs to the combat specs (Open questions; a possible switch is among the Questions for the project lead).
     - In every case the walker also stops and discards its path.
     - *The take is an override.* Taking the object, or the wall's cell, as its target is an
       override (S-0215 claim 10) to *attack* with that target and no destination. The walker's
       current behaviour, or its queued one when one is queued, is suspended, and its earlier
       target and destination are put aside.
     - *After the take (Q-2396).* In *attack* the walker approaches and engages its target as
       S-0215 claims 23 and 37 and the combat specs give. When the firing check answers *illegal*
       for that target, the walker does not fire, and S-0309 claim 29 drops the target at a check
       tick, except in the charges case below. That drop does not revert the override (S-0215
       claim 10). When neither a search (S-0309 claim 23 (a), as below) nor a retaliation (S-0215
       claim 18) gives the walker a target before its next *attack* processing, that processing
       finds no target, and the walker falls back to rest (S-0215 claims 13 and 14) with its
       destination still put aside:
       - The common effects of that fall-back come first (S-0215 claim 13), with or without
         attack-move memory. When a row of the tables of S-0215 claims 13 and 14 gives the walker
         a destination or an order at that fall-back, those claims and the order specs give what
         follows, and this claim adds nothing. The cases below are for a walker to which no such
         row applies, such as the walker of scenario C15. The refresh of attack-move memory that
         comes before those tables (S-0215 claim 13) is not such a row: S-0309 claim 23 (b)
         decides it, as below.
       - Without attack-move memory, such a walker rests in front of the obstacle and does not
         move on by itself. Its suspended behaviour stays suspended until a revert cause of
         S-0215 claim 10 meets a target that it holds later. A search does not take the same
         occupant again while the firing check answers *illegal* (S-0309, table C).
       - With attack-move memory, S-0309 claim 23 (b) decides at that fall-back. The take neither
         sets nor ends the *engaged* mark (S-0309 Terms, claim 23 (c)). So the walker is engaged at
         the fall-back in two cases. It was engaged already when it took the occupant, for example
         in *attack* towards an earlier target that a targeting-timer search of its attack move
         found. Or one of its targeting-timer expiries came while it held the occupant, and S-0309
         claim 23 (a) then kept that target and queued *attack*. When it is engaged, the attack
         move resumes, which ends the mark: the walker searches a path to the remembered
         destination again, meets the same cell when that path leads through it, and takes the
         occupant again by this claim. From then on each targeting-timer expiry acts by S-0309
         claim 23 (a), by its rows for a memory that holds a destination:
         - Not engaged: a range search runs, and it keeps the target the walker has. The occupant
           is never an opportunity target, since the take is a setting of a target and not a
           search (S-0309 claim 24). So an expiry while the walker holds the occupant makes it
           engaged. An expiry while it holds no target, as on the walk between a resumption and
           the next take, makes it engaged when the search finds another target, such as an
           enemy within its search range. It then queues *attack* on that target, and its path
           towards it can lead through the same cell.
         - Engaged, with its target within reach (S-0309 claim 23): nothing happens. No search
           runs, the timer is not restarted, and the walker stays engaged and keeps its target.
         - Engaged, with no target within reach: a range search runs that does not keep the
           present target. A walker that holds the occupant out of reach loses it there, at a
           tick that need not be a check tick, and takes another target when the search finds
           one. It stays engaged either way, so the fall-back that follows a loss without another
           target resumes the attack move again.

         At the first fall-back that finds it not engaged, its memory is removed and it rests as
         above.
       - The walker has no give-up count and no timer of its own for this. The cycle also ends
         when the occupant leaves the cell or is destroyed, or at a new order.
       - *Charges case.* A walker with explosive charges (by type or veterancy ability) that takes
         a building that accepts charges and is not invisible meets the first infantry row of
         S-0215 claim 23 at its first *attack* processing after the take, if it still holds the
         building then. That row makes the building its destination and queues *sabotage*. The
         take does not touch the processing timer (S-0215 claims 3 and 10). What follows is
         stated for a walker with no queued behaviour at the take: a queued behaviour stays
         queued at the override (S-0215 claim 10), and its start is S-0215's (claims 3, 8 and 11),
         not claimed here. The queued *sabotage* becomes current at the first per-tick start, the
         late start included, at which the walker is ready to change (S-0215 claim 11). S-0309
         claim 29 decides in the walker's turn of the first check tick after the take, after that
         turn's first per-tick start (*Place in the turn*). When *sabotage* is current there, no
         check is made, nor in any later turn while *sabotage* stays current, so the walker keeps
         the building as its target and does not rest as above. S-0215 claim 28 and S-0519 claim 13
         own what follows. Otherwise *attack* is current there, and the check drops the building
         as for any other walker. That happens when the check tick comes no later than that first
         processing, or when the walker is not ready at any per-tick start from that processing to
         the check. What the walker does after such a drop, with *sabotage* queued, is S-0215's
         (claims 11, 13 and 28) and is not claimed here. An enemy object that is not a building
         (row WP28) never meets that row.

       With `quirks.walker_attacks_unhittable_obstacle` off, rows WP13, WP24 and WP28 give
       *blocked* instead of *enemy obstacle* where the Edge cases entry says, for a walker with
       charges too, so the walker never takes the occupant this way.
       - Backing: `measurable` (scenario C15). Part e1 backs the rest without attack-move memory.
         Part e2 backs the attack-move cycle and its end: after each loss the walker either takes
         the occupant again or rests and stays at rest. Which of the two follows a given loss
         (whether a targeting-timer expiry came while the walker held the occupant) is
         `unmeasured`: a request's bounds and result do not tell the targeting timer's restarts
         from other draws with the same bounds (S-0215 claims 6 and 33). A walker engaged before
         the take, and the re-engagement by a search that finds another target, are `unmeasured`:
         no set-up is proposed, since they need a walker whose search range exceeds its weapon's
         range (S-0309 claim 8), which C15's test file does not give. C15 puts the occupant
         within reach, so its parts back the engaged row with the target within reach. The
         engaged row with the occupant out of reach is `unmeasured` here: no set-up is proposed
         for it, since an occupant out of reach also brings in the approach of S-0215 claim 37,
         which this spec does not give. The rows of S-0215 claims 13 and 14 that give a
         destination or an order keep those claims' backing: C15 excludes them. Part e5 backs
         the override, which its revert at the target's destruction tells apart from a plain change
         of behaviour. Parts e6 and e7 back the same rest for a closed enemy gate and an enemy
         vehicle. The charges case is `unmeasured`: no set-up is proposed that keeps a planted
         charge, the walker's readiness and a queued behaviour out of the comparison.
       - Confidence: medium for the override, the rest without attack-move memory, the
         attack-move cycle and its end, the resumption of a walker engaged before the take, and
         the re-engagement by a search. Medium for the loss of an occupant out of reach, as for
         S-0309 claim 23's searches. Medium for the common effects coming first, as S-0215 claim
         13 rates them. Medium for the charges case, as for the conditions of S-0215 claim 23's
         infantry rows and for S-0215 claim 11's readiness, which decides when *sabotage* starts.
   - *Cloaked blocker.* Every cloaked object in the blocking cell's ground layer, of any house,
     starts uncloaking on the asking tick (S-0516 claim 11(e)). This changes the object's cloak
     state for every house, rather than granting sight only to the walker's house. Bridge-deck
     occupants are not affected, even when the blocker was found on deck. S-0516 claim 7 owns
     the uncloak progression. On the first ask the walker also searches a new path on that tick,
     with the ground-layer objects already uncloaking (claim 7). On the repeated ask it waits
     for the rest of that tick. Its asks on later ticks are first reactions again (claim 7), and
     each cloaked-blocker answer causes the same ground-layer uncloaking.
   - Backing: `measurable` (scenarios C3 and C8; scenario C9 for the first listed object; scenario
     C16, part c2, for a terrain object; scenario C25, part a, for the walker's view; scenario C19,
     parts b, d and d2, for the asking tick, global uncloaking, ownership and cell-layer scope)
   - Confidence: medium. High for ground-layer uncloaking of any house, and that a terrain object counts as not allied. Low for what happens to
     an allied wall, or to a terrain object, after it is taken as a target (the combat specs own
     that).

10. **Moving blocker: wait, then route around.**
    - On the first tick on which the next cell gives *moving blocker*, the walker stops and starts a
      wait of the blockage delay.
    - It keeps waiting while the path retry timer runs. Each time that timer has run out, it searches
      a new path. That includes the first such tick: when the timer has already run out then, the
      first search happens on it. Before the blockage delay has run out, the search does not avoid
      the blocking units; it counts them at a small extra cost (S-0212 claims 8 and 22), so a detour
      only slightly longer than the blocked route can be taken at once. Once the blockage delay has
      also run out, the search avoids them.
    - A new path that leads on resumes walking.
    - The blockage delay starts afresh only after the walker has taken a walking step again. A walker
      that alternates between short waits and steps never reaches the end of the delay unless one
      single wait lasts the whole delay.
    - A walker whose destination turns out to be unreachable gives up its move (S-0212).
    - Backing: `measurable` (scenario C5, parts a and b; scenario C2, parts c and d)
    - Confidence: medium

### Requests to make room

11. **Which occupants obey a cell-wide scatter request.** A sender can ask every object in one cell
    layer to scatter, with a threat point (or none) and the two options of S-0208. Each object listed
    in the cell layer then receives that scatter request when at least one of these holds:
    - the request is move-anywhere,
    - at least one unit or building of elite rank is listed in that cell layer at the time of the
      request (the receiving object itself, or any other occupant; the sender counts only if it is
      listed there),
    - the game-wide player-scatter rule is on,
    - the object is a unit with the scatter veterancy ability,
    - the object belongs to a house whose skill level is at least the game-wide scatter threshold.

    Whether a unit that receives the request then moves, and where, is S-0208 (claims 11 to 13)
    for infantry and S-0207 claim 16 for vehicles.

    Skill levels:
    - A human-controlled house has skill level 0. So for units of a human player the last condition
      holds only when the threshold is 0 or less.
    - In a skirmish or network game of the original, the skill level of the house of every computer
      player is the game-wide maximum, given when the houses are set up. So the last condition holds
      for every unit of a computer player when the threshold is at most that maximum. This is the
      original's computer behaviour: units of a computer player make room when a friendly walker or
      a crushing vehicle asks, where a human player's units of the same rank stay.
    - The neutral and special houses (Terms) are computer-controlled, but they are not computer
      players: in the original they keep skill level 0, the level every house starts with, since
      the map's per-house definitions are not read in a skirmish (S-0009, claim 26). So for their
      units the last condition holds only when the threshold is 0 or less, as for a human player's.
      Our engine keeps this (the project lead's decision of 2026-10-10, Terms).
    - In our engine an AI house has skill level 0, as a human-controlled house does (glossary *AI
      house* and *house skill level*), so the last condition treats its units as a human player's.
      This follows from the project lead's decision recorded in the glossary entry *AI house*: a
      rule of our engine that holds in every spec, not a quirk of this spec, so this spec defines no
      switch for it. Our AI may choose to reproduce the original's computer behaviour by ordering
      its units out of the way.

    Every object listed in the cell layer when the request is sent is asked once, on that tick, and
    the conditions above are judged then. What one occupant does with its request does not change
    which of the others are asked.
    - Backing: `measurable` (scenario C4, parts a and c, and scenario C19, part c, for move-anywhere
      requests reaching every occupant on one tick; scenario C7 for the other conditions and the
      skill levels of the original, part b3 for two occupants of one request, parts c and c2 for the
      neutral and special houses). The skill level of an AI house is a rule of our engine, not a
      claim about the original, and needs no backing.
    - Confidence: medium (high for the skill levels given to houses at setup in the original)

12. **Who sends them.**
    - A walker sends one under claim 8: move-anywhere, forced, no threat point.
    - A crushing vehicle sends one ahead of itself (S-0210 claim 7): forced, not move-anywhere, no
      threat point.
    - A driving vehicle whose next cell gives *stationary friendly* sends one: move-anywhere,
      forced, no threat point (S-0207 claim 15). Whether a vehicle that receives a request moves,
      and where, is S-0207 claim 16.
    - A vehicle factory clearing its exit apron sends forced, move-anywhere requests on a blocked
      apron test: to the apron cell, then its occupied neighbours (S-0417 claim 4). This happens
      during ground-vehicle unloading and during idle apron clearing, including an idle naval
      vehicle factory, under S-0417 claim 13's behaviour and eligibility conditions.
      These requests explicitly select the ground cell layer defined in Terms, not the bridge-deck
      layer or the layer chosen by an entry-outcome reference height. The apron request has no
      threat point; neighbour requests use the apron's centre. Claim 11's move-anywhere condition
      admits every listed object, with acceptance still governed by its movement spec. Within
      each cell, draw assignments follow claim 13's listing order; S-0417 owns the ordering
      between the apron, neighbours and the unloading or idle-behaviour delay.
    - Other senders belong to the specs of those events.
    - Backing: `measurable` (scenarios C4 and C7) for the walker's and the crushing vehicle's
      requests. The driving vehicle's request and the other senders are pointers: S-0207 and the
      specs of those events own them and their backing.
    - Confidence: high for the walker's and the crushing vehicle's requests. Low for the list being
      complete for movement.

### The order in which a cell lists its objects

13. **Listing order.** Each cell layer lists its objects in this order:
    - first the objects that are not buildings, the one that entered the cell layer most recently
      first,
    - then the buildings, in the order in which they were placed.

    The order changes only when an object enters or leaves.

    An on-map ground-layer building is listed in every actual foundation cell defined by
    S-0406 claim 2, not just the cell containing its position or reference point. A missing
    cell of an irregular foundation does not list the building. Thus a strike on any listed
    foundation cell can select that building as its strike witness under S-0513 claim 18.
    This does not change the listing order above.

    An object counts as entering when it is placed there (including by the map at game start) or
    when its position moves into the cell layer. A walker that has chosen its walking target in a
    cell has not entered it yet: it enters on the tick on which its position moves into the cell,
    although its reservation there counts from the asking tick (claim 1).

    For an aircraft using ordinary flight, listing uses its signed integer height in leptons:

    > listing height = world height − terrain surface height − deck allowance.

    The terrain surface is at its horizontal position (S-0106 claim 2), not the projected cell
    beneath its drawn image. The deck allowance is 416 leptons when the aircraft's own deck
    mark is set, and 0 otherwise. The subtraction is exact integer arithmetic, with no rounding.
    S-0501 claim 43 owns that mark's landing, climbing and retention rules; neither a render
    layer nor an occupancy flag substitutes for it.
    A positive listing height excludes the aircraft from both cell-layer lists. At listing
    height 0 or below it is listed on deck when deck-marked, and on ground otherwise. Thus an
    ordinary aircraft landed on a high bridge can be listed on deck at height 0 above that
    deck, rather than on ground. Taking off removes its listing when the listing height
    becomes positive, and landing adds it again. A rocket missile using
    rocket-flight movement is instead absent from both cell-layer lists in every phase,
    including pause, tilt and raise at its launch point; crossing cells never adds it to
    those lists (S-0506 claim 23). This exception has high confidence and `measurable`
    backing (S-0506 scenario R17). Units with drop-pod movement and tunnelling units while
    digging are likewise listed in no cell layer (S-0102 claim 3). So a superweapon warp, which
    takes only objects listed in a cell and not in the air (S-0513 claims 4 and 34), never takes
    either kind, as it never takes a rocket missile. This exception has medium confidence and
    `unmeasured` backing: no reproducible skirmish setup puts either kind in a warp's source
    block at its launch. A render layer is not a cell layer.
    Consequently an aircraft passing over an otherwise empty foundation cell does not block
    building placement through S-0406 claim 3 (b)'s aircraft-list test; a landed aircraft does.

    Jumpjets do not use the ordinary aircraft's positive-listing-height exclusion.
    S-0502 claim 4 owns their separate flight-state rule: grounded, ascending,
    descending, crashing and crashed physics removes the previous listing and permits
    listing at the new position only when its evaluated render layer is ground
    (S-0102 claim 3). Its on-deck state chooses deck listing; otherwise listing is
    ground. Let *r* be its signed integer height above the surface in leptons:
    above terrain unless its on-deck state is set, then above the deck. Let
    *r*′ = *r* − 416 leptons only over a high-bridge cell, with on-deck state unset,
    *r* ≥ 416 leptons and no bridge/parachute falling state; otherwise *r*′ = *r*.
    New listing qualifies exactly when *r* < 208 leptons or *r*′ = 0 leptons.
    The 208-lepton comparison uses *r*, not *r*′; all arithmetic is exact integer
    arithmetic without rounding. The type's cruise height does not change this test.
    Without on-deck state, a non-falling jumpjet at 416 leptons above terrain over a
    high bridge qualifies, but at 417 or 500 leptons it does not, despite clearance
    above the deck being below 208 leptons. Starting a jumpjet crash does not itself
    set the independent bridge/parachute falling state. Actual-position occupancy
    marks still follow claim 1's kind-specific rules; they are not landing reservations
    or hover slots. Hovering and cruising create no new listing and do not transfer
    an existing listing on a cell crossing.

    S-0502 claim 22 owns the clearance consequence. Its look-ahead reads actual
    ground-layer listings, not reservations or hover slots, with no self-exclusion:
    a listed jumpjet supplies the 85-lepton allowance even to itself. Deck-only
    listing supplies none. Ordinary high hover has no listing left from ascent and
    no automatic self allowance. A low own-cell ascent-to-cruise transition can
    instead retain its previous ground listing until removal, even when subsequent
    flight crosses into another cell. That retained listing counts wherever
    look-ahead uses it; direct-reference intervals do not use the allowance.
    These pointers have high confidence for placement eligibility and the clearance
    predicate, medium confidence for exceptional retained-listing history, and
    `measurable` backing (S-0502 scenarios J3 and J18) for ordinary placement and
    clearance. Retained low-flight listing persistence remains `unmeasured` pending
    J18's diagnostic, because its history has not been isolated from reservations
    in an observation.

    This order is observable in four ways: which object a blocked walker attacks (claim 9), which
    occupant decides when an override meets a *blocked* (claim 4), the order of the random draws
    of a cell-wide request (State and ordering), and which object wins a tie for smallest distance
    in S-0207 claim 15's nearest-object choice (F8): the one listed first wins.
    - Backing: `measurable` (scenario C9 for the front and end rule and the map's order; scenario C18
      for the override case; scenario C26, parts b and c2, for the moment a walker enters,
      S-0207 scenario D17 (d), runs 3a and 3b, for the nearest-object tie; scenario C29 for aircraft
      listing, with S-0501 scenario AC27 for deck-marked listing; S-0513 scenario SW7(f) for per-foundation-cell building listing and its strike-witness
      consequence). `unmeasured` for order among multiple buildings in one cell layer: no valid setup
      establishing overlapping buildings and distinguishing their order is known here.
      `unmeasured` for drop-pod and digging tunnelling units listed in no cell layer, for the
      reason given above.
    - Confidence: high for the front and end rule and for the moment a walking unit counts as
      entering, for the nearest-object tie and for per-foundation-cell building listing.
      Medium for the order among objects placed by the
      map, for aircraft listing and for drop-pod and digging tunnelling units listed in no cell
      layer.

### Entry outcomes for an aircraft

14. **An aircraft gets clear everywhere in skirmish.** In a skirmish
    an aircraft's entry outcome for every cell is *clear*, whatever the cell's terrain,
    overlays and occupants. No other value arises. In a campaign game a cell can also give *blocked*
    to some aircraft when the cell is under shroud (low; Open questions).
    - Specs that read "the actor's entry outcome" for an aircraft (S-0213 claim 7, S-0214 row GC9) therefore always see *clear* for it in this batch's game modes.
    - Backing: `measurable` (scenario C21, part a) for skirmish. Network entry rules are deferred.
      `unmeasured` for the campaign case: campaign games are outside this batch (the project lead's decision of
      2026-10-10; Open questions), as for row VO1 of claim 5 (U7).
    - Confidence: high for skirmish; low for the campaign case

### Ordinary walkers changing surface

15. **Listing layer and reservation surface are distinct (Q-0560).** Choosing a walking target
    does not change an ordinary walker's own cell layer. Whenever its horizontal position crosses
    into another cell, its listing belongs to that newly entered cell on that tick, with the layer
    given below. Let *A* be the cell it leaves and *B* the cell its position enters, with integer
    terrain height levels *a* and *b*. These rules apply to every actual crossing, including a
    brief crossing of a third cell on the straight line to a diagonal walking target. The target's
    cell and target's surface do not override them.

    | Cell crossed into | Height relation | Layer after crossing |
    |---|---|---|
    | *B* carries a high bridge | *b* = *a* − 4 height levels | deck |
    | *B* carries a high bridge | any other relation | the layer the walker had before crossing |
    | *B* carries no high bridge, *A* carries one | any relation | ground |
    | neither cell carries a high bridge | any relation | unchanged (normally ground) |

    Thus stepping from ground at deck level onto a high bridge makes the walker a deck object
    on the first tick its position lies in that bridge cell. Stepping from deck onto a bank makes
    it a ground object on the first tick its position lies in the bank cell. Neither transition
    waits for arrival on the standing position. Its next ordinary ask uses the resulting layer
    for the reference level of Terms. A deck walker crossing a third high-bridge cell stays on
    deck; crossing a third cell without a high bridge puts it on ground. From ground, a third
    high-bridge cell gives deck membership only on the exact four-level drop in the table.

    **Surface reserved on the asking tick.** Let *z* be the walker's current integer world height
    in leptons, and *F* the terrain surface height at its desired point in the asked cell
    (S-0208 claim 3). The standing-position choice uses the deck record precisely when that cell
    carries a high bridge and *z* > *F* + 312 leptons; otherwise it uses ground. This is an exact
    integer comparison, strict at equality. The chosen standing position has height *H* + 416
    leptons in deck mode and *H* in ground mode, where *H* is the terrain surface height at that
    chosen position (S-0208 claim 1). Its reservation and infantry house are marked on that
    chosen surface immediately. The old mark is cleared using the former walking target's
    position and height, or the walker's actual position and height if there was no target,
    not using the newly selected surface or the walker's new listing layer. Crossing cells does
    not transfer this reservation. The ground record's object-present flag remains the separate
    refusal test of S-0208 claim 3 even in deck mode.

    The entry-outcome record of Terms is therefore not necessarily the standing-position record.
    For flat cells under a deck, a walker at *z* = *F* + 416 chooses and reserves deck. For a
    walker standing on deck over flat ground, with the next bridge cell's ground exactly one
    height level higher, the entry check reads ground and the standing-position choice also uses
    ground: the height difference is exactly 312 leptons. With the next ground one level lower,
    the entry check still reads ground but the position choice uses and reserves deck: the
    difference is 520 leptons. In both cases crossing directly between these bridge cells keeps
    deck listing membership. These conditional results do not establish that a valid bridge
    can span such ground. S-0212 claim 10 still decides whether a requested path step is legal.
    S-0207 claim 5 already gives the equivalent boundary-time rule for driving vehicles, not for
    walkers; S-0208 claims 1 and 3 already give target heights and reservations, but their surface
    choice must use the height test here rather than the walker's current surface alone.
    - Backing: `measurable` (scenario C22, parts e and f, for boundary-time layer changes and
      reservations on the established flat-ground routes). `unmeasured` for the exact
      312-lepton target-surface boundary, including equality: those routes do not isolate that
      boundary, and no ordinary-walking fixture that does is established here. Also `unmeasured`
      for the uneven-ground conditional examples: no valid bridge layout with that ground has
      been established here.
    - Confidence: high for the boundary-time transition and the strict target-surface comparison,
      medium for their conditional uneven-ground consequences.

## Formulas

**F1. Distance to the destination (claim 8).**
- *D* = trunc(√̃(Δ*x*² + Δ*y*² + Δ*z*²)), where Δ*x*, Δ*y*, Δ*z* are the differences between the
  walker's position and its locomotor destination (S-0208 Terms; under a move, the position of its
  destination), all in leptons of world coordinates, heights included. √̃ is the table square root *r* of S-0204 (Formulas F1): the sum
  truncated to 14 significant binary digits, its exact root truncated to 24 significant binary
  digits. S-0208 F8 lists the properties of √̃ that this formula relies on.
- Number domain: the differences Δ*x*, Δ*y*, Δ*z* are integers. Their squares and the sum are
  formed in floating point, at a precision of at least double precision, and the sum is held in
  double precision. Double precision represents every integer below 2⁵³ exactly, which covers every
  map distance, so the sum equals the integer sum. √̃ is applied to that sum, and S-0204 F1 already
  includes the truncations it makes. Its result becomes an integer by a conversion of S-0205 (claim
  7), which truncates toward zero.
- The walker gives up when *D* is less than the close-enough distance (strict).
- Because √̃ can be one below the exact root above 129 leptons, a walker whose exact distance equals
  the close-enough distance, or exceeds it by less than one lepton, can still give up. S-0204 F1
  decides each case exactly. With S-0204's switch `quirks.table_root_distance` off, √̃ is the exact
  root and *D* = ⌊√(Δ*x*² + Δ*y*² + Δ*z*²)⌋; this spec defines no switch of its own for it.
- Unit: leptons.

**F2. Close-enough distance from the data.** The close-enough distance is trunc(256 × *c*) leptons,
for an imported real *c* in cells. *c* is the decimal value form of S-0003: a single-precision value
(its claim 6), or, when written with `%`, the double-precision result of its claim 7. The product
256 × *c* is formed in double precision. Since 256 is a power of two it is exact, so the only
rounding is the conversion of S-0205 (claim 7) that turns it into an integer, truncating toward
zero. Only a *c* that
equals −1 as parsed counts as the "not given" value (Import mapping). Any other value is used, even
one whose product truncates to −256 leptons, such as −1.002. The blockage
delay is an imported integer in ticks, used as read.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Side positions that make a cell full | 3 | standing positions | claim 2 |
| Allied infantry standing still that give *stationary friendly* | exactly 3 | units | claim 4 |
| World height difference allowed for giving up | less than 208 (2 height levels of 104) | leptons | claim 8 |
| Entry outcomes | 8 | values | Terms |
| Parts of a side-position pattern for the right, left and bottom positions | 1, 2, 4 | pattern value | Terms, claim 1 |
| Added to a mover's own height level while it is on a bridge deck | 4 | height levels | Terms *cell layer* |
| Difference from the asked cell's height level above which the deck layer's objects are listed | more than 1 | height levels | Terms *cell layer* |
| Height above the asked cell's height level at which the deck layer's record is read | exactly 4 | height levels | Terms *cell layer* |

## State and ordering

| Walker state | Event (next cell's outcome) | Result |
|---|---|---|
| walking, path step ahead | clear, position free | walking target chosen on the asking tick; the first step towards it follows as S-0208 claim 6 says |
| walking | clear, no position free | stops this tick, asks again next tick |
| walking, or waiting after a repeated ask on an earlier tick | first ask: stationary friendly, friendly obstacle, enemy obstacle, blocked | stops, discards the path, searches a new one on the asking tick whatever the retry timer shows, and makes the repeated ask on that tick too |
| holding a path searched on this asking tick | repeated ask: clear, position free | walking target chosen in the first cell of the new path on the same asking tick (claim 7), so it stands still exactly as long as at a free cell boundary |
| holding a path searched on this asking tick | repeated ask: stationary friendly | gives up within the close-enough distance (claim 8), otherwise sends a cell-wide request and waits for the rest of the tick; on the next tick it asks again as a first reaction |
| holding a path searched on this asking tick | repeated ask: friendly obstacle, enemy obstacle | stops and discards the path. On the same tick it attacks the first listed object when the walker's house is not allied with that object (a terrain object included), takes the wall as its target when no object is listed, and attacks nothing when the walker's house is allied with the first listed object. Each take is an override to *attack* with no destination (claim 9) |
| holding a path searched on this asking tick | repeated ask: blocked | keeps the path and waits for the rest of the tick; on the next tick it asks again as a first reaction |
| walking | closed friendly gate | opens the gate, waits |
| walking, or waiting after a repeated ask on an earlier tick | cloaked blocker (first or repeated ask) | every cloaked ground-layer occupant of the blocking cell, of any house, starts uncloaking for every house; deck occupants are unaffected (S-0516 claim 11(e)). On the first ask: it searches a new path on the asking tick with those ground-layer occupants already uncloaking (claim 7). On the repeated ask: it waits for the rest of the tick; on the next tick it asks again as a first reaction |
| walking | moving blocker | waits; starts the blockage delay on the first such tick; re-paths whenever the retry timer allows, avoiding blockers once the delay has passed |

Ordering stated as consequences:
- The walker reacts to the outcome on its asking tick (Terms). Objects take their turns of a tick
  in the update order of S-0203 (claim 3, the order of entering play). A position or cell that
  another unit vacated in its own turn earlier in the same tick counts as vacated; one that a unit
  later in the update order vacates in that tick still counts as taken.
- A position counts as taken from the tick on which a walker chose it as its walking target. Its
  previous position counts as free from that same tick (claim 1). A later occupant-aware choice
  therefore cannot take that marked position while the mark remains. This does not cover
  choices that ignore occupants (claim 3), or promise that a mark cannot be cleared by another
  unit's clearing operation (claim 1).
- A cell-wide request reaches every occupant on the tick it is sent. Each occupant's random draw
  (S-0208 claim 12 or 13) happens then, in listing order (claim 13).

## Edge cases and quirks

- **The infantry house can be forgotten.** The last infantry on a side position may leave a cell in
  which another infantry unit still stands at the centre or top position. The infantry house is then
  set to none (claim 1), and enemy walkers no longer see that unit as enemy infantry for claim 4. The
  switch `quirks.occupancy_house_side_positions_only` (default: the original) selects this. With it
  off, the infantry house is cleared only when no standing position is taken. Scenario C12 (a)
  distinguishes the two.
- **The infantry house can outlast its infantry.** Clearing an infantry mark leaves the house as
  it is whenever a side position stays taken (claim 1). Clearing a terrain object's marks leaves
  it as it is regardless of the remaining side positions. The house is that of the last infantry unit to
  mark, even when that unit has left and only units of another house, or only a terrain object's
  marks, remain. Two cases follow:
  - When an allied unit marks after an enemy one and then leaves, the enemy unit that stays is no
    longer seen through the infantry house, so an armed walker walks into its cell (an unarmed walker
    is still blocked by the enemy unit's own row in claim 4). When an enemy unit marks after an
    allied one and then leaves, the cell gives an armed walker *enemy obstacle* and an unarmed one
    *blocked*, although only allied infantry stands there.
  - A cell whose terrain object marks a side position keeps the house of the last infantry unit that
    stood there, for as long as the terrain object stands, and after it is removed until the next
    infantry mark there is cleared (claim 1). Such a cell, empty but for the terrain
    object, gives an unarmed walker *blocked* and an armed one *enemy obstacle* when an enemy was the
    last infantry to stand there; an armed walker then takes the terrain object as its target
    (claim 9). For a vehicle the terrain object gives a value of its own whatever the house (claim
    5, rows VP26 and VP29; a train without a primary weapon aside, Open questions).

  The switch `quirks.occupancy_house_stale` (default: the original) selects this. With it off,
  clearing an infantry mark, when the house-clearing rule would leave the house in force, instead
  gives the house of the infantry unit that marked most recently among those still holding a
  position in that cell layer, or none when no infantry unit holds one. Clearing a terrain object's
  marks also recomputes the house in that way, regardless of the remaining side positions.
  The switch combines freely with `quirks.occupancy_house_side_positions_only`, which decides
  when clearing an infantry mark sets the house to none without that recomputation; it does not
  itself change terrain-object removal. Scenarios C12 (e), C16 (c), (c2) and (c3) distinguish
  the two settings.
- **Four still friends count as moving.** Row WR4 of claim 4 asks for exactly three allied infantry
  without movement in progress. When a fourth one stands still at the centre or top position of a
  full cell, the count is four and the cell gives *moving blocker* (row WR5): the walker sends no
  request, waits at least the blockage delay and then routes around the cell, although every unit
  in it stands still. The switch `quirks.stationary_friendly_exactly_three` (default: the original)
  selects this. With it off, row WR4 asks for three or more allied infantry without movement in
  progress, so the cell gives *stationary friendly*: the walker gives up within the close-enough
  distance, and otherwise sends the cell-wide request of claim 8, which reaches all four units.
  The switch changes the entry outcome wherever it is read, in the path costs of S-0212 (claim 8)
  too. Fewer than three still units (for example when a terrain object marks a side position) give
  *moving blocker* in both settings. Scenario C4 (g) distinguishes the two.
- **Giving up near a stationary friend.** A walker within the close-enough distance of its
  destination stops for good when the next cell is full of stationary friendly units (claim 8). So it
  may stop short of the destination cell. This is deliberate behaviour, not a quirk.
- **An override can hide a blocker listed after it.** When an occupant overrides, for example the
  walker's mission target when it is a unit, occupants listed after it in the cell give no value
  (claim 4). An occupant that would make the cell *blocked* still lets the walker in when it is
  listed after the mission target: a building always is, and another unit is when it entered the
  cell before the mission target. This is the original behaviour, and it can only happen in cells
  that hold several objects. The switch `quirks.occupancy_override_hides_later_blockers` (default:
  the original) selects it. With it off, any occupant that gives *blocked* decides the outcome,
  whatever its place in the list. Scenario C18 distinguishes the two.
- **A crusher ignores obstacles in cells with overlay index 0.** When a vehicle whose type has
  the crusher flag begins a step (claim 5), it reads *friendly obstacle* and *enemy obstacle* as
  *clear* whenever the cell's overlay is of the overlay type with registration index 0 (S-0004),
  whatever gave the obstacle value: an allied wall section of that type that it would otherwise
  stop at, or an enemy object standing in such a cell. The rule depends on which overlay type the
  data registers first, not on any property of that type. The switch
  `quirks.crusher_overlay_index0_clear` (default: the original) selects it. With it off, the vehicle
  reads the entry outcome unchanged. Scenario C20 (f) distinguishes the two. The rule for trains in
  claim 5 is not a quirk: it matches their path costs (S-0212 claim 8).
- **A crusher on a bridge deck looks at the ground layer.** Row VR2 of claim 5, for a crushable
  occupant with the vehicle-present flag set, asks whether the first vehicle listed
  in the cell's ground layer is crushable, even when the deck layer counts for the vehicle. On a
  deck, a crusher facing a crushable vehicle that stands on the deck therefore gets *moving blocker*
  when nothing stands under the bridge there (where on the ground it would get *clear*), and *clear*
  when a vehicle it can crush stands under the bridge. This matches the ground-layer rule of S-0210
  (`quirks.crush_slowdown_ground_layer`). The switch `quirks.crusher_entry_ground_layer` (default:
  the original) selects it. With it off, the row looks at the first vehicle listed in the cell
  layer that counts. Scenario C22 (c) distinguishes the two.
- **`quirks.teleport_vehicle_ground_reservation`** (shared with S-0207 claim 20 a and e).
  Default original: reserve ground for a vehicle's high-bridge teleport landing and keep that
  reservation through deck placement, which also marks deck. Replacement clears the remembered
  ground reservation; departure clears the actual deck mark. Removal clears only actual
  occupancy, retaining a separate remembered or pending ground reservation (S-0207 claim 20 k).
  Temporary driving clears old deck on departure but retains old ground; restoration relocates
  no mark (S-0207 claim 21). Off: reserve and clear the eventual landing layer (deck on a high
  bridge, ground elsewhere), with no separate ground reservation beneath a deck landing.
  Removal clears both pending reservation and actual occupancy, leaving no orphan reservation.
  The default removal behaviour is `measurable` through S-0207 scenario D22 run 15,
  with medium confidence. S-0207 owns the lifecycle and the alternative; claims 1 and 5 here
  own their occupancy and entry-outcome consequences.
- **`quirks.teleport_infantry_ground_reservation`** (shared with S-0208 claim 19).
  Default original: a ground standing-position reservation survives deck landing,
  which adds no deck standing-position mark. A later destination choice clears the
  actual deck position, potentially clearing another infantry's mark while leaving
  ground taken. Off: at the jump transfer the reservation to the actual landing
  surface at the same standing position, releasing ground and marking deck for a
  bridge landing; later clearing uses that deck entry. S-0208 owns the lifecycle
  and alternative, with `measurable` backing through its W11 part i and medium
  confidence. This is separate from the vehicle and aircraft switches because
  infantry placement does not add their placement-time marks.
- **`quirks.teleport_aircraft_ground_reservation`** (shared with S-0207 claim 20 a and e).
  Default original: reserve ground at terrain height, retain it through deck placement,
  clear the remembered ground reservation on the next accepted destination, and clear
  only actual occupancy on removal, not a separate remembered or pending reservation
  (S-0207 claim 20 k). Destruction on deck can therefore leave the old ground
  object-present flag or a replacement landing cell's ground flag. Off: reserve only the
  eventual landing layer, clear that reservation on replacement, and clear both pending
  reservation and actual occupancy on removal, leaving no orphan reservation.
  The default removal behaviour is `measurable` through S-0207 scenario D22 runs 14 and 15,
  with medium confidence. S-0207 owns this lifecycle and alternative.
  Claims 1, 3, 5 and 6 here own occupancy, entry and standing-position consequences.
- **An armed walker takes an occupant it cannot fire at (Q-2396).** Rows WP13, WP24 and WP28 of
  claim 4 ask no firing check, so, where one of these rows applies, an armed walker meets an
  enemy building, a closed enemy gate or
  another enemy object that its weapon cannot fire at as *enemy obstacle*: its path may lead
  through that cell, and it takes the occupant as its target (claim 9). The drop of S-0309 claim
  29 then leaves a walker under a plain move, to which no row of S-0215 claims 13 and 14 gives a
  destination or an order at its fall-back, resting in front of the occupant without its move,
  and keeps a walker under an attack move in the take-and-drop cycle of claim 9 for as long as it
  is engaged at each fall-back. A walker with explosive charges that takes a building accepting
  charges goes to *sabotage* instead when that behaviour is current in its turn of the next check
  tick, and otherwise the check drops the building as for any other walker (claim 9, charges
  case). The
  switch `quirks.walker_attacks_unhittable_obstacle` (default: the original) selects this. With
  it off, each of these three rows gives *blocked* instead of *enemy obstacle* when the firing
  check without the range test, for the weapon the walker would use against the occupant
  (S-0308), answers *illegal*, as for an unarmed walker (row WP22). That holds for a walker with
  charges too, which then never reaches *sabotage* this way. The path search then does not
  consider that step (S-0212 claim 8), and the walker routes round the occupant or reacts to
  *blocked* as claim 7 gives. Scenario C15, parts e1 to e3, e6 and e7, distinguishes the two.
- **Rounding.** The rounding mode is S-0205's (claim 5, switch `quirks.chop_rounding`), and this
  spec defines no switch of its own for it. The conversions to integers in F1 and F2 truncate toward
  zero in either setting (S-0205 claim 7). The switch reaches F1 only through √̃, for sums of
  2²⁴ square leptons or more (S-0204 F1), and F2 only for a value written with `%`, whose fraction
  S-0003 (claim 7) forms in the rounding mode.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| No additional rules or art key | ordinary walking layer transitions and target-surface comparison (claim 15) | fixed engine rules; terrain height levels and high-bridge presence come from the map, as S-0009 claim 7 and S-0211 claim 15 give them | not configurable by a missing key |
| Rules, `[General]`, `CloseEnough` | close-enough distance (F2) | real, in cells. The value −1 is this key's "not given" value (S-0006 claim 3): written explicitly, it keeps the value in force, as if absent | keeps the value in force; before any rules layer gives it, 640 leptons (2.5 cells) |
| Rules, `[AI]`, `BlockagePathDelay` | blockage delay | integer, ticks | keeps the value in force; before any rules layer gives it, 60 ticks |
| Rules, the combat-damage section (as S-0210's Import mapping names it), `PlayerScatter` | player-scatter rule (claim 11) | boolean | keeps the value in force; before any rules layer gives it, no |
| Rules, `[IQ]`, `Scatter` | scatter threshold of the house skill level (claim 11) | integer, skill levels | keeps the value in force; before any rules layer gives it, 3 skill levels |
| Rules, `[IQ]`, `MaxIQLevels` | the maximum house skill level, which the original gives to the house of every computer player in a skirmish or network game (not to the neutral and special houses, which keep 0); an AI house keeps 0 by the rule of glossary *AI house* (claim 11) | integer, skill levels | keeps the value in force; before any rules layer gives it, 5 skill levels |
| Rules, overlay sections, `Crate`, `Wall` | crate overlay, wall overlay (claims 4 and 5) | boolean | no |
| Rules, overlay sections, `Armor` | the overlay type's armour, of which only "wood" matters here (claim 5, row VO5) | armour name | as the combat specs give it |
| Rules, building type sections, `Gate` | gate (claim 4) | boolean | no |
| Rules, building type sections, `BridgeRepairHut` | bridge repair hut (claim 4) | boolean | no |
| Rules, building type sections, `InvisibleInGame` | building type that is invisible in play (claim 4) | boolean | no |
| Rules, building type sections, the laser-fence and firestorm-wall flags | as in S-0211's Import mapping | boolean | no |
| Rules, infantry type sections, `VehicleThief` | vehicle-thief type (claim 4, row WP6) | boolean | no |
| Rules, terrain-object type sections, `TemperateOccupationBits`, `SnowOccupationBits` | side-position pattern (Terms, claim 1) | integer, pattern value | keeps the value in force; before any rules layer gives it, 7 (pattern value: all three side positions) |
| Rules, unit type sections, `VeteranAbilities` / `EliteAbilities` containing `SCATTER` | scatter veterancy ability (Terms, claim 11) | list | none |
| Rules, building type sections, `UnitAbsorb`, `Grinding`, `UnitRepair`, `Bunker`, `Bib` | building that absorbs units, grinds units, repairs vehicles, is a bunker, has a bib (claim 5) | boolean | no |
| Rules, building type sections, `NumberImpassableRows` | impassable-rows value of the blocking cells (Terms, claim 5) | integer, cells counted along x from the top-left cell (Terms *blocking cell*) | keeps the value in force; before any rules layer gives it, none (every foundation cell is a blocking cell) |
| Rules, vehicle type sections, `SmallVisceroid` | small-visceroid flag (claim 5) | boolean | no |
| Rules, vehicle type sections, `Crusher` | the crusher flag of claim 5's locomotor rule (S-0210 owns the rest) | boolean | no |
| Rules, projectile sections, `AG` | the primary weapon's projectile can hit ground targets (claim 5) | boolean | as the combat specs give it |
| Rules, warhead sections, `Wall`, `Wood` | warhead can damage walls, can damage wood (claims 4 and 5) | boolean | as the combat specs give it |
| Rules, infantry type sections, `Primary`, `Secondary`, `ElitePrimary`, `EliteSecondary` | the weapons the armed test of a walker reads (Terms *armed*); the primary weapon of the wall and closed-gate rows of claim 4 (Terms *armed*, *has a primary weapon*) | weapon name | as the combat specs give it |
| Rules, vehicle type sections, `Primary`, `ElitePrimary` | the vehicle's primary weapon (Terms *armed*, *has a primary weapon*): the armed test and rows VO4 to VO6, VP10, VP11, VR9 and VR10 of claim 5. Its warhead and projectile are read through the warhead and projectile rows below | weapon name | as the combat specs give it |
| Rules, weapon sections, `Damage`, `AmbientDamage` | damage field and ambient-damage field of the armed test (Terms *armed*) | integer | as the combat specs give it |

## Measurement scenarios

All run under the *measurement setup*, game speed index 2, no engine add-on and no mods (the named test files of a scenario excepted),
on purpose-made flat maps of clear terrain, with one human player and one
bystander computer player whose start position is far away, unless stated. The observed area is
the corridor, bridge or other test site and its approach cells. Void a run if an unnamed bystander
object enters that area or acts on a test object, or the bystander is defeated before observation
ends. Named computer-owned test units are pre-placed by the map; parts requiring them idle count
only while the recorder confirms they stay idle, except for their specified trigger moves. A *recorder* reads, on
every tick, the tick number and each unit's world position, target and destination, and, where a
scenario says so, a unit's contact and behaviour (as in S-0213's scenarios).
Targeting is shown by a presentation-only target line. A *test file* is a per-scenario rules change
documented with the scenario. The search of S-0212 never leads a path through a cell whose entry
outcome is *blocked* when the path is computed (its claim 7), so a walker ordered towards such a cell
may stop short of it, or not set out at all; the expectations below allow for that. Lengths of
detours count diagonal steps as single steps.
"Nearer than the close-enough distance" refers to the value the stock rules give, read from them when
the scenario is built.

**Scenario C1: capacity (claims 1, 2, 6).**
- *Setup:* one empty target cell. Five infantry of the human player of one stock type, pre-placed in
  a row west of it, one per cell, at the right standing position (S-0208 claim 5, map value 2).
- *Procedure:* order all five into the target cell at once.
- *Observable:* how many end inside the target cell, and their positions.
- *Runs:* 3.
- *Expected:* exactly three, at the right, left and bottom positions. The other two stop in
  neighbouring cells: the walkers give up within the close-enough distance (claim 8).
- *Falsified if:* other than exactly three enter, or any entrant stands at the centre or top position.

**Scenario C2: vehicles and infantry (claims 1, 3, 4, 10).**
- *Setup and procedure:*
  - (a) An idle vehicle of the human player stands in a cell. An infantry of the human player stands
    nearer to it than the close-enough distance. The infantry is given a move order into the
    vehicle's cell.
  - (b) As (a), but the vehicle is a stock transport and the infantry is ordered to enter it.
  - (c) A computer-owned unarmed vehicle is moved by a map trigger across a one-cell-wide corridor.
    It crosses the corridor at a cell *X* and stops in *X*. An armed infantry of the human player
    walks along the corridor towards *X*, with a detour around *X* that is 8 cells longer than the
    route through *X*. That is long enough that a search before the blockage delay keeps the route
    through *X* (claim 10), and short enough that the walker takes it once *X* gives *enemy
    obstacle* (S-0212 claim 8). The start times
    are varied over 20 runs, so that in some runs the infantry reaches *X*'s edge while the vehicle is
    still driving into *X*.
  - (d) As (c), but the vehicle's route turns a quarter turn, so that it drives a two-cell curve
    (S-0207 claim 7) whose second cell is *Y*, and stops in *Y*. The infantry walks towards the first
    cell *X* of that curve along a corridor, with a detour around *X* as in (c). The start times are
    varied over 20 runs.
  - (e) As (b), with the human player's invulnerability superweapon (an iron curtain) fired on the
    transport just before the enter order is given, so that the transport is invulnerable while the
    infantry walks to it.
    A test file shortens the superweapon's charge time. The human player plays a house that has it.
- *Observable:*
  - (a) whether the infantry enters, and whether the vehicle moves.
  - (b) whether the infantry reaches the vehicle.
  - (c) per run: on the recorder, the tick *t*ₑ on which the vehicle's position enters *X*, and the
    tick *t*ᵥ on which it starts its move into *X*. Also the tick *t*ₛ on which the infantry stops at
    *X*'s edge, and the first tick on which it moves away from that edge.
  - (d) per run: the tick *t*ᵥ on which the vehicle starts the curve, the tick *t*ₑ on which its
    position enters *X*, and the tick *t*ₛ on which the infantry stops at *X*'s edge, and the first
    tick on which it moves away from that edge.
  - (e) the tick on which the infantry stops next to the transport's cell, the tick on which the
    transport stops being invulnerable, and the tick on which the infantry walks into its cell.
- *Runs:* (a), (b) and (e) 1 each, (c) and (d) 20 each.
- *Expected:*
  - (a) The infantry does not enter. It stops next to the cell and its move ends. The vehicle does
    not move: no scatter request is sent, because the walker gives up (claim 8).
  - (b) The infantry walks into the vehicle's cell and reaches it (row WP3 overrides, claim 4).
  - (e) The infantry does not walk into the transport's cell while the transport is invulnerable
    (*blocked*, claims 4 and 7). It stops short of the cell, or, if its path search finds no way in
    (S-0212 claim 7), does not set out. If it still holds its order when the effect ends, it walks in
    after that.
  - (c) In runs with *t*ᵥ ≤ *t*ₛ < *t*ₑ, the infantry is waiting on a *moving blocker* (claim 3), and
    the first tick on which its position changes after *t*ₛ is later than *t*ₑ. Once the vehicle is
    listed in *X*, the infantry searches a new path (*enemy obstacle*, claim 7) and takes the detour.
    In runs with *t*ₛ ≥ *t*ₑ it does that from *t*ₛ on.
  - (d) The same as (c): in runs with *t*ᵥ ≤ *t*ₛ < *t*ₑ, the first tick on which the infantry's
    position changes after *t*ₛ is later than *t*ₑ, because the vehicle set the flag of *X* when it
    started the curve towards *Y*.
- *Falsified if:*
  - (a) the infantry enters the occupied cell, or the vehicle moves away,
  - (b) the infantry stops next to the cell,
  - (c) or (d) in a run with *t*ᵥ ≤ *t*ₛ < *t*ₑ, the infantry's position changes on a tick after *t*ₛ
    and at or before *t*ₑ. If no run of (d) falls in that window, (d) is repeated with other start
    times; it is not counted as a result,
  - (e) the infantry walks into the transport's cell while the transport is invulnerable. If the
    enter order is refused for an invulnerable transport (S-0214), or the infantry drops its order,
    the part cannot decide.

**Scenario C3: enemy occupants and obstacles (claims 4, 5, 7, 9; Terms *armed* and *cloaked*).** Each part uses a one-cell-wide
corridor with no other route unless stated. In the wall parts (c2) to (c6), a test file gives the
walker's type the infantry-destroyer movement zone (S-0211 claim 12), which may cross wall-like
cells; with the plain infantry zone no path leads through a wall at all (S-0212 claim 6), and no
stock infantry type has the destroyer zone.
- *Setup:*
  - (a) An armed infantry of the human player, and one enemy infantry standing in a corridor cell.
  - (b) As (a), with an unarmed infantry type (for example an engineer).
  - (a2) As (a), with a test file that removes the walker type's primary weapon and gives it, as
    its secondary weapon, the weapon that was its primary. The walker is of rookie rank (the map's
    default), so the ordinary weapons count.
  - (a3) As (a), with a walker type that has a primary weapon and no secondary weapon, and a test
    file that sets that weapon's damage and ambient-damage fields to 0.
  - (a4) As (a), with a test file that sets the walker's primary weapon's damage field to 1 and its
    ambient-damage field to 0, and gives the type a secondary weapon whose damage and ambient-damage
    fields are 0.
  - (a5) As (a3), with a test file that sets the ambient-damage field of that weapon to 1 (its
    damage field stays 0).
  - (a6) As (a), with the walker made elite by the map, and a test file that gives its type an
    elite primary weapon whose damage and ambient-damage fields are 0, and no secondary or elite
    secondary weapon. Its ordinary primary weapon is left as it is.
  - (c2) An allied wall section closes the only route of an armed infantry of the human player, whose
    primary weapon can damage walls.
  - (c3) As (c2), with a test file that removes the wall-damaging property from the warhead of the
    walker's primary weapon.
  - (c4) As (c2), with the wall section built by the human player and the walker owned by the
    computer player, moved along the corridor by a map trigger.
  - (c5) As (c2), with a bypass that leaves the corridor before the wall cell and rejoins it after
    it, 40 cells longer than the route through the wall cell (straight steps only). The walker
    starts outside the usable area, as the movers of scenario C20 do, so that its search runs
    without a corridor.
  - (c6) As (c4), with the bypass of (c5).
  - (d) A stock vehicle that can crush, and a stock vehicle that cannot, each ordered through a cell
    holding one enemy infantry.
  - (d2) A stock vehicle that can crush, ordered through a cell holding one idle enemy vehicle of a
    stock type it can crush (S-0210).
  - (e) An enemy spy, disguised as seen by the human player, stands in a corridor cell of an armed
    walker's route. The walker's destination is farther than the close-enough distance.
  - (f) A test file lets an enemy vehicle type cloak. One such vehicle stands idle and cloaked in a
    corridor cell of an unarmed walker's route.
  - (f2) As (f), with a test file that gives an unarmed stock unit type of the human player the
    sensor ability, with a sensor range that covers the vehicle's cell. One such unit stands idle
    within that range, away from the corridor, so that the cloaked vehicle is visible to the human
    player before the walker is given its order. The walker starts at least 5 cells from the
    vehicle's cell, and *P* is the corridor cell in front of that cell on the walker's side.
  - (g) A test file lets an enemy infantry type cloak. One such unit stands cloaked in a dead-end
    niche cell beside a corridor cell *X* of an unarmed walker's route. After the walker has been
    given its order (so that its path was computed while *X* was empty), a map trigger moves the
    unit into *X*, where it stands idle, cloaked again, before the walker arrives.
  - (h) An armed walker, and an enemy infantry of the type of (g) standing idle and cloaked in a
    corridor cell of its route.
- *Procedure:* give the move order and watch the first 300 ticks.
- *Observable:* whether the mover walks into the cell, stops, takes a target, attacks, goes around,
  or reveals the cloaked unit. Targeting is shown by a presentation-only target line.
- *Runs:* 1 each.
- *Expected:*
  - (a) The armed walker does not enter the cell. It stops in front of it and attacks the enemy
    infantry.
  - (b) The unarmed walker does not enter and does not attack (*blocked*). It stops short of the
    cell, or does not set out, because no path leads through it.
  - (a2) As (a): the walker is armed through its secondary weapon, stops in front of the cell and
    takes the enemy infantry as its target (*enemy obstacle*).
  - (a3) and (a4) As (b): the mean damage is 0 (in (a4), trunc(1 / 2)), so the walker is unarmed. It
    does not enter, and never takes the enemy infantry as its target.
  - (a5) As (a): the mean damage is 1, so the walker is armed and takes the enemy infantry as its
    target.
  - (a6) As (b): the elite primary weapon replaces the ordinary one, so the mean damage is 0.
  - (c2) The walker stops at the wall and takes it as its target. Whether it fires is not part of this
    spec.
  - (c3) The walker never takes the wall as its target and never passes it (*blocked*). Since no path
    leads through a *blocked* cell, it may not walk up to the wall at all.
  - (c4) The same as (c2): the walker takes the wall as its target (an obstacle value).
  - (c5) The walker's route takes the bypass from the start, and it never takes the wall as its
    target. The wall gives it *friendly obstacle*, which costs 60 for the step (S-0212 claim 8),
    more than the 40 extra cells of the bypass.
  - (c6) The walker's route leads to the wall, and it takes the wall as its target as in (c4). The
    wall gives it *enemy obstacle*, which costs 20, less than the bypass.
  - (d) The crushing vehicle drives on (S-0210). The other never drives into the cell while the
    infantry stands there (*enemy obstacle* when it is armed, *blocked* when not, claim 5); what it
    does instead belongs to S-0207.
  - (d2) The crusher drives into the enemy vehicle's cell without stopping in front of it.
  - (e) The walker never attacks the spy. It stops in front of the cell and sends a cell-wide request,
    so the spy may move away (S-0208 claim 11).
  - (f) The vehicle starts uncloaking for every house when the walker asks about its cell
    (*cloaked blocker*, claim 9), not merely becoming visible to the human player. The walker does
    not enter that cell while the vehicle stands there. Scenario C19(d) distinguishes the houses.
  - (f2) The vehicle still gives *cloaked blocker*, although the human player sees it, so a path
    through its cell exists when the order is given (at a step cost of 1000, S-0212 claim 8). The
    walker sets out and walks up to *P*. It does not enter the vehicle's cell while the vehicle
    stands there; what it does in *P* is claim 9's.
  - (g) The walker stops in front of *X* and does not enter it (*blocked*). Its new search finds no
    path, which S-0212 (claim 23) handles. The cloaked unit stays invisible to the human player.
  - (h) The walker stops in front of the cell, takes the cloaked unit as its target (*enemy
    obstacle*, claim 9) and does not enter the cell. Reaching the cell does not by itself make the
    unit visible. Whether the walker can fire on it, and what firing reveals, belongs to the combat
    and visibility specs.
- *Falsified if:*
  - (a) or (b) the walker walks into the cell while the enemy stands there,
  - (b) the walker attacks,
  - (a2) the walker takes no target and stops short of the cell or does not set out (as a test for
    a primary weapon would make it: *blocked*),
  - (a3) or (a4) the walker takes the enemy infantry as its target (in (a4), as a sum of the two
    weapons' damage, 1, would make it armed),
  - (a5) the walker takes no target (as a test of the damage field alone would make it), or (a6)
    it takes the enemy infantry as its target (as a test of the ordinary weapon would make it). If
    (a3) did not show the walker unarmed, (a5) cannot decide,
  - (c2) or (c4) the walker gives up or walks away without taking the wall as its target,
  - (c3) the walker takes the wall as its target,
  - (c5) the walker's route leads to the wall (as *enemy obstacle* would), or (c6) its route takes
    the bypass from the start (as *friendly obstacle* would). If (c2) or (c4) did not show an
    obstacle value, (c5) and (c6) cannot decide,
  - (d) the crushing vehicle stops at the cell, or the other vehicle drives into it while the
    infantry stands there,
  - (d2) the crusher stops in front of the enemy vehicle's cell, or goes around it,
  - (e) the walker attacks the spy,
  - (f) the walker is stopped without the vehicle appearing, or enters its cell,
  - (f2) the walker does not set out, or gives up with its destination cleared before it reaches
    *P* (as *blocked* would make it, were the cloak state disregarded for a house that sees the
    vehicle: no path, S-0212 claim 23). If the vehicle is not visible to the human player when the
    order is given, or it moves, the run cannot decide,
  - (g) the cloaked unit becomes visible when the walker reaches the cell in front of *X*, or the
    walker enters *X*. If the unit is not in *X*, cloaked, when the walker arrives, the run cannot
    decide,
  - (h) the walker enters the cell, or takes no target.

  Part (i) of earlier revisions, a vehicle and an allied infantry unit, is now scenario C20 (a).

**Scenario C4: stationary friends and the cell-wide request (claims 7, 8, 11).**
- *Setup:* a target cell holds three idle infantry of the human player, so its side positions are
  full.
  - (a) A fourth infantry is ordered to a cell three cells beyond the target cell, with the target
    cell on the only route (a corridor of impassable cells). The destination is farther than the
    close-enough distance from the cell in front of the target cell. The corridor cells beyond the
    target cell are free, so that every occupant has free cells near it to move to (S-0208 claim 13).
  - (b) A fourth infantry is ordered to the target cell itself, from a cell nearer than the
    close-enough distance.
  - (c) As (a), with the three units replaced by three idle units of an allied computer house.
  - (d) The height condition. A test file sets the close-enough distance to 4 cells (1024 leptons by
    F2). In a straight corridor, the fourth unit asks from a flat cell *P* at height level *L*. The
    target cell *X* next to it is a slope cell of height level *L*, raised on the side away from
    *P*, and holds the three occupants. The next cell beyond *X* is a slope cell of height level
    *L* + 1, raised the same way, and the cell beyond that, the fourth unit's destination *G*, is
    flat at height level *L* + 2. Each step up has a slope as its lower cell, as the height rule of
    S-0212 (claim 10) requires. A cell's position is its centre at the floor height there, so *G*'s
    world height is 208 leptons above that of the fourth unit standing in *P*, and the distance *D*
    of F1 (about 800 leptons for three cells along an axis) is within the enlarged close-enough
    distance. Control (d′): the same layout and test file with every cell flat at height level *L*.
  - (e) As (b), with the fourth unit asking from a cell whose land type is *tunnel* (S-0211).
  - (f) As (b), but the fourth unit is given an enter order (S-0214) for an idle transport of the
    human player with room for it. The transport stands in the cell beyond the target cell, so the
    route to it leads through the target cell and the transport is nearer than the close-enough
    distance to the cell from which the fourth unit asks.
  - (g) As (a), with the three occupants placed by the map at the right, left and bottom positions
    (map values 2, 3 and 4, S-0208 claim 5) and one more idle infantry of the human player placed
    at the centre position (map value 0). The walker is a fifth unit.
- *Procedure:* give the order and watch 300 ticks.
- *Observable:* whether the three occupants move away, and where the fourth unit stops. In (a) and
  (c), also the tick on which each occupant's recorded destination changes. In (g), also
  the ticks between the walker's first stop in front of the target cell and the tick on which it
  gives up or moves away. In (f),
  also whether the recorder shows the fourth unit in contact with the transport on the tick on which
  it stops in front of the target cell.
- *Runs:* 1 each.
- *Expected:*
  - (a) A forced, move-anywhere request reaches every occupant on one tick, the fourth unit's asking
    tick, so all three recorded destinations change on that same tick. The occupants then move to
    free cells nearby and the fourth walks on.
  - (b) The fourth stops next to the target cell and its move ends. The occupants stay.
  - (c) The same as (a).
  - (d) The world height difference is 208 leptons, not less, so the fourth does not give up: the
    occupants move away and it walks on through the target cell. (d′) The fourth stops in *P* and
    its move ends (the distance is within the close-enough distance and the heights are equal); the
    occupants stay.
  - (e) The occupants move away and the fourth walks into the target cell.
  - (f) If the fourth unit is in contact with the transport when it stops, the occupants move away
    and it walks on to the transport. If it is in contact with nothing, it gives up next to the
    target cell and the occupants stay.
  - (g) Four allied infantry stand still in the cell, so it gives *moving blocker* (claim 4). No
    request is sent and the occupants stay. The walker waits at least the blockage delay of the
    stock rules before its search may avoid them (claim 10). With
    `quirks.stationary_friendly_exactly_three` off, the cell would give *stationary friendly*
    instead: all four occupants' recorded destinations would change on the walker's asking tick,
    as in (a).
- *Falsified if:*
  - (a) or (c) the occupants stay and the fourth waits forever, or any one of the three occupants
    gets no new destination on the tick on which another one does (a single occupant left out of the
    request),
  - (b) the occupants are moved away for a unit that is already within the close-enough distance,
  - (d) or (e) the fourth gives up with the occupants in place. In (d), if the control (d′) does
    not give up either, the part cannot decide,
  - (f) the fourth unit gives up while the recorder shows it in contact, or the occupants move away
    while it is in contact with nothing,
  - (g) the occupants move away, or the walker gives up or moves away before the blockage delay has
    passed.

**Scenario C5: moving blockers and the blockage delay (claims 4 and 10).**
- *Setup:* a one-cell-wide corridor, eight cells long. A side passage that only infantry can use runs
  around it: a land type that is impassable for the vehicle and passable for walkers (S-0211). The
  side passage is at least 8 cells longer than the infantry's route through the corridor, so that a
  search made before the blockage delay has run out keeps the corridor route (claim 10).
  - (a) The corridor's far end opens on two branches. A vehicle *V*₁ of the human player is ordered
    to a destination on the first branch. An idle, unarmed computer-owned vehicle *V*₂ blocks that
    branch, so *V*₁ stops in the last cell of the corridor but keeps its move order. An infantry of
    the human player follows *V*₁ two cells behind, with a destination on the second branch, so its
    route through the corridor passes *V*₁'s cell and not *V*₂'s. Whether *V*₁ keeps its order while
    it stands in front of *V*₂ belongs to the vehicle rules of S-0207 and S-0212; if another blocker
    keeps *V*₁ standing with its order more reliably, it may replace *V*₂.
  - (b) No *V*₂. *V*₁ drives through the corridor at a speed below the infantry's, without
    stopping. A test file sets the blockage delay below the corridor transit time of *V*₁ and above
    the time *V*₁ needs per cell.
- *Procedure:* record the infantry's position per tick.
- *Observable:*
  - (a) the ticks between the infantry's first stop behind *V*₁ and the tick on which it starts into
    the side passage.
  - (b) whether the infantry takes the side passage.
- *Runs:* 3 each.
- *Expected:*
  - (a) At least the blockage delay of the stock rules, and less than the blockage delay plus the
    path retry time of S-0212. *V*₁ keeps its destination and stands between steps, so it gives
    *moving blocker* (claim 4).
  - (b) The infantry follows *V*₁ through the corridor. Its total waiting exceeds the blockage delay,
    but no single wait lasts that long. *V*₁ is mostly in the middle of a step, and then the
    vehicle-present flag gives *moving blocker*.
- *Falsified if:*
  - (a) the infantry leaves before the blockage delay, or never leaves (with *V*₁ still holding its
    order). If the recorder shows *V*₁ without a destination, or moving, on any tick while the
    infantry waits behind it, the run cannot decide,
  - (b) the infantry takes the side passage.

**Scenario C6: crates and computer-owned walkers (claim 4, row WO1).** This scenario measures the
original's computer behaviour. In our engine an AI house's walkers behave as the human-owned unit
does, by the rule of glossary *AI house* (claim 4), which needs no measurement.
- *Setup:* a crate placed by the map in a cell of a one-cell-wide corridor. A computer-owned infantry
  unit is moved through the corridor by a map trigger. In a second run, a human-owned infantry unit
  is ordered through it.
- *Procedure:* watch 300 ticks.
- *Observable:* whether the unit enters the crate's cell, and whether the crate is taken.
- *Runs:* 1 each.
- *Expected:* the computer-owned unit never enters the cell: it stops short, gives up, or does not set
  out, since no path leads through a *blocked* cell. The human-owned unit walks through and takes the
  crate.
- *Falsified if:* the computer-owned unit enters the crate's cell, or the human-owned unit fails
  to enter and take the crate while it remains there and no other object or rule blocks its route.

**Scenario C7: who receives a request that is not move-anywhere (claims 11, 12).** A crushing
vehicle's warning request (S-0210 claim 7) is forced and not move-anywhere. An infantry unit that
receives it and accepts it (S-0208 claim 11) steps aside. One that does not receive it is crushed.
- *Setup:* a one-cell-wide corridor. An infantry unit stands idle in one of its cells. A vehicle
  that can crush is driven through the corridor, ordered by its owner or moved by a map trigger. The
  infantry and the crusher belong to different houses that are not allied.
  - (a) A computer-owned infantry and a human-owned crusher.
  - (a′) As (a), with a test file that sets the scatter threshold above the maximum skill level and
    turns the player-scatter rule off.
  - (b) A human-owned infantry and a computer-owned crusher, with a test file that turns the
    player-scatter rule off.
  - (b2) As (b), with a test file that turns the player-scatter rule on.
  - (b3) As (b), with a second human-owned infantry of elite rank (set by the map) in the same cell.
  - (b4) As (b), with a test file that gives the infantry type the scatter veterancy ability at
    veteran rank, and the map making the unit a veteran.
  - (b5) As (b), with a test file that sets the scatter threshold to 0.
  - (c) As (b), with the infantry pre-placed by the map as a unit of the neutral house (S-0009,
    claim 27), of a stock type that is not a fraidycat (S-0107 Terms), and the crusher owned by the
    human player. A computer player takes part, so that the
    unused-house cleanup of S-0009 claim 28 does not run and the infantry stays on the map.
  - (c2) As (c), with the infantry a unit of the special house.
- *Procedure:* run until the crusher has passed the cell.
- *Observable:* whether each infantry unit survives (steps aside) or is crushed.
- *Runs:* 3 each. In (c) and (c2) a run counts only when the recorder shows the infantry present at
  the start and not moving before the crusher reaches its cell.
- *Expected:*
  - (a) survives, provided the stock scatter threshold is at most the stock maximum skill level.
  - (a′) crushed, provided the stock infantry type has no scatter veterancy ability at the rank the
    map gives the unit (read from the stock data when the scenario is built).
  - (b) crushed, provided the stock scatter threshold is above 0 and the stock infantry type has no
    scatter veterancy ability at the rank the map gives the unit (both read from the stock data when
    the scenario is built).
  - (b2), (b3), (b4) and (b5) survives; in (b3) both units survive.
  - (c) and (c2) crushed, with the two provisions of (b), and provided the infantry's house is not
    allied with the human player's house (no spec states the alliances of the neutral and special
    houses; the recorder reads them).
- *Falsified if:*
  - (b) the infantry survives while both provisions of its expectation hold,
  - (c) or (c2) the infantry survives while the three provisions of its expectation hold (that
    would show the house at the maximum skill level, as a computer player's),
  - (a′) the infantry survives while the provision of its expectation holds,
  - any of (b2) to (b5) the infantry is crushed,
  - (a) the infantry is crushed while the stock threshold is at most the stock maximum.

**Scenario C8: gates (claims 4, 7, 9).** A gate closes a one-cell-wide corridor, and there is no
other route.
- *Setup:*
  - (a) A closed gate of the human player, and an infantry of the human player whose route leads
    through it.
  - (b) A closed gate of the computer player, and an armed infantry of the human player.
  - (c) As (b), with an unarmed infantry.
  - (c2) As (c), with the walker made elite by the map, and a test file that gives its type an
    elite primary weapon with a positive damage field and leaves it without an ordinary primary
    weapon.
  - (d) As (b), with a computer-owned unit made to pass through the gate by a map trigger, timed so
    that the gate is open when the walker reaches it.
- *Procedure:* give the move order and watch 300 ticks.
- *Observable:* whether the gate opens, and whether the walker passes, attacks the gate or waits.
- *Runs:* 1 each.
- *Expected:*
  - (a) The gate opens and the walker passes.
  - (b) The walker attacks the gate.
  - (c) The walker neither passes nor attacks.
  - (c2) As (b): the elite walker has a primary weapon (Terms *armed*), so the gate gives *enemy
    obstacle* (row WP13) and the walker attacks it.
  - (d) The walker passes while the gate is open.
- *Falsified if:*
  - (a) the walker gives up or attacks the gate,
  - (b) it passes the closed gate or never attacks,
  - (c) it attacks or passes the closed gate,
  - (c2) it neither passes nor attacks, as in (c) (the ordinary primary slot alone asked). If (c)
    does not hold, the part cannot decide,
  - (d) it stops at the open gate.

**Scenario C9: listing order (claims 9, 13).**
- *Setup:* a one-cell-wide corridor. A corridor cell *X* holds two unarmed computer-owned infantry
  units, *A* and *B*.
  - (a) Both are pre-placed by the map, *A* before *B* in the map's order of infantry.
  - (b) As (a), with the map order swapped.
  - (c) *A* is pre-placed in *X*. *B* is moved into *X* by a map trigger before the walker arrives.
- *Procedure:* an armed infantry of the human player is ordered along the corridor through *X*.
- *Observable:* which of *A* and *B* the walker attacks first.
- *Runs:* 2 each.
- *Expected:*
  - (a) *B*. It entered *X* last, so it is listed first.
  - (b) *A*.
  - (c) *B*.
- *Falsified if:* in (c) the walker attacks *A* first, in (a) it attacks *A* first, or in (b) it
  attacks *B* first. The latter two falsify the claimed map-placement order; identical first targets
  in (a) and (b) also falsify its claimed dependence on that order.

**Scenario C10: a building that is invisible in play (claims 1, 3, 4).**
- *Setup:* a one-cell-wide corridor of impassable cells. One of its cells holds a stock building of
  a type that is invisible in play (a light source), placed by the map.
  - (a) An infantry of the human player stands at one end, and is ordered to the light source's
    cell.
  - (b) As (a), with the infantry ordered to a cell past the light source.
  - (c) The building flag, through the free-cell test of S-0212 claim 25. As scenario C4 (a): a
    target cell *X* in the corridor holds three idle infantry of the human player on its side
    positions, and a fourth is ordered to a cell three cells beyond *X*, so that it sends the
    occupants a forced, move-anywhere request without a threat point from the cell *P* in front of
    *X* (claim 8). Here the light source stands in the corridor cell *E*₁ just past *X*. The corridor
    cell *E*₂ beyond it, and the corridor cell two cells before *X* (behind the fourth unit), are
    empty. Each occupant's scatter searches around *X* (S-0208 claim 13). Of the cells one step from
    *X*, *P* holds the fourth unit's mark and the side cells are impassable, so *E*₁ is the only one
    whose record can be empty.
- *Procedure:* give the order; in (c), record until the occupants have new destinations, or 300
  ticks.
- *Observable:* (a), (b) whether the infantry stands in, or walks through, the light source's cell.
  (c) each occupant's new destination cell, on the recorder.
- *Runs:* (a), (b) 1 each; (c) 3.
- *Expected:*
  - (a) The infantry stands in the cell. (b) It walks through it.
  - (c) The light source sets the building-present flag of *E*₁, so *E*₁ never qualifies, and the
    search collects only the two empty corridor cells two steps from *X*. Every occupant's new
    destination is *E*₂ or the corridor cell two cells before *X*, never *E*₁. If buildings set no
    flag, *E*₁ would be the only cell collected one step from *X*, and every occupant would get it.
- *Falsified if:* (a), (b) the infantry stops in front of the cell or goes around it. (c) any
  occupant's new destination is *E*₁. If (a) or (b) fails, or no occupant gets a new destination,
  part (c) cannot decide.

**Scenario C11: a walker guarding a vehicle (claim 4, row WP5).**
- *Setup:* a one-cell-wide corridor. An idle vehicle *V* of the human player stands in a corridor
  cell. An armed infantry *W* of the human player stands in the corridor 5 cells from *V*. A test
  file sets the close-enough distance to 0.1 cells (25 leptons by F2), so that no walker next to
  *V*'s cell is within it.
  - (a) *W* is ordered to area-guard *V*, with the gesture of S-0214 table G, row G6 (force-move and
    force-fire held, the target an allied object), as S-0213 scenario O3 gives it to a vehicle in
    its step 3. Its destination and guard spot become *V* (Terms), so its route ends in *V*'s cell.
  - (b) Control: *W* is given a plain move order to the corridor cell just past *V*.
- *Procedure:* give the order and watch 300 ticks.
- *Observable:* whether *V* moves away from its cell (on the recorder), and where *W* stops. In (a),
  also *W*'s behaviour, destination and guard spot just after the order, as S-0213 scenario O3
  records them.
- *Runs:* 2 each.
- *Expected:*
  - (a) *W* stops next to *V*'s cell. *V* does not move: row WP5 gives *blocked*, so *W*
    sends no request.
  - (b) *W* stops next to *V*'s cell and sends a cell-wide, move-anywhere request (*stationary
    friendly*, claim 8). Whether *V* then moves aside belongs to the vehicle-movement specs.
- *Falsified if:* in (a), *V* moves away while *W* stands next to its cell, in a pair of runs where
  *V* moved aside in (b). If *V* does not move in (b), if *W* in (a) never reaches the cell next to
  *V*, or if *W*'s recorded behaviour, destination and guard spot in (a) are not *area guard*, *V*
  and *V*, the scenario cannot decide.

**Scenario C12: the infantry house (claims 1, 4, Edge cases).** A one-cell-wide corridor runs
through a cell *X*. An armed infantry *W* of the human player is ordered along the corridor to a cell
past *X*. *W* starts farther from *X* than its own target-acquisition range, and is given its order
at the start, so that it does not engage the units in *X* before it reaches them. The infantry
placed in *X* below are of unarmed stock types (for example engineers), idle, and placed by the
map.
- *Setup:*
  - (a) *E*₁ stands at the centre position of *X* (map value 0, S-0208 claim 5), and *E*₂ at its
    right position (map value 2). Before *W* arrives, a map trigger moves *E*₂ out of *X*, so that
    no side position of *X* is taken any more.
  - (b) Control: as (a), with no trigger, so that *E*₂ stays.
  - (c) *X* holds *E*₁ at the right position and an unarmed infantry *H* of the human player at the
    left position.
  - (d) As (c), with the order of *E*₁ and *H* in the map file swapped.
  - (e) *X* holds *E*₁ at the right position and an unarmed infantry *A* of a computer house allied
    with the human player at the left position. *A* comes after *E*₁ in the map order that, in (c)
    and (d), made the later unit's house the infantry house. Before *W* arrives, a map trigger moves
    *A* out of *X*, so that only *E*₁ stays.
- *Procedure:* give *W* its order and record until *W* has passed *X*, or 300 ticks.
- *Observable:* whether *W* walks into *X* while *E*₁ stands there, and whether it takes *E*₁ as its
  target.
- *Runs:* 2 each.
- *Expected:*
  - (a) *W* walks into *X* and on, without taking *E*₁ as its target. When *E*₂ leaves, the infantry
    house of *X* becomes none, so *E*₁ gives no value.
  - (b) *W* stops in front of *X* and takes *E*₁ or *E*₂ as its target (*enemy obstacle*).
  - (c), (d) In one of the two map orders, the one in which *H* is placed after *E*₁, the infantry
    house is the human player's and *W* walks into *X* (it takes the free bottom position). In the
    other, the house is the computer's and *W* stops in front of *X* and takes *E*₁ as its target.
  - (e) The infantry house stays *A*'s after *A* has left (Edge cases), so *E*₁ gives no value and
    *W* walks into *X* and on without taking *E*₁ as its target.
- *Falsified if:*
  - (a) *W* stops in front of *X* or takes *E*₁ as its target (this is what
    `quirks.occupancy_house_side_positions_only` off would do),
  - (b) *W* walks into *X*,
  - (c), (d) *W* behaves the same in both map orders. Which order is which depends on the order in
    which the map places infantry (claim 13); that part decides only "the most recent one's house
    wins",
  - (e) *W* stops in front of *X* or takes *E*₁ as its target (this is what
    `quirks.occupancy_house_stale` off would do). If (c) and (d) did not show the order dependence,
    (e) cannot decide.

**Scenario C13: enslaved walkers at their master (claims 3, 4, row WP2).** The human player plays
a house whose stock units include a slave-miner harvester.
- *Setup:* a slave-miner harvester of the human player with its slaves, next to a small resource
  field, on clear terrain.
  - (a) The harvester stays undeployed and idle while its slaves mine and return.
  - (b) The harvester is deployed into its building form before the slaves return.
- *Procedure:* let the slaves mine and return for 1500 ticks.
- *Observable:* on the recorder, the cells that each returning slave's position enters, relative
  to the master's cell in (a) and to its foundation in (b).
- *Runs:* 2 each.
- *Expected:*
  - (a) Returning slaves walk into the cell the harvester stands in while it stands there, and do not
    stop at its edge.
  - (b) Returning slaves walk into one fixed foundation cell, *W* − 1 cells along x and ⌊*H* / 2⌋
    cells along y from the top-left cell (Terms).
- *Falsified if:* returning slaves only ever stop at the edge of the master's cell in (a), or of the
  foundation in (b). In (b), if they enter a different foundation cell, the Terms entry for the
  unloading cell is wrong, not row WP2.

**Scenario C14: a mission target that is being erased (claim 4, rows WP3 and WP6, and the zero
walker speed check of part 4a).** The units of the human player are pre-placed by the map,
whatever house the human player plays.
- *Setup:*
  - (a) An enemy building of a stock type that can be captured, owned by the computer player, with
    one clear approach of a one-cell-wide corridor. An engineer of the human player stands 6 cells
    away. A stock unit of the human player whose weapon erases (a chrono weapon) stands within its
    range of the building.
  - (b) An idle enemy vehicle of a stock type that is not a train, owned by the computer player, in
    a one-cell-wide corridor. An infantry unit of a stock vehicle-thief type of the human player
    stands 6 cells away in the corridor. An eraser as in (a) stands within its range of the vehicle.
  - (c) As (b), with the vehicle standing on a cell of a land type *L* on which the vehicle may
    stand, and a test file that sets *L*'s speed percentage for walkers to 0 (its speed for the
    vehicle's class stays as it is). *L* is chosen from the stock land types when the map is built,
    so that only the vehicle's cell has it.
  - (c′) Control: as (c), with no eraser.
- *Procedure:* (a) order the engineer to capture the building; (b), (c), (c′) order the thief to
  take the vehicle (S-0214 table I, row I15). In (a) to (c), when the walker is 2 cells from its
  target, order the eraser to attack the target.
- *Observable:* whether the walker's position enters the target's cell, or the building's
  foundation, while the target is being erased. On the recorder, the walker's behaviour and
  destination on each tick.
- *Runs:* 2 each.
- *Expected:*
  - (a) While the building is being erased, it counts as an enemy building and not as the
    engineer's mission target, so the unarmed engineer gets *blocked* and stops at the edge of the
    foundation.
  - (b) While the vehicle is being erased, row WP3 passes it over, but row WP6 still overrides: the
    thief walks into the vehicle's cell.
  - (c) The same as (b), although walkers may not stand on *L*: an outcome decided by row WP6 is
    exempt from the zero walker speed check (part 4a).
  - (c′) Row WP3 decides, and the zero walker speed check still applies to it, so the vehicle's cell
    is *blocked* for the thief: it never walks into that cell.
- *Falsified if:*
  - (a) the engineer walks into the foundation while the building is being erased,
  - (b) or (c) the thief stops at the edge of the vehicle's cell while the vehicle is being erased
    and the recorder still shows the vehicle as the thief's destination,
  - (c′) the thief walks into the vehicle's cell. If the thief in (c) never comes within
    one cell of the vehicle (because the test file changed which cells its path search can reach,
    S-0211 claims 11 and 12), (c) cannot decide.

  If the walker's order ends or its destination changes before it reaches the target, or the
  erasure ends first, the run cannot decide and is repeated with the eraser's order given earlier.

**Scenario C15: buildings and aircraft that block (claim 4, rows WP19, WP23 and
WP24, and rows WP13 and WP28 for parts e6 and e7; claim 9 for parts e1 to e3 and e5 to e7).**
Parts a to d use a one-cell-wide corridor that the blocker closes, with no other route. The walker's
destination is a cell beyond the blocker, farther than the close-enough distance from the cell in
front of it.
- *Setup:*
  - (a) A building of the human player, of a stock type that is neither a gate nor invisible in
    play, and an infantry of the human player.
  - (b) An aircraft of the human player resting on the ground in a corridor cell (a stock aircraft
    type that lands at the end of a move, ordered there first), and an infantry of the human player.
  - (c) A building of a stock type marked as a bridge repair hut, owned by the computer player (set
    by the map), and an armed infantry of the human player.
  - (d) As (c), with a stock building of the computer player of a type that has no weapon and is
    neither a gate, nor a bridge repair hut, nor invisible in play.
- *Procedure:* give the move order and watch 300 ticks.
- *Observable:* whether the walker's position enters a cell of the blocker, and whether the walker
  takes the blocker as its target (presentation-only target line).
- *Runs:* 1 each.
- *Expected:*
  - (a) and (b) The walker never enters the blocker's cell and takes no target (*blocked*). Since no
    path leads through a *blocked* cell, it may stop short or not set out at all.
  - (c) The walker never enters the hut and takes no target (*blocked*), with the same allowance.
  - (d) The walker stops in front of the building and takes it as its target (*enemy obstacle*,
    claim 9).
- *Falsified if:*
  - (a), (b) or (c) the walker enters a cell of the blocker,
  - (c) the walker takes the hut as its target,
  - (d) the walker enters a cell of the building, or stays in front of it without taking it as its
    target.

  If no stock aircraft type rests on open ground long enough, part (b) cannot decide.

- *Parts e1 to e3 and e5 to e7 (Q-2396): an occupant the walker cannot fire at.*
  - *Setup:* a purpose-made flat map, every cell at one height level, with a wall of impassable
    cells and a one-cell gap in it, closed
    by one foundation cell *X* of a stock building *B* of the computer player of the type of part
    (d). The rest of *B*'s foundation lies among the wall's impassable cells, and *X* is not in the
    last column along x of a building whose type has the bib flag (a test file removes the flag
    otherwise). The walker *W* is an infantry of the human player whose only weapon is a test
    weapon (a named test rules file) in its primary slot, with no secondary weapon and no elite
    weapon: positive damage, and a warhead whose versus value is 0 against every building armour,
    so the firing check answers *illegal* against *B*, and *W* is armed (Terms) and has a primary
    weapon (row WP13). The test weapon's range puts the occupant within reach of *W* (S-0307) from
    every cell next to *X* on *W*'s side: it is at least one cell more than the distance from such
    a cell's centre to the occupant's reference point (for a building, the centre of its
    foundation, S-0204 claim 4). The test file also clears the explosive-charges flag of *W*'s
    type and any explosive-charges veterancy ability of it. *W* is given its one order of the
    part and no other, and no route-mode order, so it follows no route (S-0213 claim 16). That
    order leaves it with no queued destinations (S-0213 claim 5, table T3 part g), and only orders
    fill that list (S-0215 Terms). A run counts only if *W*'s guard spot is never on a cell other
    than its own. At each fall-back after a drop *W* holds no target, and no destination, since
    the take is an override with no destination (claim 9). So no row of S-0215 claims 13 and 14
    gives it a destination or an order at its fall-backs. No other object is near, except in
    (e5) below, and no object attacks *W* in any part, so no retaliation (S-0215 claim 18) gives
    it a target.
    - *Map of parts e1, e2 and e5 to e7.* The wall runs across the whole usable area, from one edge
      of it to the opposite edge, so that inside the usable area the gap is the only way from *W*'s
      side to the other. The usable area leaves a strip of legal cells outside it along one edge
      only: the edge on *W*'s side that runs parallel to the wall, at least 6 cells from the wall's
      face. *W* is placed by the map in that strip, as the movers of scenario C20 are. Its order's
      destination is a cell inside the usable area beyond the gap, farther than the close-enough
      distance from the cell in front of *X*.
      - *The search at the order.* *W*'s cell is a legal cell outside the usable area, so the
        search skips the reach-region comparison (S-0212 claim 6, second exception) and runs
        without a corridor (S-0212 claim 14). Every legal cell outside the usable area lies on
        *W*'s side, so the path it finds enters the usable area on that side and leads through
        *X*, the only way across the wall there. A search that runs while *W* is still outside
        the usable area runs in the same way.
      - *Entering the usable area.* *W* enters the usable area before it reaches the cell in front
        of *X* (the recorder checks it, see the Procedure). From then on it has the
        in-the-playfield mark (S-0215 Terms: the mark is set once the object has entered the
        usable area), so S-0215 claim 37 (f) leaves its destination unchanged while its target
        is in range.
      - *Searches after entering.* Every search after *W* has entered runs from a cell inside the
        usable area. A building
        leaves its cells *open* and a unit never changes a category (S-0212 claim 1), so *W*'s
        cell and the destination lie in one reach region, and the search passes the reach-region
        comparison (S-0212 claim 6). Cells outside the usable area give *blocked* to a walker
        inside it (claim 4a), so every path such a search finds leads through *X*, with or
        without the corridor (S-0212 claims 13 and 14).
    - *Map of part e3.* The usable area's edge runs along the wall's face on *W*'s side: every cell
      on that side, the cell in front of *X* among them, is a legal cell outside the usable area,
      and *X*, the wall and every cell beyond it lie inside it. *W* is placed on that side as the
      movers of scenario C20 are. The
      way round the wall is 3 cells longer than the way through the gap, counting straight steps
      only, and the destination is a cell inside the usable area beyond the gap. *W* starts
      outside the usable area, so its search at the order skips the reach-region comparison
      (S-0212 claim 6, second exception) and runs without a corridor (S-0212 claim 14), and the
      route depends on the cell costs alone (S-0212 claim 8). The route through *X* costs 19 more
      than an open gap would (one *enemy obstacle* step instead of a *clear* one), so the margin
      of 16 exceeds the 3 cost units that S-0212 scenario P13 allows for the direction terms and
      the estimate.
    - (e5) an attacker *K* of the human player stands beyond the wall, at least 3 cells from *X*
      and from every cell of the straight line from *X* to the destination, with a test weapon
      whose range reaches *B* from there and that destroys *B* with one shot.

    The last two parts change the occupant:
    - (e6) As (e1), with *B* replaced by a closed gate *G* of the computer player, of a stock gate
      type, placed as *B* is, with one foundation cell *X* in the gap. No unit of the computer
      player comes near, so *G* stays closed (row WP13). The warhead's versus value of 0 against
      every building armour makes the firing check answer *illegal* against *G* too.
    - (e7) As (e1), with no building in the gap: the map places in *X* an idle unarmed vehicle *V*
      of the computer player, of an ordinary stock type (row WP28). The test weapon's warhead has
      a versus value of 0 against every armour, so the firing check answers *illegal* against *V*
      too. *W* stays armed, since Terms *armed* reads only the damage fields.
  - *Procedure:* (e1), (e3) and (e5) to (e7) a move order to the destination, (e2) an attack-move
    order to it. In (e5) the harness selects *K* by scripted normal input in the run's first input
    cycle. Let *t* be the tick at whose end the recorder first shows *B* as *W*'s target. In the
    cycle of tick *t* + 1, whose input comes after tick *t*, the harness gives *K* a force-fire
    order on *B* by scripted normal input, so the order takes effect at the end of tick *t* + 1
    (S-0213 claim 1, S-0203 claim 7). Record every tick for 600 ticks *W*'s cell, whether that cell
    is inside the usable area, its target, its current behaviour, its destination, its guard spot,
    whether it holds a path, the shots it fires and the tick number. In (e5) record *B*'s health and
    the tick of the hit that destroys it. A run of (e5) counts only if *W*'s target is still *B* at
    the end of tick *t* + 1 and the hit that destroys *B* comes in a tick before the first multiple
    of 16 after *t*. A run of (e1), (e2) or (e5) to (e7) counts only if *W*'s cell is inside the
    usable area from a tick before the one at whose end it first holds a target (the occupant), and
    stays inside it. A run of (e7) counts only if *V* stays in *X* until *W*'s first loss of *V*. Up
    to 5 runs of each, and up to 8 of (e5), moving *W*'s start by one cell each time, until 2 count.
  - *Expected:*
    - (e1) *W*'s path leads to the gap. It stops in front of *B*, takes *B* as its target in
      *attack* and fires no shot. It loses the target at the first tick that is a multiple of 16
      after the take's tick, then falls back to rest (S-0215 claims 13 and 14). It never enters
      *B*'s cell and takes no target again in the 600 ticks.
    - (e2) As (e1) up to the first loss of the target. After each loss, *W* either searches a path
      again and takes *B* again without moving on past *X*, or falls back to rest as in (e1). Once
      it rests it stays at rest and takes no target again in the 600 ticks. Each loss of *B* comes
      at a multiple of 16.
    - (e3) *W*'s path goes round the wall. It never stops in front of *B* or takes it as target.
    - (e5) At the end of the tick of the hit that destroys *B*, *W*'s current behaviour is *move*
      with its order's destination again (the override's revert, S-0215 claim 10), and *W* leaves
      the cell in front of *B* within 300 ticks of that hit.
    - (e6) As (e1), with *G* in place of *B*.
    - (e7) As (e1), with *V* in place of *B*.
  - *Falsified if:*
    - in (e1) *W* does not reach the cell in front of *X*, or in (e3) its path leads through *X*,
    - in (e1) it moves on after the loss or takes *B* again,
    - in (e2) after a loss it moves on past *X* without taking *B* again, or takes a target after
      it has fallen back to rest,
    - in (e2) *W* loses *B* at a tick that is not a multiple of 16,
    - in (e5) it is not in *move* with that destination at the end of that tick, or it stays in
      front of *B*,
    - in (e6) or (e7) as in (e1), with *G* or *V* in place of *B*,
    - or *W* enters *B*'s cell before *B* is destroyed.

**Scenario C16: terrain objects (claims 1, 2, 4 rows WP27 and WR6, claim 9, Terms *allied*,
Edge cases).** For parts (a) to (c), except (a′), the map's theater and a
stock terrain-object type *T* are chosen so that *T*'s side-position pattern for that theater marks
the bottom position only (read from the stock data when the map is built; a test file may set the
pattern if no stock type has it). Such a pattern puts the cell in a passability category that
infantry may cross (S-0211 claims 11 and 12). A one-cell-wide corridor runs through a cell *X* that
holds a *T* placed by the map. Walkers are ordered along the corridor to a cell past *X*, farther
than the close-enough distance from the cell in front of *X*.
- *Setup:*
  - (a) An armed infantry *W* of the human player.
  - (a′) As (a), with a test file that sets *T*'s side-position pattern for the map's theater to −5,
    and *W* pre-placed at the right position of its cell (map value 2, S-0208 claim 5), so that the
    class of its desired point in *X* is the right position. The pattern −5 marks the right and left
    positions (Terms) and puts the cell in a category that infantry may cross (S-0211 claims 11 and
    12).
  - (b) Two idle infantry of the human player placed by the map in *X* at the right and left
    positions (map values 2 and 3), and *W* as in (a).
  - (c) An unarmed infantry *E* of the computer player is moved through *X* and on by a map trigger
    before an unarmed infantry *W*′ of the human player arrives. Control (c′): as (c), without *E*.
  - (c2) As (c), with an armed infantry *W*″ of the human player in place of *W*′, given its order
    after *E* has left *X*. The corridor through *X* is its only route.
  - (c3) As (c), but after *E* has left *X*, destroy *T* by ordinary force-fire from a separate
    human-owned attacker whose position and walking target never enter *X*. Use a weapon that can
    destroy *T*; a named test file may give its warhead the wood-damaging property if needed.
    Give *W*′ its move order only after *T* is removed. Control (c3′): remove *T* in the same way without first
    passing *E* through *X*. A read-only recorder also logs *X*'s infantry house, standing-position
    marks and listed objects each tick. Neither run may have any intervening infantry mark in *X*.
  - (d) A separate map in the snow theater, with a stock terrain-object type *T*₇ whose snow
    side-position pattern marks all three side positions (read from the stock data). A
    computer-owned fraidycat civilian (S-0107 Terms) walks north to a cell *D* and is hit from due
    map east when 4 cells from *D*, as in S-0208 scenario W7 (b), so that it receives only the
    request with a threat point. Of *D*'s eight neighbours, the cells map south and map east of *D* are open ground and the
    other six each hold a *T*₇ placed by the map.
- *Procedure:* give the move order and record until the walker has passed *X*, or 300 ticks. In
  (c3), record from before *E* enters until 300 ticks after *T* is removed, with the control using
  the same removal and observation period. In (d), record until the civilian has a new destination
  and has reached it, or 300 ticks.
- *Observable:* whether the walker walks into *X*, the standing position it takes there (on the
  recorder), and whether it takes *T* as its target. In (b), also whether the occupants move away,
  and the ticks between *W*'s first stop in front of *X* and the tick on which it gives up or moves
  away. In (d), the civilian's new destination cell (on the recorder).
- *Runs:* 2 each; part (d) 10, so that the draw of S-0208 claim 12 takes several values.
- *Expected:*
  - (a) *W* walks into *X*, at the right or the left position but never at the bottom one, and on.
    It never takes *T* as its target (row WP27 gives none, and *T* marks the bottom).
  - (a′) *W* walks into *X* at the bottom position, the only free side position, and on. (Reading
    the pattern by the size of the number, 5, would leave the left position free; ignoring a
    negative pattern would leave the right one free.)
  - (b) The bottom position is marked by *T*, so all three side positions are taken while only two
    allied infantry stand still there: *moving blocker* (claim 4). The occupants stay, and *W* waits
    at least the blockage delay of the stock rules.
  - (c) After *E* has left, the infantry house of *X* stays the computer's (claim 1, Edge cases), so
    *X* is *blocked* for *W*′ although no unit stands there: *W*′ never walks into *X*. It stops in
    front of *X*, or short of it, or does not set out, depending on whether its path was computed
    before *E* passed. (c′) *W*′ walks through *X*.
  - (c2) *X* gives *W*″ *enemy obstacle* through the stale house (row WR3), so its route leads to
    *X*. On the repeated ask it considers the first listed object of *X*, the terrain object *T*,
    which is allied with no house, and takes *T* as its target (claim 9). It does not walk into *X*.
  - (c3) *T*'s removal frees its side-position marks but leaves *X*'s infantry house the computer's.
    With no listed objects or position marks, *X* still gives *blocked* to the unarmed *W*′
    (row WR2), which does not enter. In (c3′) the house remains none and *W*′ walks through *X*.
    With `quirks.occupancy_house_stale` off, removal would set the house to none in (c3) too.
  - (d) The six cells with a *T*₇ give *blocked* (row WR6), so the scan of
    S-0208 claim 12 passes over them, and the new destination is always the cell south or the cell
    east of *D*.
- *Falsified if:*
  - (a) *W* stands at the bottom position of *X*, takes *T* as its target, or stays in front of *X*,
  - (a′) *W* stands at the right or the left position of *X*,
  - (b) *W* walks into *X*, or the occupants move away (a request was sent, as for *stationary
    friendly*),
  - (c) *W*′ walks through *X* in a pair of runs in which it walked through in (c′) (this is what
    `quirks.occupancy_house_stale` off would do). If *W*′ stops in (c′) too, part (c) cannot decide,
  - (c2) *W*″ stops in front of *X* without taking *T* as its target (as a terrain object counted as
    allied would make it), or walks through *X*. If (c) did not show the stale house, (c2) cannot
    decide,
  - (c3) the recorded house clears on *T*'s removal, its position marks remain after removal, or
    *W*′ enters *X* while the recorded enemy house remains. If *E* did not first leave its house
    recorded in *X*, another infantry marks there before the probe, *T* is not fully removed,
    or *W*′ also fails to pass in (c3′), the pair cannot decide the removal rule.
  - (d) in any run the new destination is a cell holding a *T*₇. If the civilian gets no new
    destination in a run (S-0208 claim 11), that run cannot decide.

**Scenario C17: other occupants of a mission-target building (claim 4, row WP4).**
- *Setup:*
  - (a) An enemy building of a stock type that can be captured and on which vehicles are repaired
    (a service depot), owned by the computer player. An idle vehicle of the computer player stands
    docked on it, placed there by the map or driven there by a map trigger. An engineer of the
    human player stands 6 cells away, pre-placed by the map, on the side of the depot nearest the
    docked vehicle.
  - (b) As (a), with an eraser of the human player as in scenario C14 within range of the docked
    vehicle.
- *Procedure:* order the engineer to capture the depot, and record until it is captured or 600
  ticks. In (b), order the eraser to attack the docked vehicle when the engineer is 2 cells from the
  depot.
- *Observable:* on the recorder, the cells the engineer's position enters, whether it stops at the
  edge of the cell holding the docked vehicle, and whether the depot is captured. In (b), also the
  ticks on which the vehicle is being erased.
- *Runs:* 2 each.
- *Expected:*
  - (a) The docked vehicle gives no value and the depot overrides, so the engineer walks into the
    vehicle's cell whenever its route leads there, and captures the depot. It never stops at the edge
    of the vehicle's cell.
  - (b) While the vehicle is being erased, its own row decides: an enemy vehicle gives the unarmed
    engineer *blocked*, so the engineer stops at the edge of the vehicle's cell when its route leads
    there. It walks on once the erasure has ended.
- *Falsified if:*
  - (a) the engineer stops at the edge of the cell holding the docked vehicle while its route leads
    into that cell, and stays there,
  - (b) the engineer walks into the vehicle's cell while the vehicle is being erased.

  If the engineer is taken in before its route reaches the vehicle's cell, the vehicle leaves the
  depot first, or (in b) the erasure does not overlap the engineer's arrival, the run cannot decide.
  Part (b) is then repeated with the eraser's order given earlier.

**Scenario C18: an override hides a blocker listed after it (claims 4, 13, Edge cases).**
- *Setup:* clear ground. The map places an idle transport *T* of the human player, with room for an
  infantry unit, in a cell *X*, and also places in *X* a building of the human player of a stock type
  whose foundation is one cell and that is neither a gate nor invisible in play. Units are placed
  before buildings, and a building is listed after every unit in any case (claim 13). An unarmed
  infantry *W* of the human player stands 4 cells from *X*.
- *Procedure:* give *W* an enter order on *T*, and record until *W* is inside *T* or 300 ticks.
- *Observable:* at the start, whether *T* and the building both stand in *X*. Then whether *W*
  walks into *X* and boards *T*, or stops next to *X*.
- *Runs:* 2.
- *Expected:* *T* is listed first and overrides as *W*'s mission target. The building, listed after
  it, gives no value, so *W* walks into *X* and boards *T*.
- *Falsified if:* *W* stops next to *X* and does not board *T* while both stand in *X* (this is what
  `quirks.occupancy_override_hides_later_blockers` off would do: the allied building gives
  *blocked*). If the map does not place both objects in *X*, or *T* moves away, the run cannot
  decide.

**Scenario C19: the asking tick (claims 6 to 9, 11).** These parts time each reaction against the
walker's own pauses at free cell boundaries, so they do not depend on which tick S-0208 counts as
the asking tick.
- *Common setup:* clear flat ground. A one-cell-wide corridor runs map east to a cell *X*; *P* is
  the corridor cell just west of *X*. An infantry *W* of the human player, of an ordinary walking
  stock type, is pre-placed at least 5 cells west of *P* at the right position of its cell (map value
  2, S-0208 claim 5). It is given a move order to a cell at least 4 cells past *X*, farther than the
  close-enough distance from *P*. Under a move order *W* does not engage units on its own.
  - *t*ₐ is the tick on which *W* arrives on its standing position in *P*: the first tick on which
    its recorded position equals that position.
  - *g*₀ is, at each free cell boundary that *W* crosses before *P*, the number of ticks from its
    arrival on a standing position to the next tick on which its position changes. A run counts
    only if *g*₀ is the same at every such boundary (as S-0208 claim 6 has it), and it uses that
    value. Then *t*ₐ + *g*₀ − 1 is *W*'s asking tick in *P* (Terms).
  - *g* is the number of ticks from *t*ₐ to the next tick on which *W*'s position changes.
- *Setup:*
  - (a) A bypass leaves the corridor at *P* and rejoins it past *X*; it is 2 cells longer than the
    route through *X*. A dead-end niche cell, not part of the bypass, adjoins *X*. An unarmed
    infantry *E* of the computer player stands in the niche. *W* is unarmed. After *W* has been given
    its order (so its path leads through *X*), and while *W* is at least 3 cells from *X*, a map
    trigger moves *E* into *X*, where it stands. For *W*, *X* then gives *blocked* (claim 4), so the
    new search leaves *X* out.
  - (b) No bypass: *X* is on the only route. *E* is pre-placed in *X* by the map. *W* is of an armed
    type, so *X* gives it *enemy obstacle* (claim 4, row WR3) and its new path again leads
    through *X*.
  - (c) As (b), with three idle infantry of the human player pre-placed on the right, left and bottom
    positions of *X* (map values 2, 3 and 4) instead of *E*. The corridor cells past *X* are free.
    *X* gives *W* *stationary friendly*, and *W* is farther than the close-enough distance from its
    destination, so it sends the cell-wide request of claim 8.
  - (d) As (a), with *E* replaced by an enemy vehicle of a type that a test file lets cloak. It is
    moved from the niche into *X* by the trigger and stands there, cloaked and invisible to the
    human player, when *W* arrives. *X* gives *W* *cloaked blocker*. Include a third house with
    ordinary sight of *X* but no sensor coverage. Neither non-owning house has sensors covering *X*.
  - (d2) As (d), with *X* under a high bridge and the route and bypass at ground level. The named
    test file also lets an infantry type cloak. Put one idle infantry of *W*'s house on a side
    position of ground *X*, and one on deck *X*, both fully cloaked before *W* arrives. Keep the
    ground cell's other side positions free. The enemy ground vehicle still gives *cloaked blocker*.
    No sensor or other cause of uncloaking may affect these objects during the observation.
  - (e) Later ticks. As (d), with these changes. The cloaked vehicle is pre-placed in *X* by the
    map, so a path through *X* exists at a step cost of 1000 (S-0212 claim 8) when the order is
    given, and the test file also removes its weapons, so that it cannot fire on *W* once revealed.
    The first cell *B* of the bypass, which adjoins *P*, holds an unarmed infantry *E*₂ of the
    computer player, pre-placed by the map, so *B* gives *W* *blocked* (claim 4, row WP22) and no
    search leads through it. A dead-end niche cell *N*, not part of the bypass, adjoins *B* and no
    other cell of the corridor or the bypass. *W* reaches *P*, gets *cloaked blocker*, its new search
    again leads through *X*, and it waits (claims 7 and 9). At least 30 ticks after *t*ₐ, a map
    trigger moves *E*₂ into *N*, where it stands. *t*ₙ is the first tick on which *E*₂'s recorded
    position lies in *N*.
- *Procedure:* give *W* its order and record until it has passed *X*, or 400 ticks. In (d) and
  (d2), also record each test object's cloak state every tick and its presentation to both
  non-owning houses, using a read-only recorder and presentation capture. Compare houses in the
  same recorded skirmish state; no network game is required.
- *Observable:* *g*₀, *t*ₐ and *g*; the cells *W* moves through after *P*; in (b) the first tick on
  which *W*'s recorded target is *E*; in (c) the tick on which each occupant's recorded destination
  changes; in (d) and (d2) the first uncloaking tick of each test object and whether the cloak
  change applies to both non-owning houses; in (e) *t*ₙ
  and the first tick after *t*ₐ on which *W*'s position changes, and into which cell.
- *Runs:* 3 each.
- *Expected:*
  - (a) *W* never enters *X*. Its next position change after *t*ₐ is a step into the bypass, and
    *g* = *g*₀: the new search and the choice of the bypass cell both happen on the asking tick
    (claim 7).
  - (b) *W*'s target is first *E* on tick *t*ₐ + *g*₀ − 1: the repeated ask gives *enemy
    obstacle* again, and *W* takes *E* as its target on the same asking tick (claims 7 and 9).
  - (c) All three occupants' recorded destinations change on tick *t*ₐ + *g*₀ − 1 (claims 7, 8 and
    11).
  - (d) As (a): *g* = *g*₀ and *W* takes the bypass. The vehicle starts uncloaking on
    tick *t*ₐ + *g*₀ − 1 for every house, including the third house (claim 9).
  - (d2) On that same asking tick, both the enemy ground vehicle and the friendly ground infantry
    start uncloaking. The deck infantry stays fully cloaked. Ownership does not limit the effect,
    but the ground layer does. The route timing is not this part's observable.
  - (e) *W* stays in *P* until *E*₂ has left *B*. Its next position change is a step into *B*, on
    tick *t*ₙ + 1 or *t*ₙ + 2: on each tick it searches a new path, so the search of the first
    asking tick after *B* is vacated takes the bypass, and *W* chooses its walking target in *B* on
    that tick and steps on the next (claim 7, later ticks). Which of the two ticks it is depends on
    whether *W* takes its turn in tick *t*ₙ before or after *E*₂ (S-0203 claim 3).
- *Falsified if:*
  - (a) or (d) *W* takes the bypass with *g* > *g*₀ (a tick lost before the new search, or before
    the choice of its first cell),
  - (b) *W*'s target is first *E* on a tick after *t*ₐ + *g*₀ − 1, or *W* never takes *E* as its
    target,
  - (c) any occupant's destination does not change on tick *t*ₐ + *g*₀ − 1, while *W* stands in *P*,
  - (d) the vehicle remains fully cloaked on tick *t*ₐ + *g*₀ − 1, or the effect grants sight to
    the walker's house only rather than changing cloak state for every house,
  - (d2) either ground object does not start uncloaking on that tick, or the deck infantry does.
    If any test object is not fully cloaked beforehand, or another uncloak cause acts, the run
    cannot decide this part,
  - (e) *W* still stands in *P* on tick *t*ₙ + 30 while the vehicle stays in *X* (as if each later ask
    were a repeated ask, which waits without searching again), or its first step after *t*ₐ comes later than tick
    *t*ₙ + 2 (a later search, held back by a timer). If *W* gives up, leaves *P* before *t*ₙ, or the
    vehicle leaves *X*, the run cannot decide,
  - (a) or (d) *W* walks through *X* while *E* or the vehicle stands there (the only route past *X*
    other than the bypass).

  If *E* (or the vehicle) is not in *X* when *W* arrives in *P*, or if *W* never reaches *P*, the run
  cannot decide. In (a) or (d), if *W* stays in *P* and never takes the bypass, the run cannot decide
  the timing, because the choice of route belongs to S-0212.


**Scenario C20: vehicle movers (claim 5, Terms *cloaked* and *armed*, Edge cases).** Parts (a), (d), (e) and (g) to (k) tell
the outcomes apart by the route a vehicle's path search chooses, using the step costs of S-0212
claim 8: *clear* 1, *stationary friendly* 8, *enemy obstacle* 20, *friendly obstacle* 60, *cloaked
blocker* 1000, and no path through a *blocked* cell. A *moving blocker* whose blocker chain ends at
a unit with zero speed and no stored path also costs 1 (S-0212 claim 8). Each of these parts, except
(g2), has a one-cell-wide corridor through a cell *X*, and a bypass that leaves the corridor before
*X* and rejoins it after *X*. Its length is given as the number of cells it adds to the route
through *X*, counting straight steps only. Where a building stands at *X*, the corridor passes
through one cell *X* of its foundation, and the rest of the foundation lies outside the corridor,
among its impassable side cells, and *X* is not in the last column along x of a building whose
type has the bib flag (a test file removes the flag otherwise). As in S-0212 scenario P13, the
mover is placed by the map in a strip of legal cells outside the usable area, so that its search
runs without a corridor (S-0212 claim 14) and its route depends on the cell costs alone; the
bypass margins below exceed the 3 cost units that P13 allows for the direction terms and the
estimate. The vehicle is of an ordinary driving stock type that is not a crusher and is not a
train (except in part e3), owned by the human player, and is ordered to a corridor cell past *X* from at least 6 cells
before the bypass. Rows VP8, VP9 and VP13 of claim 5, backed by C8 (a), C8 (d) and C10 "with a
vehicle", are tested by running those scenarios again with such a vehicle in place of the walker:
their alternatives differ in whether the vehicle enters the cell (it opens a closed friendly gate
and passes, it passes an open gate, and it passes a building that is invisible in play). Row VP22,
backed by C5 (a) "with a vehicle", is tested in the same way by scenario C5 (a), with such a vehicle in place of the
infantry and no side passage: the vehicle waits behind *V*₁ and sends it no request (*V*₁'s
recorded destination does not change while the vehicle waits), which a *stationary friendly* would
make it send (claim 12). What the vehicle does instead of entering belongs
to S-0212 claim 22 (moving blocker) and S-0207 claim 15.
- *Setup:*
  - (a1) *X* holds one idle infantry of the human player, placed by the map. Bypass 3 cells longer.
  - (a2) As (a1), with a bypass 20 cells longer.
  - (b) No bypass. A side corridor crosses the corridor at *X*. A map trigger walks an infantry of
    the human player along the side corridor through *X* and on, timed so that it is in *X* when
    the vehicle reaches the cell before *X*. The start times are varied over 20 runs.
  - (c) Two vehicles of a stock type with the small-visceroid flag, both of the human player
    (placed by the map), stand idle in neighbouring cells. One is ordered into the other's cell
    with force-move held. Control (c′): the same with two idle vehicles of an ordinary stock type.
  - (d1) *X* holds an aircraft of the human player resting on the ground (as in C15 (b)). Bypass 3
    cells longer.
  - (d2) As (d1), with a bypass 20 cells longer.
  - (e1) *X* holds an idle vehicle of the computer player of a stock type that the mover cannot
    crush. The mover is armed. Bypass 30 cells longer.
  - (e2) As (e1), with an unarmed mover.
  - (e3) As (e2), with a mover that is a train, an exception to the common setup: a test file
    gives the train flag (S-0212 Terms) to an ordinary driving stock type without any weapon that
    is not a crusher, as in scenario C26 (f), and the mover is of that type. The vehicle in *X* is
    of a stock type that is not a train. The mover's first step leads straight along the corridor,
    so that the train restriction of S-0212 (claim 16) does not change its route.
  - (e4) As (e1), with a test file that sets the damage field and the ambient-damage field of the
    mover's primary weapon to 0 (its warhead and projectile stay as they are).
  - (e5) As (e2), with the mover made elite by the map, and a test file that gives its type an
    elite primary weapon and leaves it without an ordinary primary weapon.
  - (f) No bypass. The overlay type with registration index 0 in the stock data is read when the
    map is built. A section of it, owned by the human player, closes the corridor at *X*. The mover
    is a stock vehicle type with the crusher flag. Control (f′): as (f), with the section of another
    crushable wall type (registered later; a test file makes a wall type crushable if no stock one
    is).
  - (f2) As (f′), with the crusher owned by the computer player and moved along the corridor by a
    map trigger, so that the section, built by the human player, is an enemy's for it.
  - (f3) As (f) and (f′), with a stock walking vehicle type, given the crusher flag by a test file
    if its type lacks it.
  - (g1) A test file lets an enemy vehicle type cloak (as in C3 (f)). One such vehicle of the
    computer player stands idle and cloaked in *X*, invisible to the human player, when the order is
    given. The mover is armed. Bypass 30 cells longer.
  - (g2) As (g1), with no bypass: *X* is on the only route.
  - (g3) As (g1), with a test file that gives a stock unit type of the human player the sensor
    ability, with a sensor range that covers *X*. One such unit stands idle within that range, away
    from the corridor and the bypass, so that the cloaked vehicle is visible to the human player
    when the order is given.
  - (h1) *X* is a foundation cell of a closed gate of the computer player, of a stock gate type.
    The mover has a primary weapon. Bypass 30 cells longer.
  - (h2) As (h1), with a mover of a stock type without a primary weapon.
  - (i1) *X* is a foundation cell of a building of the computer player, of a stock type marked as a
    bridge repair hut. The mover has a primary weapon. Bypass 30 cells longer.
  - (i2) As (i1), with a stock building of the computer player of a type that has no weapon and is
    neither a gate, nor a bridge repair hut, nor invisible in play (as in C15 (d)).
  - (j1) A wall section built by the human player, of a wall type that is not crushable, closes the
    corridor at *X*. A test file gives the mover's type the destroyer movement zone, which may cross
    wall-like cells (S-0211 claim 12). The warhead of the mover's primary weapon can damage walls
    (a test file gives it that property if the stock type's warhead lacks it). Bypass 40 cells
    longer.
  - (j2) As (j1), with the mover owned by the computer player and moved along the corridor by a map
    trigger, so that the wall is an enemy's for it.
  - (j3) As (j2), with a test file that removes the wall-damaging property from the warhead of the
    mover's primary weapon, and that gives the wall's overlay type an armour other than wood if its
    stock armour is wood.
  - (j4) As (j3), with a test file that sets the wall's overlay armour to wood and lets the warhead
    of the mover's primary weapon damage wood.
  - (k1) *X* holds a terrain object placed by the map, of a stock type whose side-position pattern
    for the map's theater is 7 (a test file sets it to 7 otherwise), so that *X* is wall-like
    (S-0211 claim 11). The mover has the destroyer movement zone and a primary weapon, as in (j1),
    and a test file makes the warheads of all its weapons able to damage wood. Bypass 30 cells
    longer.
  - (k2) As (k1), with a test file that makes none of the mover's warheads able to damage wood.
- *Procedure:* give the order and record the cells the mover's position enters for 400 ticks, or,
  in (c), until the mover stops. In (g2), also record the mover's destination on every tick.
- *Observable:* whether the mover's route leads into *X* or into the bypass; in (b), on the
  recorder, whether the mover's position is in *X* on a tick on which the infantry's is; in (c),
  whether the mover's position enters the other vehicle's cell; in (f), whether the mover enters
  *X*; in (g2), whether the mover sets out along the corridor, and whether its destination is
  cleared.
- *Runs:* 2 each; (b) 20.
- *Expected:*
  - (a1) The route avoids *X* from the start (*stationary friendly*, 8, against 3 extra cells).
  - (a2) The route leads towards *X* (8 is less than 20 extra cells).
  - (b) The mover is never in *X* while the infantry is (*moving blocker* through the record).
  - (c) The mover drives into the other visceroid's cell. In (c′) it stops at the cell's edge.
    What follows the meeting of two visceroids belongs to other specs.
  - (d1) As (a1). (d2) As (a2): an idle allied aircraft gives *stationary friendly*, not *blocked*.
  - (e1) The route leads towards *X* (*enemy obstacle*, 20, against 30 extra cells).
  - (e2) The route takes the bypass (*blocked*).
  - (e3) The route leads towards *X*: the train counts as armed (Terms *armed*), so row VP30 gives
    *enemy obstacle*, which costs a train 1 (S-0212 claim 8).
  - (e4) As (e1): the vehicle's test of *armed* asks for a primary weapon, not for its damage.
  - (e5) As (e1): the elite mover has a primary weapon (Terms *armed*), so row VP30 gives *enemy
    obstacle*.
  - (f) The mover drives into *X* (and crushes the section, S-0210). In (f′) it sets out, drives up
    to the cell before *X*, and does not drive into *X* while the section stands: the section gives
    *friendly obstacle*, so a path through *X* exists at a step cost of 60 (no bypass), and the start
    of the step into *X* is not cleared for it. What it does in front of *X* belongs to S-0207.
  - (f3) As (f) and (f′).
  - (f2) The mover drives into *X* without stopping in front of it, and crushes the section: an
    enemy section that it crushes gives no value.
  - (g1) The route takes the bypass (*cloaked blocker*, 1000, against 30 extra cells).
  - (g3) As (g1): the route takes the bypass, although the human player sees the vehicle.
  - (g2) The mover sets out along the corridor towards *X* and keeps its destination until it
    reaches the cell before *X*: a path through a cell that costs 1000 exists. What it does there
    belongs to S-0207.
  - (h1) The route leads towards *X* (*enemy obstacle*). (h2) The route takes the bypass
    (*blocked*).
  - (i1) The route takes the bypass (*blocked*). (i2) The route leads towards *X* (*enemy
    obstacle*).
  - (j1) The route takes the bypass (*friendly obstacle*, 60, against 40 extra cells). (j2) The
    route leads towards *X* (*enemy obstacle*, 20).
  - (j3) The route takes the bypass (*blocked*, row VO6). (j4) The route leads towards *X* (*enemy
    obstacle* through the wood case of row VO5).
  - (k1) The route leads towards *X* (*enemy obstacle*). (k2) The route takes the bypass
    (*blocked*).
- *Falsified if:*
  - (a1) the mover drives straight to the cell before *X* (this is what an idle allied infantry
    unit giving *moving blocker* would do: cost 1),
  - (a2) or (d2) the mover takes the bypass from the start (as *blocked* would),
  - (b) the mover is in *X* on a tick on which the infantry is. A run in which the infantry is not
    in *X* when the mover reaches the cell before it cannot decide,
  - (c) the visceroid stops at the edge of the other's cell and stays, in a pair of runs in which
    (c′) stopped at the edge. If the force-move order is refused, the part cannot decide,
  - (d1) the mover drives straight to the cell before *X*,
  - (e1) the mover takes the bypass, or (e2) its route leads into *X*,
  - (e3) the route takes the bypass (as row VP26, *blocked*, would make it if a train without a
    weapon were not armed). If (e2) did not take the bypass, or the mover does not set out, the part
    cannot decide,
  - (e4) the route takes the bypass (as row VP26, *blocked*, would make it if the test asked for
    a positive damage). If (e1) did not lead towards *X*, the part cannot decide,
  - (e5) the route takes the bypass, as in (e2) (the ordinary primary slot alone asked). If (e1)
    did not lead towards *X*, or (e2) did not take the bypass, the part cannot decide,
  - (f) the mover does not drive into *X* while the control (f′) does not either (this is what
    `quirks.crusher_overlay_index0_clear` off would do). If the mover in (f′) drives through, or the
    stock overlay type with registration index 0 is not a wall type that the mover crushes, the
    part cannot decide,
  - (f′) the mover does not set out, or its destination is cleared before it reaches the cell
    before *X* (as *blocked* would make it: no path, S-0212 claim 23),
  - (f3) as (f) and (f′), for the walking vehicle,
  - (f2) the mover stops in front of *X* and does not drive in while the section stands, in a pair
    of runs in which (f′) stopped too. If the mover in (f′) drives through, the part cannot decide,
  - (g3) the route leads towards *X* (as *enemy obstacle* would, were the cloak state disregarded
    for a house that sees the vehicle). If the vehicle is not visible to the human player when the
    order is given, or (g1) did not take the bypass, the part cannot decide,
  - (g1) the route leads towards *X* (as *enemy obstacle* would), or (g2) the mover does not set
    out, or its destination is cleared before it reaches the cell before *X* (no path: *blocked*,
    S-0212 claim 23). If the cloaked vehicle is visible to the human player or moves before the
    mover reaches *X*, the run cannot decide,
  - (h1) the mover takes the bypass, or (h2) its route leads into *X*. If the gate opens for the
    mover, the run cannot decide,
  - (i1) the route leads into *X*, or (i2) the mover takes the bypass,
  - (j1) the route leads into *X*, or (j2) the mover takes the bypass. If the test file's zone does
    not let the mover's search cross wall-like cells at all (both parts take the bypass), the parts
    cannot decide,
  - (j3) the route leads into *X*, or (j4) the mover takes the bypass. If (j2) did not lead into
    *X*, (j3) and (j4) cannot decide,
  - (k1) the mover takes the bypass, or (k2) its route leads into *X*. If both parts take the
    bypass, the zone is in doubt as in (j), and the parts cannot decide.

**Scenario C21: entry outcomes read by the pointer (claim 14; claim 5, row VP1).** Both parts read
the cursor that S-0214 table GC gives for a cell, whose row GC9 shows *move* when the actor's entry
outcome for the cell is *clear* or *cloaked blocker*, and row GC11 *no-move* otherwise.
- *Setup:*
  - (a) Clear ground with a deep-water cell *W* and a cell *K* in the middle of a cliff, both
    inside the usable area and visible to the human player. An idle aircraft of the human player of
    a stock type that takes move orders, landed on open ground, and in a second run an idle tank of
    the human player instead.
  - (b) Clear flat ground. An idle vehicle *M* of the human player, of an ordinary driving stock
    type without a weapon, stands alone in a cell *Y*. A second idle vehicle *N* of the same type
    and owner stands alone in a cell *Y*′ at least 3 cells away.
- *Procedure:* (a) with the unit selected, hold the pointer over *W*, then over *K*, without modifier
  keys, and click on each. (b) With *M* selected, hold the pointer, without modifier keys, over a
  point of *Y* that *M*'s picture does not cover, then over a point of *Y*′ that *N*'s picture does
  not cover. Scripted input may place the pointer.
- *Observable:* the cursor shown over each cell, and in (a) whether the click gives the unit a move
  order to that cell (on the recorder, its destination).
- *Runs:* 1 each.
- *Expected:*
  - (a) The aircraft shows the move cursor over both cells and gets each cell as its destination:
    its entry outcome is *clear* (claim 14), so S-0214 row GC9 gives move. The tank shows the
    no-move cursor (its terrain checks give *blocked*).
  - (b) Over *Y* the cursor is move: *M* gives itself no value and it is excused from its own
    vehicle-present flag (row VP1), so its entry outcome for its own cell is *clear*. Over *Y*′ it is
    no-move: *N* gives *stationary friendly* (row VP20).
- *Falsified if:*
  - (a) the aircraft shows the no-move cursor over either cell, or gets no move order there, while
    the tank shows no-move. If the tank shows move, or S-0214 resolves an aircraft's click on such a
    cell by a row before GC9, the part cannot decide,
  - (b) the cursor over *Y* is no-move while the cursor over *Y*′ is no-move too. If the cursor over
    *Y*′ is move, or no point of *Y* is free of *M*'s picture, the part cannot decide.

**Teleporting-vehicle companion measurement (claims 1 and 5).** S-0207 scenario D22,
run 13 supplies the setup, procedure, recorder, repeats and falsification controls for the
pending ground reservation, ground/deck entry probes, two-record placement, replacement clearing
and next-jump deck clearing. Its pre-jump probes qualify only when the recorder captures that
interval; ordinary input alone does not establish a delay before the jump. No measurement has
been run for these claims.

**Teleporting-infantry companion measurement (claim 1; Q-0311).** S-0208 scenario
W11 part i supplies the named test map, teleporting infantry, speed, houses, recorder,
procedure, repeats and falsifiers. Record ground and deck standing-position entries,
infantry houses and listings before and after deck landing, then after a successful
next destination and jump. The expected distinction is retained ground reservation,
no landing-time deck mark, and later clearing by actual deck position. Its ground
walking probe tests the residual taken position. For a supplementary shared-deck-mark
probe, use the same setup and repeat three times: after the ground-height landing,
walk an ordinary allied infantry onto the same deck standing position, qualifying
only runs where the recorder confirms that position is marked by the walker. Give
the teleporting infantry its next successful destination after phasing in. Record
both records, positions and listings through destination choice. Expect the deck
entry to clear although the ordinary walker remains there, while ground stays taken.
A deck entry that remains marked falsifies that clearing consequence. Void any run
where another mark writer acts there or the walker leaves before the check. Use
read-only recording and scripted normal input only. No measurement has been run here.

**Scenario C22: cell layers at a high bridge (Terms *cell layer*, claims 4 and 5 parts 4a and 5a,
Edge cases).**
- *Setup:* a purpose-made map with a high bridge, at least 6 cells long, over flat clear ground of
  one height level. *X* is a cell in the middle of the bridge. A one-cell-wide ground corridor,
  walled by impassable cells, crosses under the bridge through *X*, so that one route runs along the
  deck through *X* and another runs on the ground through *X* under the deck. Neither route has
  another way past *X* within 20 cells.
  - (a) An idle, unarmed infantry of the computer player stands on the ground in *X*. An unarmed
    infantry *W* of the human player is ordered along the deck, through the deck of *X*, to the far
    end of the bridge.
  - (b) An idle, unarmed infantry of the computer player stands on the deck in *X*. *W* is ordered
    along the ground route, through *X* under the bridge.
  - (c) A stock vehicle that can crush is ordered along the deck through *X*. An idle enemy vehicle
    of a stock type it can crush (as in scenario C3 (d2)) stands on the deck in *X*, and nothing
    stands under the bridge in *X*. Control (c′): as (c), with a second idle enemy vehicle of a type
    the crusher can crush standing on the ground under the bridge in *X*.
  - (d) A separate map: a river of deep water, at least 4 cells wide, crossed by a high bridge. A
    test file, as the override of S-0211 scenario L6, gives the high bridge's overlay types the
    land type *clear* and turns their own-land-type flag off, so that the bridge's cells keep the
    water land type of their ground (S-0211 claim 9). An infantry of the human player, and in a
    second run an ordinary vehicle of the human player whose type has no restricted land type
    (S-0211 claim 14), is ordered from one bank across the bridge to the other bank.
- *Procedure:* give the order and record the mover's position on every tick, for 400 ticks.
- *Observable:* whether the mover's position enters *X* on its level, and, in (c) and (c′), whether
  the crusher stops in front of *X* (its position unchanged for at least one tick there) before it
  enters *X*.
- *Runs:* 2 each.
- *Expected:*
  - (a) and (b) *W* walks through *X* without stopping in front of it: the occupant on the other
    cell layer does not count.
  - (c) The crusher does not drive straight into *X*: it gets *moving blocker* (Edge cases,
    `quirks.crusher_entry_ground_layer`), and what it does then belongs to S-0207. (c′) It drives
    into *X* without stopping in front of it, as in scenario C3 (d2).
  - (d) Both movers cross the bridge to the other bank: the zero speed checks of walkers and
    vehicles on water are not applied where the deck layer counts.
- *Falsified if:*
  - (a) or (b) *W* stops in front of *X* or goes around it while the occupant on the other cell layer stands
    there,
  - (c) the crusher drives into *X* without stopping in front of it, in a pair of runs in which it
    did so in (c′) (this is what `quirks.crusher_entry_ground_layer` off would do). If the crusher
    in (c′) also stops in front of *X*, or the vehicle on the deck moves away first, the part cannot
    decide. The vehicle's check at the start of a step reads the deck layer by the rule of Terms
    (*cell layer*). When parts a and b show that the cell layers are not kept apart, (c) cannot decide
    either,
  - (d) either mover stops at the bank, or at the edge of the first deck cell, or does not set out.
    If a ship of S-0211 scenario L6's kind cannot pass under the bridge on this map (so that the
    bridge's cells are not water), (d) cannot decide.

**C22 extensions for claim 15 (Q-0560).**
- *Setup:* the same stock walking infantry and high bridge over flat ground, with banks at deck
  level, human owner and one distant bystander computer player. Game speed index 2, no engine add-on and no mods
  (the named test files of a scenario excepted). Keep the bridge, banks and their neighbouring cells as the observed area. In (e),
  order a walk from one bank across the bridge to the other, then reverse it. In (f), use map
  placement at different side positions and nearby allied infantry to make the straight line to
  a diagonal target pass through a third cell; test a third cell carrying a high bridge and one
  without one. Accept a run only when the recorded line actually enters that third cell.
- *Procedure:* record every tick from the asking tick through arrival, for two runs per direction
  and third-cell case. Use scripted normal input if needed. Void a run if an unnamed bystander
  object reaches the observed area.
- *Observable:* horizontal cell, world height, walking target, cell-layer membership, the taken
  standing positions and infantry house of both records in every crossed cell. A read-only
  recorder is suitable; no state is written into the game.
- *Expected:* (e) the asking tick changes the reservation but not membership. The first boundary
  crossing onto the bridge changes membership to deck, and the first crossing off it changes
  membership to ground, before arrival. (f) every third-cell crossing follows claim 15's table,
  not the walking target's surface. On the flat-ground route, target choice reads deck when
  the walker is at deck height above the desired point's ground and marks that deck record
  immediately; crossing alone transfers no mark. These runs do not isolate the exact
  312-lepton threshold or its equality case.
- *Falsified if:* membership changes at target choice or only at arrival; a third-cell crossing
  uses the target's layer instead of the table; the flat-ground deck-height choice marks ground
  instead of deck or fails to mark deck on the asking tick; or reservation and listing are always
  forced to the same surface. If no valid third-cell route is obtained, that sub-test is inconclusive.
- *Conditional uneven-ground probe:* if a valid high bridge spanning a one-level ground change is
  established, repeat (e) in both directions across that change, recording the target-surface
  choice as well as the entry check's record. A flat departure deck 416 leptons above its ground
  reserves ground when entering the one-level-higher-ground cell, but deck when entering the
  one-level-lower-ground cell; listing stays deck in both. The opposite choice falsifies the
  corresponding conditional example. This is a proposed protocol, not evidence that such a map
  is valid.

**Scenario C23: building cells a mover may cross (claim 4, row WP10; claim 5, rows VP12, VP15 and
VP16; Terms *blocking cell*).** Every building
here belongs to the human player, and every mover is of the human player. Walkers are of an
ordinary walking stock type. Vehicles are of an ordinary driving stock type that is not a crusher
and is not a train, at full health, and are given plain move orders. Corridors and bypasses are as
in scenario C20, with a bypass 10 cells longer than the route through the building. The foundation
sizes and the type flags used are read from the stock data when the map is built. For a building of
*W* × *H* cells, *column i* is the set of its foundation cells *i* cells along x from its top-left
cell. A corridor *through column i* runs along y and crosses the foundation through every cell of
that column; the foundation cells beside the column are not part of the corridor. Except in part
(e), a test file removes the bib flag from every building type used, so that the bib row of claim 5
cannot decide in place of the row under test.
- *Setup:*
  - (a) A test file gives the firestorm-wall flag to a stock building type that is neither a gate
    nor invisible in play. One such building closes a one-cell-wide corridor at a foundation cell
    *X*, with no other route (the rest of the foundation outside the corridor, as in C20). No
    house's firestorm is on. A walker is ordered to a cell past *X*. Control (a′): the same without
    the test file.
  - (b) As (a), with a vehicle as the mover and the bypass. Control (b′): the same without the test
    file.
  - (c1) A building of a stock type on which vehicles are repaired, with *W* ≥ 2. A test file sets
    its impassable-rows value to 1, so that only column 0 holds blocking cells. The vehicle's
    corridor runs through column *W* − 1.
  - (c2) As (c1), with the corridor through column 0.
  - (d1) A building of a stock bunker type, with no unit in it. A test file sets its impassable-rows
    value to 0, so that it has no blocking cells while it is empty. The vehicle's corridor runs
    through column *W* − 1.
  - (d2) As (d1), after a vehicle of the human player has been ordered into the bunker and sits in
    it, so that every foundation cell is a blocking cell.
  - (e1) A building of a stock type with the bib flag (a test file gives the flag to a stock type
    if none has it), that is neither a gate, nor a repair type, nor a bunker type, nor invisible in
    play, with *W* ≥ 2. The vehicle's corridor runs through column *W* − 1.
  - (e2) As (e1), with the corridor through column *W* − 2.
  - (e′) Control: as (e1), with a test file that removes the bib flag.
- *Procedure:* give the order and record the cells the mover's position enters for 400 ticks.
- *Observable:* whether the mover's position enters a foundation cell, and which; whether its route
  leads through the foundation or into the bypass.
- *Runs:* 2 each.
- *Expected:*
  - (a) The walker walks through *X*: the firestorm wall gives no value (claim 4). (a′) The walker
    never enters *X* (*blocked*, as in C15 (a)).
  - (b) The vehicle's route leads through *X* (*clear*). (b′) It takes the bypass (*blocked*).
  - (c1) The route leads through column *W* − 1 (no value: not a blocking cell). (c2) The route
    takes the bypass (column 0 holds blocking cells, so the allied building gives *blocked*). If
    the count ran along y instead, (c1) would take the bypass too, since its corridor crosses the
    first row.
  - (d1) The route leads through the bunker's foundation. (d2) The route takes the bypass.
  - (e1) The route leads through column *W* − 1: the bib row gives no value there. (e2) and (e′)
    The route takes the bypass (the allied building gives *blocked*).
- *Falsified if:*
  - (a) the walker stops in front of *X*, or does not set out, and the walker in (a′) is as expected. If the
    walker in (a′) walks through *X* too, the part cannot decide,
  - (b) the vehicle takes the bypass, and in (b′) it takes it too. If the vehicle in (b′) drives
    through, the part cannot decide,
  - (c1) the vehicle takes the bypass, or enters a cell of column 0; (c2) its route leads into
    column 0. If the vehicle docks at the building or is repaired on the way, the run cannot
    decide,
  - (d1) the vehicle takes the bypass, or (d2) its route leads into the foundation. If the mover
    is taken into the bunker, the run cannot decide,
  - (e1) the vehicle takes the bypass and in (e′) it does too, or (e2) its route leads through column
    *W* − 2. If the vehicle in (e′) drives through column *W* − 1, the part cannot decide.

**Scenario C24: behaviour relations of a vehicle mover (claim 5, rows VP3, VP5 to VP7, VP17 and VP18,
and the override case of part 5a).** Every unit and building here belongs to the human player,
unless stated. The recorder also reads each unit's behaviour, destination, guard spot and contact.
- *Setup:*
  - (a) The human player plays a house with a stock building type that grinds units (a recycling
    building, S-0214). One such building stands on clear ground. An ordinary vehicle 6 cells from
    its foundation is given the action on the building that S-0214 (table V, value *n*, row n2)
    makes *recycle*, so that its behaviour is *eaten* and its destination the building.
  - (a2) As (a), with a grinder whose foundation is at least 2 cells wide along x. The map places an
    ordinary vehicle *R* of the human player on a foundation cell *F*₁ of the grinder (units are
    placed before buildings), and a one-cell-wide corridor of the scenario C23 kind runs from the
    neighbouring foundation cell *F*₂, the only other foundation cell in the corridor, on to a
    cell *Z* outside the foundation, with a bypass 10 cells longer around *F*₂. *R* is given a
    plain move to *Z*. Control (a2′): the same move given to a vehicle that stands outside the
    foundation, on the corridor before *F*₂.
  - (b) A test file gives the absorb-units flag to a stock building type with a foundation of at
    least 2 × 2 cells. An ordinary vehicle 6 cells from one such building is given the action on it
    that S-0214 (table V, value *n*, row n3) makes *enter*.
  - (c1) A stock harvester and a stock refinery, on clear ground beside a small resource field. The
    harvester harvests and returns to the refinery to unload. In (c1) to (c3) a test file removes
    the bib flag from the refinery type, if it has it, so that the bib row cannot decide.
  - (c2) As (c1), with a test file that sets the refinery type's impassable-rows value to its
    foundation width *W* along x, so that every foundation cell is a blocking cell.
  - (c3) Control, on a map built after (c1) has been run: an ordinary vehicle that is not in
    contact with the refinery is given a plain move along a corridor through the foundation cells
    that the harvester entered in (c1), with a bypass as in scenario C23.
  - (d) A stock transport type that carries vehicles (read from the stock data) stands idle on clear
    ground. An ordinary vehicle that fits in it, 5 cells away, is given the action on it that S-0214
    (table V, value *q*, row q2) makes *enter*.
  - (d2) As (d), with the transport placed by the map on a cell of a land type *L*, and a test file
    that sets *L*'s speed percentage for the mover's locomotion class to 0. *L* is chosen from the
    stock land types when the map is built, so that only the transport's cell and one neighbouring
    cell *E* have it. Control (d2′): on the same map, the vehicle is given a plain move to *E*.
  - (e) As scenario C11, with an armed vehicle *G* in place of the walker *W*: *G* is ordered to
    area-guard the idle vehicle *V* with the gesture of S-0214 table G, row G6, as S-0213 scenario
    O3 gives it to a vehicle in its step 3. Control (e′): *G* is given a plain move order to the
    corridor cell just past *V*.
- *Procedure:* give the order and record until the order is carried out, or 600 ticks; in (c1) and
  (c2), until the harvester has unloaded twice, or 1500 ticks.
- *Observable:* the cells the mover's position enters; whether it is taken in (a, b), boards (d) or
  unloads (c); in (c), the ticks on which the recorder shows the harvester in contact with the
  refinery; in (e), whether *V* moves away from its cell.
- *Runs:* 2 each.
- *Expected:*
  - (a) The vehicle's position enters a foundation cell of the grinder before it is taken in: row
    VP6 overrides with *clear*.
  - (a2) *R*'s route leads through *F*₂: the grinder is listed in *R*'s own cell, so row VP7
    overrides with *clear*. (a2′) The control vehicle's route takes the bypass (row VP19, *blocked*).
  - (b) The vehicle's position enters a foundation cell of the absorbing building before it is
    taken in.
  - (c1) While in contact, the harvester's position enters only foundation cells of the refinery
    that are not blocking cells (by the stock impassable-rows value). (c2) Its position never
    enters a foundation cell. (c3) The control vehicle takes the bypass (*blocked*: without contact
    row VP19 applies).
  - (d) The vehicle drives into the transport's cell and boards it: row VP17 overrides with *clear*.
  - (d2) As (d): an outcome decided by an override is exempt from the zero speed check (part 5a).
    (d2′) The vehicle never enters *E* (*blocked* by the zero speed check).
  - (e) *V* does not move: row VP18 gives *blocked*, so *G* sends no request. (e′) *G*
    stops next to *V*'s cell and sends a request (*stationary friendly*, claim 12); whether *V*
    then moves aside belongs to S-0207.
- *Falsified if:*
  - (a) or (b) the vehicle stops at the edge of the foundation and stays there without being taken
    in (as row VP19, *blocked*, would make it). If the recorded behaviour and
    destination are not *eaten* and the grinder in (a), or *enter* and the building in (b), or if
    the vehicle is taken in from a cell outside the foundation, the part cannot decide,
  - (a2) *R*'s route takes the bypass, in a pair of runs in which the control took it too. If the
    map does not place *R* on *F*₁, if *R* is taken in before it moves, or if the control's route
    leads through *F*₂, the part cannot decide,
  - (c1) the harvester's position enters a blocking cell while in contact, or (c2) its position
    enters a foundation cell while in contact, in a pair of runs in which the control (c3) took the
    bypass. If the harvester in (c1) never enters a foundation cell, or never shows contact, the
    part cannot decide,
  - (d) the vehicle stops at the edge of the transport's cell and stays there without boarding. If
    the order is not *enter* on the recorder, or the vehicle boards from a neighbouring cell, the
    part cannot decide,
  - (d2) the vehicle stops at the edge of the transport's cell and stays there without boarding, in
    a pair of runs in which it did not enter *E* in (d2′). If it never comes within one cell of the
    transport (because the test file changed which cells its path search can reach, S-0211 claims
    11 and 12), or if it enters *E* in (d2′), the part cannot decide,
  - (e) *V* moves away while *G* stands next to its cell, in a pair of runs where *V* moved aside in
    (e′). If *V* does not move in (e′), if *G* in (e) never reaches the cell next to *V*, or if *G*'s
    recorded behaviour, destination and guard spot in (e) are not *area guard*, *V* and *V*, the
    part cannot decide.

**Scenario C25: the view of an alliance (Terms *allied*; claims 4 and 5).**
- *Setup:* the game settings allow alliances to change during play. A computer house *K* is
  hostile to the human house *H* at the start. Early in the game, the human player gives the
  alliance command towards *K*. *K*'s objects in parts a to c are placed by the map and are not
  armed, so that *K* attacks nothing near them. Each part uses a one-cell-wide corridor; walkers and vehicles
  of *H* are of rookie rank and are ordered to a cell past *X*, farther than the close-enough
  distance from the cell in front of *X*.
  - (a) *X* is a foundation cell of a closed gate of *K*, of a stock gate type, on the only route.
    The mover is an armed infantry of *H* whose type has a primary weapon.
  - (a2) As (a), with an armed vehicle of *H* that has a primary weapon.
  - (b) An idle, unarmed infantry of *K* stands in *X*, on the only route. The mover is an armed
    infantry of *H*.
  - (c) An idle, unarmed vehicle of *K* stands in *X*. The mover is an armed vehicle of *H*, of a
    type that cannot crush, placed as in scenario C20, with a bypass 12 cells longer than the route
    through *X*.
  - (d) As scenario C3 (c6): a wall section built by *H* closes the corridor at *X*, with the bypass
    of C3 (c5), 40 cells longer. The walker belongs to *K*, has the infantry-destroyer movement
    zone and a primary weapon that can damage walls (test file as in C3), and is moved along the
    corridor by a map trigger.
  - (d2) As scenario C20 (j2), with the vehicle owned by *K* and the wall section built by *H*:
    the vehicle has the destroyer movement zone and a primary weapon whose warhead can damage
    walls (test files as in C20 (j1)), and is moved along the corridor by a map trigger, with a
    bypass 40 cells longer.
- *Procedure:* give the orders, or fire the trigger, after the alliance command, and record for 400
  ticks.
- *Observable:* whether the gate opens and the mover passes, and whether the mover takes the gate
  as its target (a, a2); whether the walker walks into *X* (b); whether
  the mover's route leads towards *X* or into the bypass (c, d, d2).
- *Runs:* 2 each.
- *Expected:*
  - (a) *K* is not allied with *H*, so the gate owner's view gives an enemy gate (*enemy
    obstacle*): the gate does not open for the walker, which stops in front of it. *H* is allied
    with *K*, so claim 9's attack test, in the walker's view, finds the gate's house allied, and the
    walker takes no target.
  - (a2) The vehicle does not pass the closed gate.
  - (b) *H* is allied with *K*, so the walker's view makes *K*'s infantry allied: the cell gives
    *clear* (one side position taken), and the walker walks into *X* without taking a target.
  - (c) The idle vehicle is allied in the mover's view and gives *stationary friendly* (cost 8,
    less than 12 extra cells): the route leads towards *X*.
  - (d) *K* is not allied with *H*, so the wall is an enemy's for the walker (*enemy obstacle*, cost
    20): the route leads to the wall, as in C3 (c6).
  - (d2) As (d), for the vehicle: the route leads to the wall, as in C20 (j2).
- *Falsified if:*
  - (a) the gate opens for the walker and it passes (*closed friendly gate*, as the walker's view
    of the gate would give), or the walker takes the gate as its target (as the gate owner's view
    of claim 9's attack test would give), or (a2) the vehicle passes through the opened gate,
  - (b) the walker stops in front of *X* and does not enter it (the occupant's view would give
    *enemy obstacle*),
  - (c) the route takes the bypass from the start (*enemy obstacle*, cost 20, as the occupant's view
    would give),
  - (d) the route takes the bypass from the start (*friendly obstacle*, as the wall owner's view
    would give). If C3 (c5) and (c6) did not tell the two obstacle values apart, (d) cannot decide,
  - (d2) the route takes the bypass from the start. If C20 (j1) and (j2) did not tell the two
    obstacle values apart, (d2) cannot decide.

  If the alliance command is refused, or *K* also counts *H* as its ally (for example, *K*'s armed
  units elsewhere stop attacking *H*), the scenario cannot decide. If the measurement setup cannot
  allow alliances to change during play, the scenario cannot be run, and the views stay unbacked.

**Scenario C26: a flag or a reserved position with nothing listed (claims 1 and 13; claim 5, rows
VR5 to VR10; Terms *allied*).** Each part catches a cell *X* in a moment of a few ticks in which
the occupancy record of *X* shows something while *X* lists no object:
- in part (a), a vehicle *U* has started a movement step into *X*, so the vehicle-present flag of
  *X* is set, while its position is still outside *X* (claim 1),
- in parts (b) to (f), an infantry unit *E* has chosen its walking target in *X*, so *X* shows a
  taken side position and *E*'s house, while *E* is still listed in the cell it is leaving (claims 1
  and 13).

That moment is called the *window*. Common setup:
- *Layout.* As in scenario C20: a one-cell-wide corridor through *X*, a bypass that leaves the
  corridor before *X* and rejoins it after *X*, of the length given in each part, and a mover *M*
  of the human player, placed outside the usable area and ordered to a corridor cell past *X*. A
  second one-cell-wide corridor crosses the first at *X*. In parts (b) to (f) it is made of a land
  type on which walkers may move and *M* may not (S-0211), so *M*'s routes never use it; in part
  (a) it is clear ground.
- *Crossing unit.* In part (a), *U* is an unarmed vehicle of the computer player of an ordinary
  driving stock type. In parts (b) to (d) and (f), *E* is an unarmed infantry of the computer
  player; in part (e) it belongs to the house *K* of part (e). Map triggers move the crossing unit
  back and forth along the second corridor, through *X* and on to a cell two cells beyond *X* on
  either side, so that it crosses *X* many times.
- *Recorder.* On every tick it reads the crossing unit's position, *E*'s walking target, the cell
  into which *U* has started its step (S-0207), and *M*'s stored path (S-0212 claim 20).
- *Scripted input* (glossary *measurement setup*) gives *M*'s order, on a tick that is varied
  from run to run. A person may give the order by hand instead, at varied times. The counting rule
  below then discards more runs, so more runs are needed, but nothing else changes.
- *Search tick:* the tick on which the recorder first shows *M* holding the stored path computed
  for that order.
- *Window:* in part (a), the ticks on which *U* has started its step into *X* while its position
  is outside *X*; in parts (b) to (f), the ticks on which *E*'s walking target lies in *X* while its
  position is outside *X*.
- *Counting runs.* A run counts only if its search tick, and the ticks just before and after it,
  lie in one window. Runs are repeated until 5 runs of each part count. A run whose search tick
  falls before the crossing unit's first step towards *X* is a control run: *X* is then empty and
  its record clear, so *M*'s route leads through *X*.
- *Setup:*
  - (a1) *M* is an armed vehicle of an ordinary driving stock type that is not a crusher, and the
    projectile of its primary weapon can hit ground targets. Bypass 10 cells longer.
  - (a2) As (a1), without the bypass and without scripted input: *M* drives along the corridor and
    reaches the cell before *X* at varied times, over 20 runs, as in scenario C2 (c). The recorder
    also reads the tick *t*ₛ on which *M*'s position is first the centre of the cell before *X*.
  - (b) *M* is an armed vehicle of a stock type that is a crusher. A test file makes *E*'s infantry
    type not crushable. Bypass 10 cells longer.
  - (c) *M* as in (a1). Bypass 30 cells longer.
  - (c2) As (c), with a test file that makes the projectile of *M*'s primary weapon unable to hit
    ground targets (its warhead and damage stay as they are).
  - (d) *M* is a vehicle of an ordinary driving stock type without a primary weapon, not a crusher.
    Bypass 10 cells longer.
  - (e) The alliance setup of scenario C25: the human house *H* has given the alliance command
    towards the computer house *K*, and *K* still counts *H* as hostile. *E* belongs to *K*. *M* as in
    (a1), owned by *H*. Bypass 10 cells longer.
  - (f) A test file gives the train flag (S-0212 Terms) to an ordinary driving stock type without a
    primary weapon that is not a crusher, and *M* is of that type. Bypass 10 cells longer. *M*'s
    first step leads straight along the corridor, so that the train restriction of S-0212 (claim
    16) does not change its route.
- *Procedure:* run the triggers, give *M*'s order by scripted input, and record until *M* has passed
  the end of the bypass, or 400 ticks.
- *Observable:* whether *M*'s stored path leads into *X* or into the bypass, in the counting runs
  and the control runs. In (a2), per run, the window and *t*ₛ, and the first tick after *t*ₛ on
  which *M*'s position moves on from the centre of the cell before *X*.
- *Runs:* as many as needed for 5 counting runs per part (a few ticks of each crossing fall in a
  window, so about 40 runs per part); (a2) 20.
- *Expected:*
  - Control runs of every part: the route leads through *X*.
  - (a1) The route leads through *X*: the flag gives *moving blocker* (row VR5), whose blocker chain
    ends at *X*, which lists no object, at cost 1 (S-0212 claim 8).
  - (a2) In runs in which *t*ₛ lies in the window, *M*'s position does not move on from the centre
    of the cell before *X* before *U*'s position has entered *X*: *moving blocker* does not let it
    start its step.
  - (b) The route leads through *X*: row VR7 gives the crusher *clear* (cost 1).
  - (c) The route leads through *X*: row VR10 gives *enemy obstacle* (cost 20, against 30 extra
    cells).
  - (c2) The route takes the bypass: row VR10 gives *blocked*.
  - (d) The route takes the bypass: row VR9 gives *blocked*.
  - (e) The route leads through *X*: in *M*'s view *K* is allied, so row VR6 gives *moving blocker*
    at cost 1.
  - (f) The route leads through *X*: row VR8 gives *enemy obstacle*, which costs a train 1.
- *Falsified if:*
  - (a1) the route takes the bypass in a counting run (as *enemy obstacle*, cost 20, would make it,
    were the flag read only together with a taken position),
  - (a2) in a run in which *t*ₛ lies in the window, *M*'s position moves on towards *X* before
    *U*'s position has entered *X* (as *clear* would let it),
  - (b) the route takes the bypass in a counting run (as row VP30, *enemy obstacle*, would make it
    if *E* were already listed in *X*, or as rows VR8 to VR10 would without row VR7),
  - (c) the route takes the bypass in a counting run,
  - (c2) the route leads through *X* in a counting run (as row VP30, *enemy obstacle*, would make it
    if *E* were already listed in *X*: that row does not ask about the projectile),
  - (d) the route leads through *X* in a counting run (as *clear* would),
  - (e) the route takes the bypass in a counting run (as the infantry house in *K*'s view would
    make it: row VR10, *enemy obstacle*, cost 20),
  - (f) the route takes the bypass in a counting run (as row VR9, *blocked*, would make it without
    row VR8).

  Parts (d), (e) and (f) take for granted that *E* is not yet listed in *X* during the window. If
  neither (b) nor (c2) came out as expected, they cannot decide. If a control run's route takes the
  bypass, the margins are in doubt and the part cannot decide. In (e), the conditions of scenario
  C25 for a one-sided alliance apply. In (f), if *M* does not set out, the part cannot decide.

**Scenario C27: a train at the start of a step (claim 5, part 5f).**
- *Setup:*
  - (a) A one-cell-wide corridor of clear ground with no bypass. An idle vehicle *V* of the human
    player stands in a corridor cell *X*. A test file gives the train flag (S-0212 Terms) to an
    ordinary driving stock type without a primary weapon that is not a crusher. A vehicle *T* of
    that type, of the human player, stands 6 cells before *X* and is ordered to a corridor cell
    past *X*. Control (a′): the same with a vehicle of that type without the test file.
  - (b) As (a) and (a′), on a corridor of water cells, with a stock ship type for *T* and an idle
    ship of the human player as *V*.
- *Procedure:* give the order and record *T*'s position on every tick, for 300 ticks.
- *Observable:* whether *T*'s position enters *X* while *V* stands there, and whether *T* first
  stays at the centre of the cell before *X* for at least one tick.
- *Runs:* 2 each.
- *Expected:*
  - (a) *V* gives *stationary friendly* (row VP20), which costs a train 1, so *T* sets out, and at
    the start of the step into *X* that value counts as *clear*: *T* drives into *X* without stopping
    in front of it. What then happens to *V* belongs to other specs. (a′) *T* stops in front of *X*
    and does not enter it while *V* stands there.
  - (b) As (a) and (a′).
- *Falsified if:* (a) or (b) *T* stops in front of *X* and does not enter it while *V* stands
  there, in a pair of runs in which the control stopped too. If the control enters *X* while *V*
  stands there, or *T* does not set out, the part cannot decide.

**Scenario C28: when a driving vehicle clears its flag (claim 1).**
- *Setup:* a vehicle *U* of the human player, of an ordinary driving stock type, with a test file
  that sets its speed field to 1 and its acceleration flag off (as in S-0207 scenario D8). *U* is
  placed by the map, already facing map north, and has not moved before, so it has no leftover
  progress. (a) A straight route of 5 cells along map north. (b) A corridor one cell wide that makes
  *U* drive a two-cell curve (S-0207 curve D: a step north, then a step east). (c) *U* stands still
  in a cell, and a test map trigger destroys it.
- *Recorder:* on every tick, *U*'s position, the vehicle-present flags of the ground layer of the
  cells of the route, and the tick number.
- *Procedure:* order *U* along the route (parts a and b) and record until it arrives. In (c),
  fire the destruction trigger and record through *U*'s removal.
- *Observable:* the tick on which each cell's flag is set and cleared, against *U*'s position.
- *Runs:* 1 each (no random draw is involved).
- *Expected:*
  - (a) The flag of the start cell stays set through *U*'s first three ticks of movement, in which
    its position already moves (3, 7 and 10 leptons, S-0207 claim 4), and is cleared on the fourth,
    the first tick in which *U* passes a track point (progress 8, above 7). The next cell's flag is
    set from the tick on which the step starts. Each later cell keeps its flag from the start of the
    step into it until the first track point of the step out of it.
  - (b) The flag of the curve's first cell is cleared on the tick on which *U* reaches the release
    point of the curve (S-0207 table T1), while *U*'s position is still in that cell. The target
    cell of the curve keeps its flag.
  - (c) The flag of *U*'s cell is cleared when *U* is removed.
- *Falsified if:* in (a) the start cell's flag clears on the first tick of movement, or only when
  *U*'s position leaves the cell, in (b) the first cell's flag clears at another point, or in (c)
  the flag stays set after *U* is gone.

**Scenario C29: aircraft listing during flight (claim 13).**
- *Setup:* a tiny flat skirmish map, game speed index 2, stock data, no engine add-on and no mods (the named test files of a scenario excepted), with
  one human player and one bystander computer player kept outside the observed area. The observed
  area contains an empty, explored and buildable foundation site within adjacency of the human
  player's buildings, and an ordinary transport aircraft belonging to that player. Prepare a
  completed ordinary building whose foundation fits that site.
- *Procedure:* fly the transport over the site and issue a placement order while its horizontal
  position is inside a foundation cell and its height above the terrain is positive. In a fresh
  run, land it in that same foundation cell before issuing the placement order. In a third run,
  land there, take off and repeat placement once its height becomes positive.
- *Observable:* record the aircraft's horizontal position, integer height above terrain and the
  placement-order tick with the read-only recorder, and whether the building enters play. Use
  scripted input to align the order with the crossing; discard a crossing outside the foundation.
- *Runs:* one valid run per part; void any run in which an unnamed bystander object enters the
  observed area or acts on its objects, or the bystander is defeated before observation ends.
- *Expected:* placement succeeds during flight and after takeoff, but fails while the aircraft
  is landed. All other placement conditions must pass in each run.
- *Falsified if:* the flying aircraft alone refuses placement, a landed aircraft allows it, or
  the takeoff leaves a listing that still refuses placement after height becomes positive.
- *Bridge contrast:* use S-0501 scenario AC27's high-bridge landing and departure setup and
  five repetitions, recording world height, terrain surface height, the aircraft's own deck
  mark and both occupant lists separately from occupancy flags. Expect no listing during a
  positive-listing-height flyover, deck-only listing at marked deck rest, and no listing once
  takeoff gives positive listing height. Ground listing at marked deck rest, or either listing
  at positive listing height, falsifies claim 13. Failure to reach a positive-height marked
  retention state in AC27 is inconclusive, not a listing-rule counterexample.

**Scenario C30: retained aircraft ground mark versus entry and standing-position choice
(claims 1, 3, 5 and 6; Q-0310).**

- *Setup:* use S-0207 D22 run 14's named test map and teleporting aircraft, high
  bridge over traversable level ground, human house and distant bystander computer
  player, game speed index 2, no engine add-on and no mods (the named test files of a scenario excepted).
  Keep the landing cell free of all other flag writers. Void unrelated bystander
  interference in the bridge and probe area, or its defeat.
- *Procedure:* repeat the accepted-next-destination and destruction branches three
  times each. At reservation, after deck placement, after replacement and after
  removal, use a read-only recorder to record both occupancy records, listings and
  ticks. In separate repeats probe the ground beneath with an ordinary non-crushing
  driving vehicle and a walking infantry unit. Use a ground corridor that puts the
  cell next in each probe's path. Record the ground entry outcome and, separately,
  the infantry's walking target and position on each asking tick. Capture any
  pre-jump reservation interval rather than assuming it lasts a full tick.
- *Expected:* ground alone is marked before placement, both records after placement,
  with deck listing only. Replacement clears old ground and departure clears old
  deck. Destruction on deck clears deck but leaves ground marked. In otherwise clear
  ground with that mark alone, both probes get *clear*: the vehicle can start its
  step, but the walker chooses no standing position, remains still on that asking
  tick and asks again next tick. After replacement clears ground, that refusal ends.
- *Falsified if:* ground clears solely on deck placement or destruction, deck remains
  marked after removal, the retained flag alone makes either entry outcome non-clear,
  or the ordinary walker reserves a standing position despite that flag and no open
  gate. A missing captured pre-jump interval is inconclusive for reservation timing.
- *Additional removal controls (Q-0356):* S-0207 scenario D22 run 15 supplies the
  shared setup, three repetitions, recorder fields and falsifiers for vehicle and
  aircraft removal before a pending jump, deck removal after replacement, and
  off-bridge removal with coincident marks. Expect actual occupancy clear and any
  distinct pending ground reservation retained, with old ground already clear after
  replacement. A missed pre-jump interval is inconclusive. Apply the separate entry
  and standing-position probes above to each isolated aircraft ground flag.
- *No result is yet an M-record.*

## Open questions

- Factory release establishes an infantry tether situation (S-0417 claim 10), but whether it
  can expose claim 4's zero walker speed exception or tethered-entering override remains open
  (U1). Its contact alone also does not establish a factory-release setup reaching claim 8's
  stationary-friendly give-up comparison while that contact persists. Scenario C4 (f) remains
  the contact-condition proposal; ordinary factory release is not additional backing for it.
- The exact list of behaviours that have a mission target in claim 4 belongs to the behaviour specs.
  In the original they include entering, capturing, being taken in, and sabotage by a unit that
  carries explosives. They also include guarding, area guarding and patrolling by an engineer-like
  unit.
- Which two states of a laser-fence building make it count as nothing in claims 4 and 5 is not
  known, nor which events put a section in them (S-0211). Once the structure specs identify the
  states, the laser-fence rows become measurable with a test file that gives a building type the
  laser-fence flag.
- A human-controlled house that the computer takes over during the game appears to be given the
  maximum skill level then. That belongs to the future network-game spec (network games are out of
  scope for now, the project lead's decision of 2026-10-09).
- Whether between-point movement (S-0207 claim 4) can put a driving vehicle's position in a
  cell before that cell's vehicle-present flag has been set for its movement remains open.
  Claim 1 establishes reservations at step start and the stated release moments, not an
  exhaustive comparison of every between-point position with the cells reserved. Scenario C28
  checks ordinary straight and two-cell-curve cases; it does not settle every track or chained
  curve. This is a shared clarification for S-0207 and S-0209: any resolution must reconcile the
  position rule with the reservation and release rules and be handed off in both specs together.
- When ships and walking vehicles clear their vehicle-present flags (claim 1) is
  not established. Hovering vehicles use S-0503 claim 13's reservation and between-steps rules. The driving vehicle's moments are claim 1's. Whether a vehicle that is removed from
  the map in the middle of a track clears the flags it set ahead of itself is not established either.
- The underground locomotor's rule (claim 5, part 5b): which cells it makes *blocked*, the
  game-wide setting that turns it off, and the situations in which it applies belong to the specs
  of underground travel. Stock data gives no unit type that locomotor.
- Which reference height level order targeting (S-0213, S-0214), apart from the group look's
  *obstructed* class (S-0213 claim 7), and other situations outside the walking ask, vehicle step
  check, path search and teleporting-infantry landing check use, and
  so which cell layer of a high-bridge cell counts for them and for that group look (Terms,
  *cell layer*), is not established. Path-search queries use the expanded pair's height level and cell, as S-0212
  claims 7 and 10 give them. S-0208 claim 19 establishes the landing check's initial −1 reference
  and its effective deck reference; it does not establish those choices for other callers.
- On a high bridge whose deck spans ground that changes by 1 height level, the objects come from the
  deck layer and the occupancy record from the ground layer (Terms, *cell layer*). Whether such a
  bridge can exist belongs to the bridge specs. Whether this case deserves a switch is a question
  for the project lead (below).
- The blocking cells of a building (Terms) count the impassable-rows value along x from the
  top-left cell. Whether that matches the rows a player sees as impassable was not checked.
- In a campaign game an aircraft can get *blocked* for a cell under shroud (claim 14). The
  conditions involve state of the aircraft that was not identified. Campaigns are out of scope for
  now (the project lead's decision of 2026-10-10).
- A building as an actor (for example one that undeploys into a vehicle, S-0214 row GC9) has an
  entry outcome of its own that this spec does not give.
- Row VP29 of claim 5 asks about the weapon a vehicle would use against a terrain object. For a
  train without a primary weapon, which counts as armed (Terms *armed*), what that row gives was
  not established.
- When the repeated ask gives *friendly obstacle* and no object is listed in the cell, the walker
  takes the allied wall as its target (claim 9). Whether it then fires on the wall belongs to the combat
  specs. Whether this case deserves a switch is a question for the project lead (below).
- An infantry type whose own locomotor is a jumpjet locomotor but which is walking at the time may, after
  the new path of claim 7 when it has fewer than four steps, end its walking tick without choosing a
  walking target (low confidence). That belongs to the movement specs of such units. Scenario C19
  uses an ordinary walking type for that reason.

## Cross-spec coordination

S-0209 owns occupancy outcomes by the decision of 2026-10-10. No new policy decision is needed.
S-0208's current correction-batch text agrees with claim 5 for a still allied infantry occupant,
and S-0212's current claim 27 agrees with rows VP21 to VP23 for moving allied units.
The current S-0212 draft contains the corrections below; confirm their final
approved text at paired hand-off, rather than treating the earlier approved summary as authority:

- Retain claim 5's override rule: a *blocked* value before the first override defeats it, while
  occupants after that override do not decide the outcome under the default
  `quirks.occupancy_override_hides_later_blockers`. Without an override, use the most severe
  contents value and then the record table. Claim 5 carries the backing and confidence.
- Retain the allied-building summary's pointer to rows VP3 and VP5 to VP18 instead of
  equating all building exceptions with mission targets. Invisible buildings, non-blocking repair
  or bunker cells, the bib edge and inactive firestorm walls are among its exceptions.
- Retain the non-crushable-wall summary's qualification by rows VO4 to VO6: a primary warhead
  able to damage wood also suffices when the overlay armour is wood, even when it cannot
  damage walls.
- Confirm claim 1 and claim 5 with S-0207 claim 20 a/e: ground reservation,
  remembered-point clearing, placement-time deck marks and the shared vehicle quirk.
  Confirm the aircraft lifecycle and its separate shared quirk with that same claim,
  including *clear* ground entry versus infantry standing-position refusal (C30).
- Confirm claim 1 with S-0208 claim 19: retained infantry ground mark, no new deck
  mark at landing, later actual-deck clearing and the shared infantry quirk (W11 i).
  Keep position-choice deck mode, landing entry layer and fallback search deck mode
  distinct. S-0212 claim 25 retains those fallback choices and fixation-tick selection.
- Retain S-0207 claim 15 and S-0212 claim 22's immediate fresh search and same-tick
  repeated check. Jump-time target loss and reversion stay owned by S-0207 claim 20 e
  and S-0215 claim 10; reservation layers do not change that contract.
- S-0215 claim 10 (Q-2396): its list of other users of override and revert can name claim 9
  here next to S-0207 claim 15. A walker's repeated ask that gives *friendly obstacle* or *enemy
  obstacle* overrides to *attack*, with no destination, against the first object listed in the
  cell when its house is not allied with that object, or against the wall when no object is
  listed and the cell holds a wall. S-0309 needs no change: its claims 23 and 29 already give the
  drop and the attack-move resumption that claim 9 relies on.

The S-0102 alignment item is satisfied: its claim 3 and layer table agree with
S-0502 claim 4's two-height predicate and distinguish on-deck surface height
and the independent bridge/parachute falling state from crashing. No further
S-0102 correction is required.

## Questions for the project lead

- **Switch for objects and record from different cell layers.** If the bridge specs show that a
  deck can span uneven ground (Open questions), should there be a switch
  `quirks.cell_layer_split_on_uneven_deck` (default: the original), whose alternative would read the
  record from the same cell layer as the objects?
- **Switch for a walker that targets an allied wall.** Once the combat specs say whether the walker
  fires on the allied wall it takes as its target (claim 9), should there be a switch
  `quirks.walker_targets_allied_wall` (default: the original), whose alternative would leave the
  walker without a target, stopped and without a path?
