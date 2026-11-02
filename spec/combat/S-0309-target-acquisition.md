---
id: S-0309
title: Target acquisition (automatic searches, candidate tests, threat score, retaliation, target switching)
topic: combat
revision: 93
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-02
measurements: []
requests: [Q-0061, Q-0079, Q-0162, Q-1502, Q-1504, Q-1761, Q-1763, Q-2006, Q-2112, Q-2114, Q-2261,
Q-2262, Q-2263, Q-2264]
supersedes: []
---

# S-0309: Target acquisition

## Summary

How objects pick targets by themselves in *automatic searches* (range, area and map-wide): what a
search looks for, where and in which order it looks, which objects are candidates, the *threat
score* and the winner, plus the retaliation conditions, the rules for dropping or replacing a
self-chosen target, the drop of a held target that is destroyed or removed, starts to cloak or
goes underground, the periodic check that drops a held target the firing check refuses, and
the owner rules that the neutral and special houses keep. S-0215 owns when
most searches run (claim 1 also names contexts owned by other specs). Skirmish only: network games and
campaign missions are out of scope.

## Terms

Glossary terms are used as defined there, in particular *object*, *type*, *field*, *house*,
*human-controlled*, *computer-controlled*, *AI house*, *behaviour*, *target, destination*, *guard
spot*, *cell*, *lepton*, *legal cell*, *usable cell*, *bridge deck*, *cell layer*, *render layer*,
*scenario random stream*, *tick number*, *known attacker*, *disguise*, *being erased*, *armed* (here in the sense
of S-0214), *quirk switch*, *departure*, *importer warning*, *measurement setup*, and the *neutral
house* and *special house* (glossary *house*). From other
specs: *behaviour* names, *targeting timer*, *opportunity target*, *processing*, *team member* and
*mind control* (S-0215); *first per-tick start*, *late start* and *second start* (S-0215 claim
11); *allied*, *health ratio*, *counts as a vehicle* and *mind-controlled*
(S-0214); *veterancy rank* and *veterancy ability* (S-0311, claims 1 and 3); *versus value*, τ<sub>r</sub> and τ<sub>a</sub> (S-0304); *weapon
slot*, *weapon number*, *weapon it would use*, *mean weapon damage*, *healer for acquisition*,
*multi-weapon type*, *gattling type*, *switching type* and *airborne* (S-0308); *floor-bound* (S-0307, Terms); *expected damage* (S-0302, claim 18); *reference point* and *table square
root* (S-0204); *movement zone* (S-0211); *firing check* and its answers *illegal*, *impossible*, *busy*,
*rearming* and *out of range* (S-0301, claim 9); *tracked flying object*, *sector* and the flying-object search
(S-0303, Terms and F7); *land-targeting mode* (S-0308, table P and Import mapping); *naval-targeting
mode* (S-0308, table N); *spray-attack
type* (S-0313, Terms); *spawner weapon*, *spawned aircraft* and *launching* (S-0301, Terms),
*slave master* and *harvesting slave* (S-0215, Terms); *vehicle thief* (glossary); *draining weapon* and *locomotor warhead*
(S-0308, table P, rows 7 and 8); *charging mark* (S-0518 claim 15).

**Houses in our engine.** By the project lead's decision of 2026-10-09, an AI house follows the
same unit rules as a human-controlled house (S-0215, Terms, *Houses in our engine*). That ruling
covers player houses only. By the project lead's decision of 2026-10-10, the two non-player houses
that every skirmish creates, the neutral house and the special house, are not AI houses and keep
the original's rules for owners that are not human-controlled (glossary *AI house*, *Player houses
only*). The original tells owners apart only by whether they are human-controlled. So throughout
this spec:
- a rule given for a "human-controlled owner" applies in our engine to every player house, human
  or AI,
- a rule given for a "computer-controlled owner", which here means any owner that is not
  human-controlled, applies in our engine to the neutral and special houses only. Claim 25 gathers
  these rules. For AI houses they are the original's only: Edge cases and quirks records them as a
  note for AI designers, for an AI that may reproduce them through ordinary orders.

**Test-only exception.** An explicitly marked test-only computer-controlled player house
(glossary; S-0215 claims 18 and 19) uses the original's computer-owner rules, not the
human-owner rules. This setting is for oracle comparisons only, including both houses in
headless arena mode; it is unavailable in normal games and is not given to our AI. Throughout
this spec, statements restricting computer-owner rules to the neutral and special houses,
or applying human-owner rules to every player house, exclude these marked test houses.
References to the neutral and special houses in the owner-dependent rules include them,
but those houses' setup values do not: a marked test house uses its corresponding oracle
house's skill level and difficulty index, and has no teams. In particular, claim 25 applies,
with its difficulty-indexed entries selected by that test house's index rather than necessarily
index 1. This is a design rule, not an additional claim about the original.

- **Automatic search.** A search in which an object (the *searcher*) picks a target by itself. It
  has a *kind*, a *centre* and, for two kinds, a *radius*:
  - **range search**: centre the searcher's position, radius from claim 8 (Range). S-0215 calls the
    per-tick search its *opportunity search*,
  - **area search**: centre the guard spot, radius from claim 8 (Area). S-0215's *area search*,
  - **map-wide search**: no radius. S-0215's *normal search*.
- **Search classes.** The kinds of object a search may pick: *infantry*, *vehicles*, *aircraft* and
  *buildings*, and the modifier *friendly* (claim 6). A search's classes are fixed before it starts
  (claim 5). A search with the *friendly* modifier accepts only objects allied with the searcher,
  in every part of it (table C, row "healer"); only an infantry unit or a vehicle that is a healer
  for acquisition searches with it (claim 6).
- **Friendly-row exceptions.** A searcher whose type attacks friendlies, that is berserk (S-0302,
  Terms), that has the permanent-berserk mark, or that is a passenger of a mind-controlled
  open-topped transport. Table C's rows "friendly" and "healer aircraft" do not apply to it.
- **Candidate.** An object that passes every test of claim 12 for the searcher. Every candidate has
  a *threat score*, an integer of at least 1 (claim 13).
- **Threat coefficients.** Five decimal fields of the searcher's type: *own-effectiveness*,
  *target-effectiveness*, *special-threat*, *strength* and *distance* coefficients, plus the
  *special threat value* of each candidate's type (Import mapping).
- **Expected remaining health.** An integer per object that estimates its health after the shots
  already aimed at it (claim 15).
- **Anger, enemy house.** Each house keeps an *anger* value for every other house, and an *enemy
  house*, or none (claim 16).
- **Sensed cell.** As defined by S-0516 (Terms, claims 21–23), which owns sensor coverage.
- **On the playfield.** An object inside the usable area that takes part in play; the visibility
  specs own the exact test.
- **Ring *r*.** The cells whose larger coordinate difference from the centre cell is exactly *r*
  (the square ring of Chebyshev radius *r*).
- **Current weapon.** The weapon in the slot of an object's weapon number when its type is a
  multi-weapon type (S-0308, gattling types included), and the weapon in its slot 0 otherwise.
- **Passive acquisition.** Whether an object may pick targets by itself at all (claim 2).
- **Engaged.** A yes/no mark on a mobile object that has an attack-move memory (S-0213, claim 6),
  set by claim 23 and ended as claim 23 gives.
- **Limited-scan mark.** A yes/no mark on a mobile object that turns its area and map-wide searches
  into range searches (claim 22).
- **Fired-at list.** The list of objects that a distributed-fire object has shot at in its present
  round (claim 20).
- **Engineer, money thief, infiltrator.** Infantry types named by the imported flags of the same
  names (Import mapping: *engineer*, *thief* and *infiltrate*). An *unarmed money thief* is an
  unarmed infantry type with both the money-thief and the infiltrate flag (claim 5). Other specs
  own what these units do once they act.
- **Permanent-berserk mark.** A yes/no mark on an infantry unit, without any time limit. It makes
  claim 10's per-cell selection take any object for the unit and exempts the unit from table C's
  rows "friendly" and "healer aircraft", as berserk (S-0302, Terms) does; unlike berserk, it does
  not make the searches look at allied objects (claims 7, 9 and 10). Nothing in the rules of our
  engine sets it (Edge cases and quirks).
- **Multiplayer-passive house.** A house whose country is marked passive for multiplayer (glossary
  *multiplayer game*; Import mapping). Which houses that is depends only on the rules data.
- **House order.** The order in which a skirmish creates its houses, as S-0009 (claim 19) gives
  it; among computer players it is their lobby-slot order. Claim 16 breaks ties between anger
  values by it.
- **Target-health scan mode.** A per-type setting with three values, *none*, *normal* and *strong*
  (Import mapping, default *none*), that says how a searcher treats a candidate's expected remaining
  health: *normal* applies the weak-target preference of claim 14, *strong* rejects candidates whose
  expected remaining health is 0 or less (table C, row "strong scan"), *none* does neither.
- **Spawner type.** A type that keeps spawned aircraft (S-0301, Terms). An object of such a type
  *controls spawned aircraft*, whether or not any of them is launching or alive.
- **Teleporting move.** A move that ends in a chrono move by the object's own teleporting
  locomotor (S-0207, claim 20). The jump and the end of *phasing in* are distinct events:
  the search of claim 1 runs at the latter, not when the object reappears at its new place.
- **Planned route.** A route through several points that the person laid out for a unit in
  advance, in the original's waypoint-planning mode. The orders specs own it (S-0213's order set
  does not include it yet). A unit *follows no planned route* when it has no such route or none
  with a point left.
- **Opportunity gate.** The test of claim 3 that the range searches of claim 1's first and last
  rows must pass, besides claim 2.
- **Threat sum.** The double-precision sum inside conv in F4, before conversion and before the
  modifiers of claim 14, with *d* in cells. Claim 25, part h, compares two of them.
- **Disguise-detection list.** A game-wide list of integers, one per difficulty row, that sets
  how likely a searcher of the neutral or special house is to see through a blinking disguise
  (claim 25, part g; Import mapping).
- **Destroy-walls flag.** A yes/no entry in the per-difficulty settings (Import mapping). Claim 25, part i, uses the entry selected by the house's difficulty index; the neutral and special houses have index 1, which selects the second entry. This is a per-difficulty-list selection, not an applied difficulty row.
- **Wall candidate, wall score.** A cell holding an enemy wall that a searcher of the neutral or
  special house could shoot, and the integer that ranks such cells (claim 25, part i).
- **Wall target.** A target that is a cell holding a wall, not an object (claim 25, part i).
- **Red condition ratio.** A game-wide health ratio, the health-bar threshold (Import mapping).
  Claims 10 and 25 and table C's row "engineer" compare a building's health ratio with it.
- **Threat value.** Of a building: its occupant count times the game-wide threat per occupant when
  it holds occupants; otherwise the threat-posed field of the unit in its tank bunker, if one is
  there, otherwise its own type's threat-posed field (table C, row "harmless building").
- **Fully cloaked.** The *cloaked* state of S-0516 (Terms, claim 1), tested by table C's row
  "cloak".
- **Appears disguised to a house.** The game-state test of S-0516 claim 35. S-0516 (Terms,
  claim 40) owns the disguise-detector coverage supplied by detector buildings, distinct from
  the sensor coverage used by table C's row "cloak".
- **Creation order.** The order in which objects came into existence, the object created first
  coming first. Unlike the update order of S-0203 claim 3, it does not change when an object
  leaves play and enters play again: an object that leaves a transport keeps its place in it.

## Behaviour

### When searches run and what they do first

1. **Search contexts.** An infantry unit, vehicle or building runs an automatic search in these
   situations. S-0215 says when the first, third and fourth happen (claims 33, 26 and 27); claim
   23 here owns the second. S-0517 claim 16 owns the deployed-firing context, run by S-0215
   claims 23 and 31. S-0518 claim 15 owns the charging break-off context, and S-0207
   claim 20 f owns the phasing-in end that triggers the last:

   | Situation | Kind | Centre |
   |---|---|---|
   | its targeting timer has run out while its current behaviour is *guard*, *move* or *harvest* and claim 3 allows it, and it carries no attack-move memory | range | its position |
   | its targeting timer has run out while it carries an attack-move memory, whatever its current behaviour, in the cases of claim 23 that run a search (in the others no search runs and the timer is not restarted) | range | its position |
   | the processing of *area guard* (S-0215, claim 26) | area | the guard spot |
   | the processing of *hunt* (S-0215, claim 27) | map-wide | its position |
   | the deployed-firing rule of a deployed infantry unit (S-0517 claim 16), during the processing of S-0215 claims 23 or 31, when it has no target within the range of the weapon selected with no target supplied; no targeting-timer gate | range | its position |
   | an infantry unit's *attack* processing while its charging mark is set (S-0518 claim 15); no targeting-timer gate or reset draw | range | its position |
   | phasing in after its own teleporting move ends (S-0207, claim 20 f), it has no target, and claim 2 and the opportunity gate of claim 3 allow it; never for a harvester-flag type, which does not phase in; a target it sets is an opportunity target as claim 24 gives | range | its position |

   S-0517 claim 16 owns the deployed-firing rule's target retention, replacement and fall-back
   outcomes. Its search context does not by itself settle whether claims 2 and 3 limit that
   search (S-0517, Open questions).

   S-0518 claim 15 owns the charging search's replacement and mark-clearing outcomes. The
   held charging target neither prevents the search nor is cleared or tested for usability to
   permit it. Any nonempty result becomes the target and clears the charging mark, even when
   it is the same object as the held target; there is no further owner, firing-legality or
   identity test on that result. An empty result leaves both target and mark unchanged.
   Claims 2 and 3 do not limit this search. The intrinsic restrictions, candidate tests and
   ranking of claims 5 to 17 still apply, including claim 5's no-auto-fire restriction: a
   charging building does not become eligible merely because it is the held target. The search
   neither waits for nor restarts the targeting timer. The ordinary behaviour-delay draws
   still apply (S-0215 claim 21). S-0215 claim 33 explicitly excepts
   this search from the targeting-timer reset rule for searches covered by claim 4 here,
   and from its ordering rule for processing draws before per-tick-search draws. A replacement
   found by this search can be attacked in that same processing.

   An actor that carries an attack-move memory never runs the search of the first row, in any
   behaviour: it runs the search of the second row instead, which claim 23 describes. That search
   is not limited by claims 2 and 3, and the targets it sets are not opportunity targets (claim
   24). S-0215 (claim 33) points here for it.

   At the phasing-in end, with no target and more than 10 integer ticks left on the targeting
   timer, one ranged draw from the scenario random stream over 4 to 8 ticks, both included,
   resets that timer (S-0207, claim 20 f). This draw does not require claims 2 and 3 to allow a
   search. With a target or at most 10 ticks left, it is not taken. A search that is allowed
   still takes its own normal timer draw (claim 4) after this conditional draw, replacing that
   temporary timer value. Thus the end tick can take both draws, only the normal search draw,
   only the conditional draw, or neither, depending on these conditions.

   Other behaviours run searches with the same rules (claims 5 to 17) but their own kinds and
   centres: *patrol* (an area search with the patrol limits of F1), *rescue*, the behaviours of
   aircraft, and the original's team scripts. When they search is owned by S-0215, the aircraft
   specs and (for team scripts) the original's AI, which is out of scope.
   - Backing: `measurable` (the per-tick row by scenario A1, parts b and m; the attack-move row by
     scenario A6, parts d and g; the area-guard row by scenario A1, part g, and A11, part d; the
     hunt row by scenario A11, part a; the teleport case and its draw ordering by scenario A1,
     part f, with S-0207 scenario D22 runs 1, 2 and 7 for the phasing-in and conditional-draw
     conditions); the deployed-firing row by S-0517 scenario P7, parts b, c and f, including
     runs with positive ticks left on the targeting timer in part b; the charging break-off
     row and its no-timer-draw rule by scenario A17 below, supplementing S-0518 scenario TC3
     with positive remaining targeting-timer runs, a no-candidate case, permission and same-result
     controls, and random-state recording. The paragraph on
     other behaviours is a pointer to the specs that own those searches, not a claim of this spec,
     and carries no backing or confidence (Open questions).
   - Confidence: high (the first four and the charging break-off context); medium (the deployed-firing context and the teleport case)

2. **Passive acquisition.** An object may pick a target in the range searches of the first and last
   rows of claim 1, and in the area search, only when all of these hold (the map-wide search of
   *hunt* is not limited this way):
   - its type's passive-acquire flag is set,
   - it is armed,
   - it is not a harvesting slave (S-0215), and no temporal weapon of its own is erasing a target,
   - it is not a building that can be occupied and holds no occupants,
   - it is not an engineer of a human-controlled owner,
   - it is not a mind-controlling object that already controls as many objects as it can.
   - Backing: `measurable` (scenario A1: part d for the flag, part n for "armed", part l for the
     harvesting slave, part j for the empty building, part e for the engineer, which is given a
     weapon so that "armed" does not decide, and part k for the mind controller); `unmeasured` for the temporal-weapon condition, because a firer whose
     temporal weapon is erasing an object holds that object as its target, so a search keeps it
     (claim 4) whatever this condition says, and no setup is known in which the beam goes on
     without the target
   - Confidence: high

3. **The opportunity gate.** In addition to claim 2, the range searches of claim 1's first row
   (the per-tick search) and last row (the teleport search) run only when one of these holds,
   judged in the object's current behaviour:
   - its type has the opportunity-fire flag (in any behaviour, *guard* included, where the flag
     also lifts the area-fire exception below),
   - its current behaviour is *move*, it has arrived (its destination is its own cell) and follows
     no planned route (Terms), and its type has the balloon-hover flag or is a vehicle type that
     deploys in place,
   - its current behaviour is *guard*, unless the weapon in its slot 1 (slot 0 for a spray-attack
     type, S-0313) has the area-fire flag and is the weapon it would use against its present target
     (with no target, the slot that S-0308 gives when there is no target: its claims 8 and 9 for
     deployed infantry and vehicles, otherwise the first of rows 1 to 3 of its table P that
     applies).

   So without the opportunity-fire flag the per-tick search never runs in *harvest*, nor in *move*
   before the object has arrived. Which behaviour an object is in when its phasing in ends is
   not established here (S-0215 owns it), so for the teleport search only a type with the
   opportunity-fire flag is known to pass the gate.
   - Backing: `measurable` (scenario A1: part c for the area-fire exception in *guard* and for the
     opportunity-fire flag lifting it, part o for the spray-attack form, part b for the
     opportunity-fire flag in *move* and *harvest*, part m for the arrival cases, part f for the
     gate on the teleport search); `unmeasured` for the condition "follows no planned route": a
     unit with route points left is given its next point as destination by the orders specs' rules
     for planned routes, so no setup is known that holds it in *move* on its own cell with points
     left when its targeting timer runs out, and whether a route laid out but not yet started
     counts is not established
   - Confidence: high (guard and the opportunity-fire flag in *move* and *harvest*); medium (the
     arrival cases, and the flag lifting the area-fire exception in *guard*)

4. **A search keeps a target unless it was an opportunity target that has become unusable (claim
   23 gives the attack-move exceptions).** The charging break-off search is excluded from
   this claim: claim 1 and S-0518 claim 15 give its outcomes instead. Every
   search covered by this claim restarts the targeting timer (S-0215, F6), and that ranged draw is the search's first
   draw from the scenario random stream. In the teleport case this is separate from the
   conditional 4-to-8-tick draw before the search (claim 1). An opportunity target for which the firing check of the
   weapon the searcher would use against it answers *illegal*, *out of range* or *impossible* is
   dropped, and the same search may pick a new target (for a searcher that claim 29 covers, its
   periodic check can drop it earlier). One exception: a searcher that controls
   spawned aircraft (a spawner type) and gets *impossible* keeps the target, but resets its
   spawn controller as S-0506 claim 11 gives. For a searcher that claim 29 covers, that keep
   ends at its first checking turn in a check tick of claim 29 at which the answer is *illegal*
   or *impossible*: claim 29 makes no exception for it. Released
   missiles already in flight continue to
   their fixed aim points (S-0506 claims 11–12). The answer *busy*
   (the searcher is launching spawned aircraft, S-0301) is none of the three, so the target is kept
   and the spawned aircraft are left alone. Apart from the attack-move searches of claim 23 that
   run as for an object with no target, a search looks for a target only when the searcher is
   left with none, so it never replaces a kept target, and in particular never a target given by
   an order. This is retention within a search, not protection against other target-loss rules:
   before its behaviour processing, an aircraft clears even a living target whose in-the-air
   answer is yes (S-0501 claim 32, using the test defined in its Terms). The table *Search
   outcome and target state* (State and ordering) gives the cases covered by this claim.
   For a spawner, dropping an unusable opportunity target requests the conditional pending-target
   clear of S-0506 claim 25. A direct replacement by a nonempty target does not itself change
   pending, and a search that finds nothing while already targetless does not cancel pending.
   The *impossible* reset above remains a separate event that clears both controller records
   (S-0506 claim 11). Claim 23 gives the attack-move exception to requesting a clear.
   When the search picks a target object, that object's expected remaining health is reduced at
   once by the expected damage of the weapon the searcher would use against it (S-0302, claim 18),
   unless that weapon's projectile has the inaccurate flag (Import mapping). So a searcher that
   searches later in the same tick sees the lowered value, and in mode *strong* or *normal*
   (claim 14, table C) may reject or deprioritise that target. A searcher of a distributed-fire
   type picks by claim 20 instead, and its pick lowers no expected remaining health.
   - Backing: `measurable` (scenario A6 for the keep and drop cases, A6 part f for the *busy*
     case; that a
     player house's search takes no draw per tested or scored object by
     scenario A14, part n; that the timer draw is the search's first draw and that a search takes
     no fixed further draw by scenario A15, part h; the expected-health reduction, including the same-tick effect, through scenario
     A8, parts b and c; the inaccurate-projectile exception through A8, part d; the distributed-fire
     exception through A8, part e). `unmeasured`: a spawner's keep on *impossible*, its
     controller reset and its released missiles' continuing to their fixed aim points, since no
     part here gives a spawner an *impossible* answer; the drop and re-pick on *impossible* at a
     search by a firer that is not a spawner, since A6 part f reaches it only when a search of the
     firer falls in a hidden span that holds no check tick of claim 29, and no setup here is shown
     to give one)
   - Confidence: high (medium for the inaccurate-projectile and distributed-fire exceptions, for
     a spawner's keep on *impossible*, its controller reset and its released missiles' continuing
     to their fixed aim points, and for the drop and re-pick on *impossible* at a search by a
     firer that is not a spawner)

### What a search looks for

5. **Search classes from weapons.** A search looks for the classes that the searcher's weapons can
   hit: aircraft if a weapon's projectile is anti-air; infantry, vehicles and buildings (and ships,
   which are vehicles) if a weapon's projectile is anti-ground. For a switching type the weapon of its
   weapon number counts; for every other object its weapons in slots 0 and 1. A search that looks
   for vehicles also accepts aircraft (in practice landed ones, since an airborne aircraft fails the
   firing check of a weapon that is not anti-air) and every object that counts as a vehicle
   (S-0214). These further rules apply:
   - a vehicle of a human-controlled owner whose type deploys to fire never searches,
   - an object whose type has the no-auto-fire flag never searches if its owner is human-controlled,
   - an aircraft takes no classes from its weapons, so the searches of claim 1 find nothing for it
     (the aircraft behaviours run their own searches with classes they name: aircraft specs),
   - an unarmed infantry unit that is a vehicle thief finds targets only by the vehicle-thief rule
     below (and, with the money-thief flag too, by the combination below); an unarmed infantry unit
     that is not a vehicle thief finds targets only if its type has the infiltrate flag *and* the
     money-thief flag, and then only by the money-thief rule below. Any other unarmed infantry unit
     (a plain infiltrator included) finds none,
   - an unarmed vehicle thief searches for vehicles only (with the landed aircraft and other
     objects that every search for vehicles accepts, above), never for infantry or buildings, and it
     runs no scan of airborne objects (claim 9). An armed vehicle thief takes its classes from its
     weapons like any other armed infantry unit. Any vehicle thief, armed or not, whose destination
     is a vehicle (an object that counts as a vehicle and is not a train)
     at a distance below 3840 leptons takes that vehicle as its target at once, without any test of
     claim 12 and without scoring. The distance is the distance from the thief to that object of
     S-0204 (claim 6), between reference points, with that claim's building reduction when the
     object is a building that counts as a vehicle,
   - an armed infantry unit whose primary weapon's warhead has versus values of exactly zero against
     both the *medium* and the *wood* armour class (S-0305), an *infantry-only* warhead, searches
     for infantry only,
   - an engineer never searches for infantry or vehicles, whatever its weapons or other flags
     allow; it searches only for the other classes those give (with an anti-ground weapon, for
     buildings). Like any infantry unit, it finds targets at all only by the rules above: an
     armed engineer by its weapons, and an unarmed engineer that is neither a vehicle thief nor an
     unarmed money thief not at all,
   - an infantry type with the money-thief flag also searches for buildings, and every candidate of
     its searches, whatever its weapons allow, must then be a capturable building whose type has
     storage capacity (table C, rows "capture" and "storage"),
   - a building always searches with the range kind, whatever the context asks.

   **How the infantry rules combine.** Every rule above that fits an infantry unit applies. For an
   infantry unit that is not a healer for acquisition (claim 6 replaces all of this for a healer)
   and that the vehicle-thief take-over does not settle, the classes searched are

   > *C* = ((*B* ∩ *I*) ∪ *A*) ∖ *E*

   where *B* is the classes of its weapons if it is armed, the vehicle class alone for an unarmed
   vehicle thief, and no class for an unarmed money thief with the infiltrate flag that is not a
   vehicle thief (any other unarmed infantry unit does not search); *I* is the infantry class alone
   when it is armed and its primary warhead is infantry-only (the versus rule above), and every
   class otherwise; *A* is the building class when its type has the money-thief flag (and for the
   neutral and special houses as claim 25, parts b and c, add), and no class otherwise; *E* is the
   infantry and vehicle classes for an engineer, and no class otherwise. The candidate rules of the money thief (table C, rows "capture" and
   "storage") hold for every candidate whatever *C* is. So in the combinations the bullets leave
   open:

   | Infantry searcher | Classes searched | Candidates |
   |---|---|---|
   | armed engineer with an infantry-only warhead, no money-thief flag | none | none: it finds nothing |
   | armed engineer with an infantry-only warhead and the money-thief flag | buildings | capturable buildings with storage |
   | unarmed vehicle thief with the money-thief flag | vehicles and buildings | capturable buildings with storage only, so no vehicle is ever a candidate (the take-over above can still take a vehicle) |
   | armed vehicle thief with the money-thief flag | its weapons' classes and buildings | capturable buildings with storage |
   | unarmed engineer that is a vehicle thief, no money-thief flag | none | none (only the take-over can give it a target) |

   Claim 25, parts b and c, changes some of these rules for the neutral and special houses.
   - Backing: `measurable` (scenario A2: the main part for the weapon classes, the infantry-only
     warhead and deploy-to-fire, part b for the vehicle thief, part c for the money thief, part j for the armed
     vehicle thief and the combinations of the infantry rules, part d
     for aircraft, part e for no-auto-fire, part f for unarmed infantry, part g for the armed and
     the unarmed engineer, part h for the landed aircraft that a search for vehicles accepts), except
     three rules that are `unmeasured`. The acceptance of a vehicle-like building (S-0214,
     Terms, *counts as a vehicle*) by a search for vehicles: a searcher that looks for buildings
     accepts such a building by its building class anyway, and the only searcher known to look for
     vehicles but not buildings is an unarmed vehicle thief, whose firing check against a building
     this spec does not establish, so no expected result can be stated. The vehicle-thief take-over: no rule of the current specs is
     known to leave a vehicle thief searching with a vehicle as its destination and no target (an
     order or a hunt that gives it a vehicle also gives it that vehicle as its target, which a
     search keeps), so the case cannot be set up. The building rule: no situation in our engine asks a building for an
     area or map-wide search (map set-up queues *guard*, not *area guard* or *hunt*, S-0009
     claim 40, and S-0215 claim 32 processes a building's *area guard* as *guard*), so the rule
     cannot be seen; it fixes the result for any later rule that does ask. Queuing *guard* does
     not guarantee that it is current; the set-up building's first automatic start remains open
     in S-0215 (Q-0241)
   - Confidence: high (weapon classes, deploy-to-fire, no-auto-fire, infantry-only warhead),
     medium (the aircraft, vehicle-thief and money-thief cases, including the infiltrate flag
     that an unarmed money thief needs, the armed vehicle thief and the combinations of the
     infantry rules)

6. **Healers look for damaged friends.** An infantry unit or vehicle that is a healer for
   acquisition (S-0308, claim 15) looks for friendly objects instead of enemies: its search classes
   are replaced by *infantry* and the modifier *friendly* for an infantry unit, and by *vehicles*
   and the modifier *friendly* for a vehicle (with the aircraft and the objects that count as a
   vehicle that every search for vehicles accepts, claim 5). A candidate must be allied with the
   searcher (table C, row "healer", which holds for every such healer, whatever the friendly-row
   exceptions) and have a health ratio below 1 (table C, row "friendly"; for a searcher with a
   friendly-row exception, the firing check of its healing weapon rejects an object at full health
   instead: S-0310, claim 11, the firing check's rule for healing weapons, which S-0314, claim 1,
   points to). A vehicle that is a healer for acquisition also passes over an
   aircraft whose height above the surface is above 0 leptons (a height test, not the *airborne*
   state of S-0308) or that stands on a cell holding a building (table C, row "healer aircraft",
   which a friendly-row exception waives). A healer for acquisition whose current behaviour is
   *guard* uses a radius of 512 leptons in any search that has a radius (F1); of the searches of
   claim 1 that is the range search. A building that is a healer for acquisition keeps the classes
   of its weapons and gets no *friendly* modifier, so table C's row "healer" does not apply to it.
   Three rules do treat it as a healer (low confidence): the radius rule above, the per-cell
   selection and test of claim 10, and table C's row "friendly", which lets an allied object
   through when its health ratio is below 1 and rejects one at full health.
   - Backing: `measurable` (scenario A12: the main part for the classes and the health ratio, part
     b for the radius, part c for the landed aircraft and the building cell, parts d and e for the
     allied condition in the map-wide search, with and without the attack-friendlies flag).
     `unmeasured` for an aircraft in the air: a vehicle healer has no aircraft class, so it runs no
     scan of airborne objects (claim 9) and sees only the objects that scanned cells offer, and
     whether a cell offers an aircraft that is in the air (claim 10 counts the objects of the
     cell's ground or deck layer) is not established, so no expected result can be stated.
     `unmeasured` for the waiver of the aircraft rule by a friendly-row exception: the only
     setup known, a vehicle healer with the attack-friendlies flag next to a damaged allied
     aircraft on a pad, may also be rejected by the firing check (whether an aircraft on a pad
     counts as a vehicle is not established), so no expected result can be stated. `unmeasured`
     for the building healer: what decides its picks is the firing check of a building with a
     healing weapon, which the combat specs do not give (S-0310, claim 11, covers infantry and
     vehicles), so no expected result can be stated
   - Confidence: high (medium for the waiver of the aircraft rule; low for the building healer)

### Where a search looks, and in which order

7. **The map-wide search.** The aircraft pass comprises every aircraft on the map when the search looks for aircraft; the ground-layer pass comprises every infantry unit, vehicle, aircraft and building whose render layer (S-0102) is the ground layer. On equal scores, an aircraft-pass candidate precedes a ground-layer-pass candidate. In both passes it considers the objects that are not allied with
   the searcher; for a searcher whose type attacks friendlies, that is berserk or that is a
   passenger of a mind-controlled open-topped transport it considers every object (the
   permanent-berserk mark does not widen it), and for a healer for acquisition only allied ones, whatever friendly-row exception it has
   (table C then decides). In the second pass an engineer of the neutral or special house also
   considers allied objects (claim 25, part d). A healer for
   acquisition has no aircraft class (claim 6), so the aircraft pass does not run for it. For a
   searcher that is a mobile object other than an aircraft,
   a ground-layer object is considered only if its cell lies in the same connected region of the
   searcher's movement zone (S-0212 claim 4) as the searcher's own cell, measured in the cell layer
   the object occupies, except for the no-link deck case below. There is no distance limit. So an object that is neither an aircraft nor in
   the ground layer, such as a hovering unit with jumpjet movement (air or top layer, S-0102
   claim 3), is never found by a map-wide search, whatever the searcher's weapons can hit
   (`quirks.hunt_ground_layer_only`, Edge cases).
   A landed aircraft in the ground render layer is considered in both passes when the search
   looks for aircraft: it is not removed from the second pass after being considered in the first.
   Each consideration has its own candidate tests and score. The aircraft pass has no connected-region
   test and uses distance in cells for F4; the ground-layer pass has the connected-region test
   above and uses distance in leptons. Either score can win, and the same object can supply both.
   Without the aircraft search class, the first pass does not run, but a search for vehicles
   still accepts a landed aircraft in the second pass (claim 5), with distance in leptons.
   The order of the two passes decides ties (claim 17), not which distance unit an aircraft uses.

   *Cells of no reach region.* Except for the no-link searcher case below, the connected-region
   test compares reach regions as S-0212 claim 4 does: two cells of no reach region of the
   searcher's movement zone count as lying in the same region, wherever they are on the map.
   The searcher's own cell is read for its deck (S-0212 claim 4) whenever it is a bridge cell,
   whatever layer the searcher is in; the object's cell is read in the layer the object occupies,
   always for the searcher's movement zone, never the object's.

   *No-link deck exception.* If the searcher's own deck read finds no bridge link, target
   acquisition omits the connected-region restriction for the whole search: no object or
   visited cell is rejected for its reach region. This is a search eligibility exception, not
   a change to S-0212 claim 4's definition of reach regions. In particular, two no-link deck
   reads are never compared in this search. If the searcher's read instead gives any other
   region, an object's no-link deck read fails the comparison, since S-0212 claim 4 makes that
   deck region distinct from every other region. The exception also governs the area-search
   cell test of claim 11. All other eligibility tests remain in force. This exception is low
   confidence and unmeasured (backing below).

   Away from that exception:
   - a searcher on a cell of a reach region never considers an object standing on a cell of no
     reach region of the searcher's movement zone (for example a cliff cell that this zone cannot
     use, S-0212 claim 4), however close it is,
   - a searcher standing on a cell of no reach region of its own movement zone considers only
     objects on cells of no reach region of that zone, anywhere on the map: not its enemies on open
     ground, but, for example, an enemy on another cliff cell or, for a land movement zone, on
     water,
   - neither unit leaves its cell because of the search: a hunting actor whose search finds nothing
     stays put (S-0215, claim 27).

   Such an object can still become a target through the searches and events that have no
   connected-region test: range searches (claim 11, including a map-wide search turned into one
   by the limited-scan mark of claim 22) and retaliation (claim 18).

   *Hovering jumpjets.* The aircraft pass holds aircraft-kind objects only, not infantry units or
   vehicles with jumpjet movement, so a hunting actor's own map-wide search never picks a jumpjet
   while it is outside the ground layer. Disc-laser firing does not itself force descent or
   ground-layer entry (S-0502 claim 32, answering Q-1504 question 2). Ordinary low ascent,
   descent, retained low target height or wobble can still put an attacking jumpjet in the
   ground layer independently of its weapon; that layer, not firing, decides eligibility here.
   These height rules inherit S-0502 claim 32's backing and confidence.
   A hunting actor can still attack one in these ways, each only with a projectile that is
   anti-air (the firing check refuses the hovering jumpjet to any other, claim 18 and S-0306
   claim 23):
   - it picked the jumpjet while that was in the ground layer (before lift-off, after landing,
     or while its height keeps it in the ground layer, S-0102 claim 3) and still holds it: a
     target set by a map-wide search is not an opportunity target (claim 24), so later searches
     keep it (claim 4) after the jumpjet rises,
   - the airborne scan of a range search (claim 9) reaches tracked flying objects, hovering
     jumpjets among them (S-0303, Terms); a hunting vehicle runs such a search in place of its
     map-wide search only while it has the limited-scan mark (claim 22),
   - retaliation (claim 18) has no render-layer test of its own, and the disc-laser weapon makes its firer
     the attacker (S-0615, claim 16). A hunting actor retaliates only when the *hunt* behaviour
     has the retaliate property, which the user's behaviour data sets (S-0215, claim 18 and Import
     mapping), and when the owner rules of S-0215 claim 18 allow it.
   - it gives up its move: a route search that fails more than 1 cell from its destination
     (S-0212 claim 23, row 2) also drops the actor's target, and the actor queues the
     owner-dependent resting behaviour that row gives: *area guard* in the original for a
     computer-controlled owner, and *guard* for a human-controlled owner and, in our engine, for
     every player house (Departure D5 there). Queueing it does not by itself change the current
     behaviour; it becomes current at a permitted start under S-0215 claim 11. The searches of
     either behaviour are range or area searches, whose airborne scan (claim 9) reaches
     hovering jumpjets.
   - Backing: `measurable` (scenario A11: part a for the absent distance limit, part b for the
     zone test, part f for the ground-layer restriction, part g for the aircraft pass coming
     first, parts h and i for landed aircraft in both passes and their distance units, part j for
     cells of no reach region, part k for the hovering jumpjet and the paths that reach it; scenario A12, parts d and e, for a healer's second pass; scenario A15, part e, run 3,
     for the allied objects an engineer of the neutral house considers). The objects considered by a
     searcher whose type attacks friendlies, that is berserk or that is such a passenger follow
     table C's rows "friendly" and "controller's allies", whose scenarios are named there. The deck
     read of the searcher's own cell is `unmeasured`: it differs from a read in the searcher's own
     layer only for a searcher under a bridge whose deck and ground lie in different reach
     regions, and which deck region S-0212 claim 4 then gives depends on bridge links that no
     scenario of this spec places at will. The no-link deck exception, on both the searcher and
     object sides and for visited area-search cells, is also `unmeasured`: no proposed fixture
     establishes a bridge cell with no matching link, so A11 does not test that exception.
     The give-up path: part c, run c3, for the target drop at a give-up and the queued *guard* that
     becomes current after it under a human-controlled owner, and part k, run 4, for a range search
     of *guard* reaching a hovering jumpjet. The original's *area guard* after a give-up is
     `unmeasured` here: S-0212 claim 23a can then give a computer-owned unit a new destination, so
     no fixture here keeps a hovering jumpjet within its search at will
   - Confidence: high (the ground-layer restriction, for a hunting vehicle whose weapon can hit
     aircraft, as parts f and k run 1 have), medium (the ground-layer restriction for other
     searchers, the zone test and the order of the passes),
     medium (the give-up path: the target drop at the give-up, the queued *guard* becoming
     current after it, and the reach of
     claim 9's airborne scan, rated medium there; the original's *area guard* after it),
     high (cells of no reach region), low (the deck read
     of the searcher's own cell and the no-link deck exception)

8. **Radius of range and area searches.** The radius *R* in leptons, and from it the number of
   rings *n*, follow Formulas F1. For a range search *R* is the type's guard range if that is not 0
   (and the searcher is not an engineer), and 0 otherwise. (In our engine the engineer exception
   takes effect in the range searches of claim 1's first and last rows only for an engineer of
   the neutral or special house, which claim 2 does not keep from them (claim 25, part a); claim
   2 keeps every engineer of a player house from those searches. It also takes effect in the
   attack-move searches of claim 23, which claim 2 does not limit, for an engineer of any house:
   an engineer with a weapon in its first slot can be given an attack move, S-0214 claim 5.) For an area search *R* is twice the guard
   range, or, when the guard range is 0, twice the longer of the ranges of the weapons in slots 0
   and 1, limited to 0 … 4096 leptons; in *patrol* it is limited to 1792 … 4096 instead. When *R*
   is 0 the rings come from the weapon ranges instead (F1) and a candidate must be within the
   reach of the weapon the searcher would use against it (claim 12). For a garrisoned building the
   number of rings is set from its foundation and the occupant range bonus (F1).
   - Backing: `measurable` (scenario A1, parts a, g, h and i; scenario A4, parts e and f, for the
     ring count from weapon ranges, including the air range bonus, the switching type and the
     underwater exception of F1; scenario A6, part k, for the engineer exception in an attack-move search; scenario A15, part e,
     run 4, for the engineer exception in the per-tick search of an engineer of the neutral
     house). The open-topped shortest range of F1 is owned by S-0307 claim 13 and inherits
     its backing; F1 reads the passengers' current weapons (Terms)
   - Confidence: high (guard range and weapon-range cases); medium (the garrison and patrol cases,
     the air range bonus, the switching type, the underwater exception and the engineer
     exception)

9. **Scan order: airborne objects, then rings outward, with early stops.** When a range or area search looks for aircraft, its airborne candidates are the *tracked flying objects* (S-0303, Terms) that the flying-object search of S-0303 (claim 6 and F7) finds around the centre cell with search radius max(*n*, 1) cells. On equal scores, an airborne candidate precedes a candidate from a cell. This is the same search, under the same switch
   `quirks.flying_search_sectors`, that area damage uses. It reads whole sectors: the sector of the
   centre cell, the runs of sectors along the four map axes and the diagonal groups of S-0303's F7. So it can
   miss a tracked flying object that is within the search's reach but lies in another sector, and it
   can find objects that are beyond it; the distance row of table C (F3) rejects those, so range
   is still decided by *R* (or the weapon's reach) and not by the sectors. Of the objects it finds
   only those that are on the map and not in the ground render layer are looked at, each once, with
   no one-object-per-cell rule (claim 10), and objects allied with the searcher only when its type
   attacks friendlies, it is berserk or it is a passenger of a mind-controlled open-topped
   transport. Equal-score airborne candidates are resolved by S-0303's F7 order: the centre sector, the east, north, south and west runs, then the four diagonal groups, and within a sector by the order in which the objects were registered there; registration during jumpjet movement follows S-0502 claim 29, including its placement and touchdown exceptions. In particular, a jumpjet becomes registered in its take-off update, normally at height 0 leptons, without waiting for a rise. Ordinary registration ends at touchdown, crash end or going off the map, not at a height threshold. With `quirks.flying_search_sectors` off, equal-score candidates are resolved by registration order among all tracked flying objects, each considered once.
   Except in a range search that looks for aircraft and nothing else, cell candidates are drawn from legal cells in rings around the centre cell, from ring 0 through at most ring *n* − 1. Equal-score cell candidates are resolved by ring order and then by the F2 order of legal cells within a ring. For *n* = 0 no ring is looked at, whatever the
   airborne objects hold. For *n* ≥ 1, let *r*₁ = trunc(*n* / 4) and *r*₂ = trunc(*n* / 2), and let
   *K*(*r*) be the candidates among the airborne objects together with those in rings 0 … *r*. The
   last ring looked at, ℓ, is the smaller of *r*₁ and *r*₂ for which *K* is not empty, or *n* − 1
   when *K*(*r*₂) is empty. (Since *r*₁ ≤ *r*₂ ≤ *n* − 1 and *K* only grows with *r*, ℓ is *r*₁ when
   *K*(*r*₁) is not empty.) The search chooses among *K*(ℓ). So a higher-scoring object in an
   outer cell is never reached, while an airborne candidate, scored before any ring, still competes
   on its score. A
   search whose radius is above 0 but below 256 leptons has *n* = 0 and looks at no cell at all
   (only at airborne objects, whose search radius is then 1 cell).
   - Backing: `measurable` (scenario A4; the sector effects by scenario A4, part d, and S-0303's
     scenario A16; jumpjet registration boundaries inherit S-0502 claim 29's backing and
     confidence, including scenario J28 and its stated unmeasured exceptions), except the rule
     that a radius of 1 to 255 leptons looks at no cell, which is
     `unmeasured`: it follows from F1 and the ring rule, and seeing it would need an enemy object
     within 255 leptons of the searcher in another cell, which no placement makes reliable. Also
     `unmeasured`: the rule that leaves out found objects in the ground render layer, because
     S-0502 claim 29 establishes registered jumpjets in that layer during low flight, but A4
     does not isolate their rejection by the airborne scan from their later cell consideration
   - Confidence: high for the ring order and the early stops; medium for the airborne scan (the
     set of tracked objects and F7 are S-0303's, claim 6)

10. **One object per cell.** Each visited cell offers the search at most one object, its *offered
    object*, and only that object can be a candidate from that cell. Which objects of the cell
    count: when the cell's bridge-deck layer holds any object, only the objects on the deck, and
    objects under the bridge are not looked at; otherwise the objects of the ground layer. Counting
    those objects in the order the cell holds them, and leaving out the searcher itself, the
    offered object is:
    - the first infantry unit, vehicle, aircraft or building that the searcher *selects*, by the
      first row of this table that fits the searcher:

    | Searcher | Selects |
    |---|---|
    | a healer for acquisition, whatever else holds for it | an object allied with it whose health ratio is below 1 |
    | a searcher whose type attacks friendlies, that is berserk or that has the permanent-berserk mark | any object |
    | a passenger of a mind-controlled open-topped transport | an object allied with the house that controls the transport |
    | an engineer of the neutral or special house whose current behaviour is *area guard* (claim 25, part d) | an object not allied with it, or a building allied with it whose health ratio is at most the red condition ratio and whose type accepts repair by it |
    | any other searcher | an object not allied with it |

    - when none of those objects is one that the searcher selects, the last of them, provided
      that it is an infantry unit, vehicle, aircraft or building; otherwise the cell offers none
      (low confidence for this fall-back). On a bridge cell whose deck holds an object, the
      fall-back is thus the last object on the deck.

    The offered object is tested against table C only when it is not allied with the searcher, or
    the searcher is a healer for acquisition, has a type that attacks friendlies, is berserk, is
    a passenger of a mind-controlled open-topped transport, or is an engineer of the neutral or
    special house (claim 25, part d). When it is not tested, or fails a
    candidate test, the cell gives no candidate, whatever else it holds. So for a searcher with
    the permanent-berserk mark an allied object that comes first in a cell hides the rest of that
    cell (`quirks.search_one_object_per_cell`, Edge cases).

    Two consequences. For an infantry or vehicle healer the fall-back object never becomes a
    candidate (it is either not allied, which row "healer" rejects, or allied at full health,
    which row "friendly" or the firing check rejects). For a passenger of a mind-controlled
    open-topped transport, a cell gives a candidate only when it holds no object allied with the
    controlling house, and the candidate is then the cell's last object
    (`quirks.mind_controlled_passengers_cell_choice`, Edge cases).
    - Backing: `measurable` (scenario A3: the main part for one object per cell, part b for the
      bridge rule, part c for the berserk searcher; scenario A15, part e, runs 2 and 3, for the row
      of an engineer of the neutral or special house, claim 25, backing; scenario A8, part f, uses a searcher whose type
      attacks friendlies; the passengers of a mind-controlled open-topped transport, including the
      fall-back to the last object, by scenario A14, part l); `unmeasured` for the fall-back of a
      building healer (claim 6, backing) and for the permanent-berserk mark, which nothing in our
      engine sets (Edge cases)
    - Confidence: high (one object per cell); medium (the bridge rule); low (the passengers of a
      mind-controlled open-topped transport and the fall-back to the last object)

11. **A cell in another reach region is skipped.** For a mobile searcher other than an aircraft, an
    area search (and a map-wide search, claim 7) skips every cell that is not in the same connected
    region of the searcher's movement zone as the searcher's own cell, except for claim 7's
    no-link deck case. A range search does not.
    Both the visited cell and the searcher's own cell are read for their decks when they are
    bridge cells, and the regions are compared as in claim 7 (S-0212 claim 4), so two cells of no
    reach region of that zone count as the same region. Claim 7's no-link searcher exception
    omits this restriction for every visited cell, including other no-link decks, rather than
    comparing two no-link reads under S-0212 claim 4.
    - Backing: `measurable` (scenario A11, parts b, d, e and j); `unmeasured` for the deck reads
      and the no-link deck exception, for the reasons in claim 7
    - Confidence: medium (low for the deck reads and the no-link deck exception)

### Which objects are candidates

12. **Candidate tests.** An object *T* is a candidate for searcher *A* only if every row of table C
    (State and ordering) allows it. The tests do not depend on each other's order, and none of them
    takes a random draw, except the disguise row for a searcher of the neutral or special house,
    which takes one ranged draw when the target's disguise is blinking (claim 25). Whether *T* is
    a candidate does not depend on the order of the tests; whether that draw is taken does, and
    claim 25, part g, says for which objects it is taken. The weapon tested is the weapon *A*
    would use against *T* (S-0308).
    - Backing: `measurable`, each row through the scenario part named in table C's last column,
      including the 0-health case of *exists* through scenario A14, part o: a destroyed building
      awaiting removal remains in its foundation cells with 0 health (S-0312, claim 21), but is
      not a candidate. The nonexistent and removed-from-play cases of *exists* remain
      `unmeasured`: no setup offers such an object to a search. Two other rows are `unmeasured`.
      *Self*: no setup offers a searcher itself as a target that the other rows would accept, so
      this row only makes the list complete. *Firing check*: every *illegal*
      case known to the combat specs is either also rejected by another row (a versus value of
      0 by the versus row, a class the projectile cannot hit by the class row) or belongs to
      feature specs not yet written, so no setup known here isolates the row. That no row takes a
      draw for a searcher of a player house is `measurable` by scenario A14, part n, which compares
      the random stream on the search tick of two runs that differ only in whether the objects in
      range are candidates. S-0215 (claim 33, part d) orders the search's draws after the
      processing's.
    - Confidence: as each row of table C gives

### Threat score and preference

13. **Threat score.** Each candidate gets the threat score of Formulas F4: a sum of terms weighted
    by the searcher's threat coefficients, for the candidate's danger to the searcher, the
    searcher's effectiveness against the candidate, the candidate's special threat value, its
    health ratio and its distance beyond the searcher's weapon range, plus a bonus when the
    candidate belongs to the searcher's owner's enemy house, plus 100000. The sum is converted to an
    integer (F4), and the final score is that integer under the two modifiers of claim 14 and the
    floor, composed as F9 gives: the suppression (F6) acts on the result of the weak-target
    preference (F5). A final score of exactly 0 rejects the candidate, and any other final score
    below 1 counts as 1 (`quirks.zero_score_rejected`). The composition is observable: a candidate
    with a converted score of 3 whose score claim 14 doubles and that has one suppressing building
    cell scores 3, not the 2 that the opposite composition would give (scenario A7, part h).
    - Backing: `measurable` (scenario A4 for the own-effectiveness term through the versus
      value; scenario A7, parts a, b and e, for the special-threat, distance and danger terms, part f for
      the base of 100000 and the conversion, part h for the composition; scenario A12, main part,
      for the strength term; scenario A9 for the enemy-house bonus; scenario A8 for the modifiers)
    - Confidence: high for the terms and their signs; medium for the distance term of the
      map-wide search (F4, quirk)

14. **Score modifiers.** Both act on the integer score, composed as claim 13 and F9 give (the
    suppression acts on the result of the weak-target preference).
    - *Weak-target preference.* For a searcher whose type's target-health scan mode is *normal*, a
      candidate whose expected remaining health is 0 or less has its score halved (truncated toward
      zero), and one whose expected remaining health is at most half its type's strength field
      (truncated toward zero) has its score doubled. Mode *strong* instead rejects candidates whose
      expected remaining health is 0 or less (table C). Mode *none* does neither.
    - *Suppression.* For a searcher whose *current weapon* (the weapon in the slot of its weapon
      number for every multi-weapon type, gattling types included, and in slot 0 otherwise) has the
      suppress flag, the score is multiplied by 1/2 for every cell in
      rings 1 … *N* − 1 around the candidate's cell that holds a building allied with the searcher,
      where *N* is the game-wide fire-suppression distance in cells (F6). A building is counted once
      per cell of it in those rings.
    - Backing: `measurable` (weak-target preference: scenario A8, part a, for the doubling and
      part g for the halving at 0 or less; mode *strong*: A8, part b; suppression: scenario A10,
      main part, for the halving per building cell, part b for the count per cell of a building
      with several cells, part c for a switching type's weapon number and part d for the distinction
      between a gattling type's current weapon and the weapon it would use against a candidate,
      and the composition: A7, part h). Suppression does not choose a gattling weapon against each
      candidate: its current weapon is the same weapon-number slot for all candidates of that
      search, even when S-0308 claim 7 would choose different firing slots
    - Confidence: high (weak-target preference); medium (suppression)

15. **Expected remaining health.** An object's expected remaining health starts equal to its health.
    It is lowered by the expected damage each time a search covered by claim 4 picks it as a target and each
    time a shot is fired at it (S-0302, claim 18). On every tick, in the object's own turn, it
    changes as F7 gives: a value above the health becomes the health, and on ticks whose tick
    number *t* has (⌊*t* / 4⌋ mod 2) = 1 (four ticks of every eight), a value below the health
    becomes one more than the larger of itself and −30. A paid repair step (a building repairing
    itself, or a unit serviced by a repair building, S-0314) adds to it the same amount that it adds
    to the health, and a step that brings the health to the strength field sets both to that
    field. Other healing (a healer's shot, self-healing, house-wide healing) changes only the
    health, so the expected remaining health then catches up only as F7 gives.
    - Backing: `measurable` (scenario A8: part b for the lowering and the recovery, part h for the
      floor of −30 and the rate of recovery, part f for the repair step, the full repair and other
      healing, part a, run 2, for a value above the health becoming the health)
    - Confidence: medium (low for the statement that other healing leaves it unchanged)

16. **Anger and the enemy house.** When an object takes damage from a known attacker, its owner's
    anger towards the attacker's owner changes by the increment Δ of F8, (*d* / *S*) × *C*
    converted to an integer, where *d* is the
    damage dealt (negative for a healing hit, which lowers anger), *S* the victim type's strength
    field and *C* the victim type's *type's cost* as S-0314 (claim 5) defines it: its cost field,
    adjusted for some building types by the production specs, and taken without any house or
    country cost factor. So *C* does not depend on the attacker or its owner, and two equal hits
    from attackers of two different houses give the same increment. After every change of an anger value, increase or decrease, the owner's enemy
    house is worked out again from the present anger values: it is the house, other than itself,
    that it is not allied with and that is not defeated, with the greatest anger above 0 (the
    earlier house in the house order on ties); if no house has an anger above 0, it has no enemy
    house. This holds for every house, human-controlled or not, so it also changes the target
    preference of a person's units (F4). The enemy house is worked out only then: in particular,
    when a house's enemy house is defeated, it stays that house's enemy house until one of that
    house's anger values next changes, and only that change moves it to another house
    (`quirks.defeated_enemy_house_kept`, Edge cases). An
    implementation that works the enemy house out whenever it is read would move it at once and
    gives different picks (scenario A9, part c). Two more rules change anger, each followed by the same
    recomputation: when a house is made an enemy of another house, its anger towards that house
    grows by 1 point of anger (F8) (low confidence for the amount when the two were allies before), and when it is
    made an ally, its anger towards the new ally becomes 0 and its enemy house is cleared if the
    ally was it. When houses are made allies or enemies belongs to the skirmish set-up and
    diplomacy specs. No other rule of our engine is known to change anger, except two rules for
    owners that are not human-controlled, which in our engine apply to the neutral and special
    houses only (claim 25, part j; see also Open questions).
    - Backing: `measurable` for an increase from damage and the recomputation (scenario A9, part
      a) and for a defeated enemy house staying the enemy house (scenario A9, part c). `unmeasured` for a decrease from a healing hit: it needs another house's unit to heal a
      unit of the searcher's owner, which the person cannot order for a computer-controlled house,
      and an infantry or vehicle healer accepts only allied objects (claim 6, table C row
      "healer"), while anger
      towards an ally never makes an enemy house. `measurable` for the tie rule and for *C* not
      depending on the attacker's owner (scenario A9, part d, where two equal hits from two houses
      give equal anger). `measurable` for the anger change when a house becomes an enemy or an ally: S-0414 (claims 12–14) permits alliance orders in skirmishes, and proposed scenario A9, part e, distinguishes the two changes through target preference. The initial alliance and anger state is still setup-owned; scenario A9, part b, records the initial enemy-house state without depending on it (Open questions).
    - Confidence: medium (high for the cost term, low for the tie rule and the alliance rules,
      medium for the defeated enemy house, since no rule of our engine that changes anger on
      defeat is known)

17. **The highest score wins.** A search picks the candidate with the highest threat score. On equal
    scores the candidate considered first wins, in the order of claims 7 and 9 (airborne objects
    before cells, inner rings before outer ones, and within a ring the cell order of F2; in a
    map-wide search, aircraft before ground-layer objects, whose order among themselves is not
    established, Open questions). A search with no candidate picks nothing. A distributed-fire searcher applies claim 20 instead (in a
    range or area search to the candidates, and in a map-wide search it picks nothing).
    - Backing: `measurable` (scenarios A4 and A7; scenario A11, part g, for the map-wide order of
      aircraft before ground-layer objects); `unmeasured` for ties between ground-layer objects of
      a map-wide search, whose order is not established (Open questions)
    - Confidence: high

### Retaliation

18. **When an object retaliates.** S-0215 (claim 18 and its Formulas F7) owns what retaliation
    does and its behaviour, owner and distance conditions (its claim 19, scatter, only refers to
    retaliation); S-0304 (claim 3) owns the versus condition. In
    addition, an object *A* retaliates against attacker *X* only when all of these hold:
    - its type's retaliate flag is set,
    - it is neither a harvesting slave nor a slave master, and it is not of a spawner type (Terms:
      the condition holds even while none of its spawned aircraft is launching or alive),
    - it is not a mind-controlling object that already controls as many objects as it can,
    - *X* is not allied with it, and *X* does not appear disguised to *A*'s owner,
    - its mean weapon damage is above 0 and it is armed,
    - the firing check (without range) for the weapon it would use against *X* answers neither
      *illegal* nor *impossible*,
    - if its owner is human-controlled and *X* is a building: it is neither an infantry unit whose
      type has the explosive-charges flag nor an object that has the explosive-charges veterancy
      ability (S-0311, claims 3 and 4),
    - if its owner is human-controlled and it is a vehicle whose type deploys into a building with
      the artillery flag: never,
    - *X* is not the parasite attached to it (S-0614 claim 17).

    Retaliation adds no condition of its own on where *X* is (every row of the firing check
    still applies: its water and land rows, and its first and second rows, which refuse an
    airborne attacker, or an infantry or vehicle attacker outside the ground render layer, to a
    projectile that is not anti-air; S-0306 claim 23): it has no connected-region test (claims 7
    and 11) and no render-layer test of its own. So an attacker standing on a cell of no reach
    region of *A*'s movement zone is retaliated against like any other attacker, and so is a
    hovering jumpjet by a defender whose projectile is anti-air; a defender whose projectile is
    not anti-air is refused the hovering jumpjet by those firing-check rows, as for any other
    attacker outside the ground render layer.

    For the neutral and special houses claim 25, part h, gives the conditions that differ.

    Retaliation adds no projectile-flag or search-class restriction of its own. Firing remains
    subject to the firing check and to the actor's behaviour rules (S-0215). For a ground
    attacker this gives three cases:
    - an object whose type has land-targeting mode 1 never fires back at a floor-bound attacker
      (S-0307, Terms) that is on a bridge or on a cell that is neither water nor beach (S-0306
      claim 23's land row), and the same claim's water row likewise stops any object firing back
      at an attacker on water or beach (not airborne and not on a bridge) when its type's
      naval-targeting mode gives "none" for that attacker (S-0308, table N). In S-0301 claim 9,
      *rearming* is rank 5 and the two rows are at rank 4 (S-0306 claim 23), so both rows rank
      above the *rearming* of a running rearm countdown, and the refusal holds whatever the
      object's rearm countdown. Their rank
      against *out of ammunition*, *cloaked* and *out of range*, and their place within rank 4,
      are as S-0306 claim 23 gives and rates them.
    - a building whose type has the SAM flag (S-0215 Terms) never fires at a ground attacker
      that is not S-0215 claim 32's suitable target (an aircraft whose world height is not 0),
      whatever its land-targeting mode or projectiles: its *attack* processing fires only at
      S-0215 claim 32's
      suitable targets.
    - any other object whose weapons' projectiles are anti-air only is refused a ground attacker
      by none of the conditions above for that reason; each of them still applies as for any
      other attacker. This covers a type with land-targeting mode 0 or 2 (for a building, with
      no SAM flag) against any ground attacker that the water row does not refuse (an attacker on
      water or beach is refused only when the type's naval-targeting mode gives "none" for it).
      It also covers a type with mode 1 against an attacker that stands on a water or beach
      cell, not on a bridge, when its naval-targeting mode does not give "none" for that
      attacker: the land row covers no such cell and the water row refuses only when the mode
      gives "none".

    The firing check refuses an airborne attacker to a projectile that is not anti-air, except
    when the attacker is the object's own gripped unit, for which it answers *rearming* unless an
    answer that S-0301 claim 9 ranks above it comes first (S-0306 claim 23; S-0607 claim 11).

    **Q-1763, question 1.** Retaliation refuses no attacker by its weapon's anti-ground or
    anti-air flags of its own. Its flag-related refusals are two. The first is the firing
    check's (S-0306 claim 23, with its answer to this question), the same for infantry,
    vehicles, buildings and landed aircraft as attackers: none of them is refused by the
    projectile's anti-ground flag, and each is refused by the land and water rows only where the
    firer's targeting modes give a refusal for an object on its cell. The second is the SAM flag
    of a building's type (the second case above): its *attack* processing fires only at S-0215
    claim 32's suitable targets, whatever the attacker's kind.

    **Q-1763, question 2.** Yes, there is such a case: a building of the third case above against
    an infantry or vehicle attacker in the ground render layer (a search for aircraft does not
    pick infantry or vehicles, claim 5, and the scan of airborne objects looks only at objects
    outside that layer, claim 9; table C), including a building with land-targeting mode 1 against such an attacker on water or beach for
    which its naval-targeting mode does not give "none". Its
    automatic searches keep to aircraft (claim 5), and a person's attack order on a ground object
    gives it nothing (S-0214 table B, row B7, which tests the anti-ground flag), yet it fires back
    at a ground attacker. So retaliation cannot be bounded by the eligibility of the search and
    the order. A unit of the third case is not such a case through the anti-ground flag: no order rule tests
    that flag for it (S-0214 claim 6(e), which defers to the firing check of S-0306 claim 23), so a
    person's order does not refuse it for that reason either (S-0306 scenario P16 part h). This
    half of the answer needs the S-0214 T4 part C (e) amendment that S-0306 lists under Changes
    for other specs. No other case is established beyond the third case, but the search for one
    was not exhaustive (Open questions). The play report of Q-1763, that an anti-air-only
    defence building never fires at a ground attacker, would follow from the first two cases if
    the stock building's type meets one of them; that is a hypothesis, which scenario A5 part b
    checks. Which case the stock building falls in is not established (Open questions).

    **Q-1761, question 1, out of range and computer owners.** The refusals of the first two cases
    depend neither on distance nor on owner: the land and water rows test neither, and nor does
    the SAM-flag drop. Distance and owner enter only through the conditions that apply to every
    attacker alike: S-0215 claim 18's behaviour list, existing-target and distance conditions
    for a human-controlled owner, which in our engine apply to every player house, human or
    computer-controlled (S-0215 claim 18), and claim 25 part h for the neutral and special
    houses. (S-0215 claim 18 gives how the original treats computer-controlled houses; the first
    two cases' refusals hold for every owner either way.) So, in our
    engine, the answer is the same for an attacker out of range and for a
    building of a computer-controlled player house: a building of the first two cases never fires back at a ground attacker that those
    cases refuse (for the SAM-flag building, one that is not S-0215 claim 32's suitable target).
    A building of the third case
    retaliates when those common conditions let it; against an attacker beyond its weapon's
    range it then drops the target without firing, at the first processing in *attack* at which
    the firing check answers *out of range* (S-0215 claim 32; while its countdown runs the answer
    is *rearming*, and it rotates).
    - Backing: `measurable` (scenario A5, including the veterancy-ability variant of the charges
      row and part b for the targeting modes and, in its Variant 3, the drop of an attacker
      beyond the third case's range; scenario A11, parts j and k, for attackers on a cell of no
      reach region and hovering jumpjets against an anti-air defender; a defender without anti-air
      projectiles is refused the hovering jumpjet by S-0306 claim 23's layer row, backed there).
      `unmeasured` for the third case's mode-1 defender
      against an attacker on water or beach: in A5 part b
      and S-0306 scenario P16 the attackers or targets of mode-1 objects stand only on land and
      bridges and are floor-bound.
    - Confidence:
      - high: the flag, slaves, spawners, alliance and mean damage.
      - the firing check apart from its layer, water and land rows and the absence of its
        anti-ground test: as S-0301 claim 9 rates each of its answers.
      - medium: the firing check's layer row, as S-0306 claim 23 rates it.
      - medium: no return fire on land by a type with land-targeting mode 1, as S-0306 claim 23
        rates its land row (A5 part b and S-0306 scenario P16 parts f and g have not run).
      - medium: the SAM-flag case, as S-0215 claim 32 rates the SAM *attack* stages.
      - medium: mind controllers, the charges and artillery rows, and the parasite row.
      - medium: the water case for naval-targeting modes 2 and 6.
      - low: the third case's mode-1 defender against an attacker on water or beach
        (`unmeasured`).
      - low: that the land and water rows' refusal holds whatever the object's rearm countdown,
        which rests on S-0306 claim 23's rank, rated low there against *rearming*.
      - medium: the answer to Q-1763 question 1, which rests on S-0306 claim 23's absence of the
        anti-ground test, rated medium there.
      - medium: that the conditions above do not refuse a ground attacker of the third case,
        and the answer to Q-1761 question 1, which rests on it.
      - medium: the third case's drop of an attacker beyond its weapon's range without firing.
      - low: the answer to Q-1763 question 2, which also rests on S-0214 table B row B7, rated
        low there.
      - low: the water case for naval-targeting mode 0, which S-0306 claim 23 leaves
        `unmeasured`.

### Target switching

19. **An opportunity target is replaced when it becomes unusable.** Because of claim 4, an object
    whose opportunity target becomes illegal, moves out of range or becomes a target the firing check
    answers *impossible* for drops it (unless the answer is *impossible* and it controls spawned
    aircraft, claim 4) no later than its next search, and that search picks a new target. When the
    answer is *illegal* or *impossible*, claim 29's periodic check drops it at a check tick if that
    comes first, for a holder that claim 29 covers; claim 4 gives where a spawner's keep ends.
    An object whose target came from an order keeps it through searches covered by
    claim 4, except the attack-move searches of claim 23 that run as for an object with no target.
    The charging break-off search has its separate replacement rule in claim 1 and S-0518 claim 15.
    The behaviour rules of S-0215 (claims 18, 26 and 33) add the other cases in which a target is
    dropped: retaliation replaces the target, the area-guard leash drops it, and some behaviours
    drop an opportunity target.
    - Backing: `measurable` (scenario A6: part a for a target out of range, part m for one that
      becomes illegal). `unmeasured` for a target the firing check answers *impossible* for: its
      drop at a search is claim 4's (`unmeasured` there), and run p5 of part p judges only the drop
      at a check tick, for an ordered target.
    - Confidence: high that the target is gone by the next search and that this search picks a
      new one, for a target out of range or illegal; medium for one the firing check answers
      *impossible* for. The earlier drop at a check tick is rated in claim 29 (part m judges its
      tick).

20. **Distributed fire.** A searcher whose type has the distributed-fire flag does not pick by
    claim 17. In a map-wide search it collects no candidates and so picks nothing and empties its
    fired-at list, however many objects the search could have picked 
    (`quirks.hunt_ignores_distributed_fire`, Edge cases). In a range or area search, with *C* the candidates of that search (each with its
    score, in scan order) and *L* its fired-at list before the search:

    | Case | Pick | Fired-at list after the search |
    |---|---|---|
    | *C* is empty | nothing | empty |
    | some candidate in *C* is not on *L* | the highest-scoring candidate not on *L* (ties: the first in scan order) | the objects of *L* that are in *C* |
    | every candidate in *C* is on *L* | the highest-scoring candidate in *C* (ties: the first in scan order) | empty |

    Each time such an object fires
    while its current behaviour is not *attack* (a building: in any behaviour), the target is added
    to its fired-at list and the object drops it, so that its next search picks again.
    - Backing: `measurable` (scenario A7, parts c and d; part c, runs 2 and 3, for the condition on
      *attack* and for buildings)
    - Confidence: medium

21. **Fire-once weapons drop their target.** An object that fires a weapon with the fire-once flag
    drops its target after the shot.
    - Backing: `measurable` (scenario A6, part c)
    - Confidence: medium

22. **A blocked approach limits the next search.** When a route search of a vehicle with the
    driving locomotor fails and the vehicle does not give its move up, and the vehicle is then
    standing still and holds a target that is not within reach (claim 23), the vehicle drops the
    target and gets the limited-scan mark. Giving the move up means here either S-0212 claim 23's
    row 2 (a failure more than 1 cell from the destination), which drops the target itself
    (claim 7) and sets no mark, or the give-up of S-0207 claim 15 in front of a stationary
    friend, which sets no mark either. Every other failure that leaves the vehicle standing
    still sets it, for example a failure within 1 cell of the destination (S-0212 claim 23,
    row 3). While the vehicle has the mark, its area and map-wide searches are range searches.
    A search that finds a target does not clear the mark, nor does the death or removal of the
    target. Only a search that finds nothing clears it. Vehicles with the hovering or ship
    locomotor also get the mark at some of their route failures, under conditions that were not
    examined.
    - Backing: `measurable` (scenario A11, part c: run c1 for the drop, the mark and its clearing by
      a search that finds nothing, run c2 for the mark kept through a search that finds a target
      and through that target's removal). `unmeasured`: the absence of the mark at either
      give-up, since after a give-up of row 2 the vehicle's searches in *guard* are range searches
      with the mark or without it, and no run of part c makes the vehicle stop in front of a
      stationary friend; and the hovering and ship locomotors, which no run of part c uses
    - Confidence: medium for the drop, the mark and its clearing (run c1), medium for the mark
      kept (run c2), medium for no mark at either give-up; low for the hovering and ship
      locomotors (which situations make each locomotor's route search fail belong to the
      movement specs)

23. **Attack-move searches and resumption.** An object with an attack-move memory (S-0213, claim 6)
    acts on it in two places. A target is *within reach* when the range test of S-0307
    (Terms, *range test*) answers yes for it with the weapon the object would use against it
    (S-0308), and *engaged* is the mark of Terms.

    *(a) On each expiry of its targeting timer* it acts as this table gives.

    | Memory holds | State | Result |
    |---|---|---|
    | a destination | engaged, and its target is within reach | nothing happens |
    | a destination | engaged, and it has no target within reach | a range search runs as for an object with no target, so its present target is not kept; if it then has a target, it queues *attack* with an immediate start and counts as engaged; if not, it is left with no target and is still engaged |
    | a destination | not engaged | it runs a range search, in which a target it has is kept unless it is an unusable opportunity target (claim 4); if it then has a target, it queues *attack* with an immediate start and counts as engaged |
    | only a target | the memory's target is within reach, or it is engaged and its present target is within reach | nothing happens |
    | only a target | any other | a range search runs as for an object with no target, so its present target is not kept; if the search finds a target, it queues *attack* with an immediate start and counts as engaged; if not, its target becomes the memory's target again |

    These searches are not limited by claims 2 and 3, and the targets they find are not
    opportunity targets. For a spawner, running as for an object with no target does not itself
    request a pending-target clear: pending is unchanged whether this search finds a replacement,
    leaves the object targetless or restores the memory's target. Subsequent shots, target-loss
    notifications and controller resets remain governed by S-0506 claims 3, 11 and 25.

    *(b) On a fall-back to rest*, among the common effects of S-0215 claim 13, the memory is
    refreshed at most once per actor per tick. If the object is engaged, or its current behaviour
    is *guard*, the attack move is resumed as the table below gives. Otherwise the memory is
    removed, including its destination, target and engaged mark. This refresh is not part of
    the per-tick start of S-0215 claim 11. The infantry or vehicle rest table of S-0215 claims
    14 and 15 judges the state after the refresh: its row 1 applies only if the memory survives.
    S-0215 claim 13 also specifies how the common fall-back destination rows and route-generated
    orders interact with this refreshed state.
    Resuming the attack move ends the engaged mark and has this effect:

    | Memory holds | Result |
    |---|---|
    | a destination | *move* is queued with an immediate start, and the memory's destination becomes the object's destination |
    | only a target | *attack* is queued with an immediate start, and the memory's target becomes the object's target |
    | neither | the memory is removed; nothing else happens |

    *(c) The engaged mark ends* when the attack move is resumed (part b), a new attack-move
    order is applied (S-0213, claim 6), or the memory is removed. A search that finds nothing
    never ends it, and nothing else sets it. An engagement therefore survives later tick starts:
    while the held target remains within reach, a targeting-timer expiry leaves it engaged,
    makes no search and does not restart the targeting timer. When the target is no longer
    within reach, the engaged rows of part (a) can search again. Ending an engagement through
    a fall-back resumes the remembered destination or target rather than forgetting it.

    *(d) Arrival and loss of the remembered target (Q-0162).* Reaching a destination does not
    itself remove the remembered destination. In the uncontested case, the object arrives in
    *move*, has no current destination, is not moving, has nothing queued and is not engaged.
    Its next *move* processing falls back to rest and chooses an integer delay of 1 tick
    (S-0215 claim 22). Part (b) removes its memory during that fall-back, so the ordinary rest
    rules apply: the object queues *guard* or *area guard* as S-0215 claims 14 and 15 decide,
    without an immediate start. It does not remain indefinitely in *move* with the memory,
    and does not repeat that arrival delay every tick indefinitely. If another fall-back already
    used the common effects on the same tick, the refresh waits for a later fall-back.

    When the object remembered only a target, removal of that target changes the memory to a
    destination: the cell of the target's last position. The remembered target is cleared, but
    the memory and its engaged mark are kept. A fall-back while engaged resumes *move* to
    that cell by part (b). Arrival there without a further engagement ends the memory as above.
    If the actor is not engaged and falls back while still in *attack*, part (b) removes the
    memory instead of resuming it. A destroyed object that still exists is not yet a removed
    target for this rule (S-0312 and S-0203 own removal). No separate attack-move arrival
    timeout or search-radius extension exists in these cases: an enemy beyond the later resting
    object's search area does not by itself cause a new advance.
    - Backing: `measurable` (scenario A6: part d for the not-engaged destination search,
      part g for engaged searches and resumption, including its run 2 with a remembered target,
      part k for F1's engineer exception, part l for targets not being opportunity targets,
      part n for arrival, memory removal and the 1-tick delay, and part o for conversion of a
      removed target to its last cell and the subsequent fall-back and arrival). The once-per-tick
      limit inherits S-0215 claim 13's backing. The spawner pending-target consequences are
      `measurable` by scenario A6 part i and S-0506 scenario R19.
    - Confidence: high for the refresh context and removal when neither engaged nor in *guard*,
      medium for the searches, resumption, arrival consequences and removed-target conversion

24. **Redirected targets.** Setting a target redirects it in three cases:

    | Target set | Target the object holds |
    |---|---|
    | a unit sitting in a tank bunker, when the object has a temporal weapon, its primary weapon's warhead is a locomotor warhead, or its type can call airstrikes (it has an airstrike team, Import mapping), whether or not an airstrike is in progress | the bunker building |
    | the object itself | its own cell |
    | an object that is dead or sinking, or an infantry unit whose current sequence is a death kind (S-0107: die 1 to 5, wet die 1 and 2, the three air-death kinds) | no target |

    The bunker redirection also applies to temporal delivery (S-0514 claim 1).
    Any other target is held as set. The second and third rows judge the target as the first row
    leaves it. So a dead or sinking unit sitting in a tank bunker, set as the target of an object
    that the first row covers, gives the bunker building, and the third row then judges the
    building: the object is left with no target only if the building itself is dead. For a spawner,
    setting a target that leaves it with none requests the conditional pending-target clear of
    S-0506 claim 25, not an immediate clearing of its spawned aircraft's targets. With the original
    quirk setting, pending is cleared only if the controller target is nonempty. Explicitly
    clearing an already-empty own target is a no-op, but assigning a nonempty target that is
    redirected to no target still requests the conditional clear, even if the old own target
    was empty. A replacement that remains nonempty does not itself update pending.
    Controller adoption, reset and loss notifications remain S-0506 claims 3, 11 and 25.

    Every setting of a target, even to the same one, leaves a target that is not an opportunity
    target, with one exception: a target that the per-tick range search of claim 1's first row
    (after any redirect above) or the teleport search of claim 1 sets, and that is a different
    target from the one the object held before that search (an object, or a wall target of claim
    25, part i), is an opportunity target. No other search marks one: not the attack-move search of
    claim 1's second row (claim 23), nor the area search or the map-wide search. A search that keeps its target sets none, and the mark
    stays as it was. (A spawned aircraft that is not a missile has one further case in which a new
    target leaves it with none; the aircraft specs own that case.) Every change of target of an
    object that can call airstrikes cancels its strike, whether or not the strike is in progress.
    Setting the same target again does not cancel it. S-0519 claim 19 owns this cancellation.
    - Backing: `measurable` (scenario A6: part e for the temporal weapon and the locomotor warhead,
      part j for the airstrike, with run 2 for an airstrike in progress, part h for the own-cell
      and no-target cases, the sinking target included, part i for the spawned aircraft, part a for
      the opportunity mark); strike cancellation inherits S-0519 claim 19's backing, with
      S-0519 scenario T18 part b covering cancellation after a strike has ended; the opportunity
      mark of a wall target is `unmeasured` (claim 25,
      backing); that the third row judges the building that the first row gives is `unmeasured`,
      because no setup is known in which a target is set to a unit that is still in a tank bunker
      while it is dead, sinking or in a death sequence
    - Confidence: medium (high for the order of the rows)

### Owners that are not human-controlled

25. **Computer-owner acquisition rules.** In our engine the rules below apply to the neutral
    and special houses and to explicitly marked test-only computer-controlled player houses
    (Terms, *Test-only exception*). They are the original's rules for every owner that is not
    human-controlled; for every other player house the other claims apply unchanged. In this
    claim, “these houses” includes those marked test houses. Difficulty index 1 and the second
    list entry describe the neutral and special houses only; test houses use their oracle
    difficulty index. Each part names the rule it changes.
    - *(a) Passive acquisition (claim 2).* An engineer of these houses is not excluded.
    - *(b) Search classes (claim 5).* A vehicle whose type deploys to fire and an object whose type
      has the no-auto-fire flag search like any other object. An infantry unit whose type has the
      explosive-charges flag, or that has the explosive-charges veterancy ability (S-0311, claims 3
      and 4), also searches for buildings.
    - *(c) Engineers (claim 5).* The searches of an engineer of these houses also look for
      buildings, and every candidate must then be a capturable building (table C, row "capture").
      An unarmed engineer of these houses that is not a vehicle thief searches only when its type
      has the infiltrate flag, and then finds only capturable buildings. Besides, when its house
      has chosen a building for its engineers and that building lies less than 3840 leptons from
      the engineer (ρ₃ of Formulas between their reference points: the table square root of
      S-0204 on the squared 3-D difference, truncated, with no building reduction), the engineer
      takes that building as its target at once, without
      any test of claim 12 and without scoring. Choosing that building is a task of the original's
      computer player, and no rule of our engine makes these two houses choose one, so in our
      engine this last rule never takes effect (low confidence; Open questions).
    - *(d) Allied objects for engineers (claims 7 and 10).* For an engineer of these houses, claim
      10's per-cell selection, when its current behaviour is *area guard*, selects, besides the
      objects not allied with it, a building allied with it whose health ratio is at most the red condition ratio and whose type accepts
      repair by it (the structures specs own that test); an allied offered object is tested against
      table C like an enemy one; and the map-wide search also considers allied objects. Table C's
      rows "friendly", "engineer" and "capture" then decide.
    - *(e) Rows "passive house" and "insignificant" of table C.* These rows do not reject outright
      for a searcher *A* of these houses. *T* passes either row only when *T* is a building, *A* is
      an infantry unit, and at least one of these two holds:
      - capture by an engineer of another owner: *T*'s type needs an engineer to be captured, *A*
        is an engineer, and *T*'s owner is not *A*'s owner, or
      - occupation: *A*'s type can occupy buildings and *T* can be occupied by *A* (the structures
        specs own that test).
    - *(f) Row "harmless building" of table C* does not apply to a searcher of these houses.
    - *(g) Row "disguise" of table C.* For a searcher of these houses, *T* that appears disguised
      to *A*'s owner, when *A*'s type cannot detect disguises, is rejected unless *T*'s disguise is
      blinking (the disguise specs own when it blinks) and one ranged draw *R*(0, 99) from the
      scenario random stream, taken for this test, is at most the house's entry of the game-wide
      *disguise-detection list* (Terms, Import mapping). The entry is selected by the house's difficulty index from the list in easy, normal, difficult order. The neutral and special houses have difficulty index 1, which selects the second entry (S-0412, claim 5). In our engine this index-based list selection remains the same; it does not mean that a difficulty row is applied to either house. The draw is taken only for an object
      that every other row of table C lets through, except the rows "capture", "engineer",
      "storage", "bridge layers", "healer" and "score": an object that one of those six rows
      rejects has still taken the draw, and an object that any other row rejects takes none (row
      "harmless building" never applies to these houses, part f). When one search tests several objects whose disguises blink, each qualifying object takes its own draw. Draw assignments follow the search's candidate order (claims 9 and 10; for the map-wide search, claim 7's order, which is partly open: Open questions), and an object beyond the point where the search ends takes none. Consequently, in a range search with *n* of at least 8 (its first early stop comes after ring 2 or later), with one blinking disguised object in ring 1 and one in ring 2, the first draw after the timer draw decides the object in ring 1 and the second draw the object in ring 2.
    - *(h) Retaliation (claim 18).* The charges and artillery conditions of claim 18 do not apply.
      Instead an object of these houses does not retaliate while it is draining an object with a
      draining weapon, and, when its target is an infantry unit, vehicle, aircraft or building, it
      retaliates only when the attacker's *threat sum* is at least its target's. The threat sum of
      an object *T* is the argument of conv in F4 for the retaliating object as *A*, with *d* in
      cells as in a range search; the two sums are compared exactly, in double precision. The
      behaviour, owner and distance conditions are S-0215's (claim 18); its rules for
      computer-controlled owners hold for these two houses by the same decision, as S-0215 (claim
      18) states.
    - *(i) Enemy walls (claims 9 and 17; table Search outcome and target state).* This part applies to the range and area searches of a searcher of these houses when the destroy-walls entry selected by its difficulty index is set (in our engine, the second entry selected by index 1), and the search looks at rings (claim 9). This per-difficulty-list selection does not mean that a difficulty row is applied to either house. The map-wide search looks at no walls, and a
      distributed-fire searcher, which picks by claim 20 among candidate objects, never takes a
      wall target.
      A *wall candidate* is a cell that the search looks at before it has any candidate object,
      that holds a wall owned by a house not allied with the searcher, and that is within the
      reach (S-0307) of the searcher's current weapon, when that weapon has a warhead with the
      wall flag and an anti-ground projectile (or no projectile). "Before it has any candidate
      object" counts in the scan order of claim 9: a candidate among the airborne objects, which
      come before every cell, leaves the search with no wall candidates at all, and the cell that
      offers the first candidate object is not itself a wall candidate, nor is any cell after it.
      Its *wall score* is given by F10. Let *r* be the first ring that holds a wall candidate with
      a wall score above 0, and *W* the one of those cells in ring *r* with the highest wall score
      (on ties, the first in scan order). The search's target is then:

      | Case | Target |
      |---|---|
      | no ring holds a wall candidate with a wall score above 0 | as claims 9 and 17 give |
      | *r* is an early-stop ring of claim 9 and a candidate object has been found by the end of ring *r* | the object that claims 9 and 17 give |
      | any other case | the cell of *W*, a wall target (Terms), also in preference to a candidate object that comes later than *W* in ring *r* (`quirks.wall_cell_over_same_ring_object`, Edge cases) |

      In the last case the search ends with ring *r*: no later ring is looked at, so neither a
      candidate object nor a wall candidate in a later ring can replace *W*, whatever its score.
      A wall target set by the per-tick or the teleport search is an opportunity target, as claim
      24 gives for a target different from the one held before.
    - *(j) Anger (claim 16).* A house of these two whose country is not marked passive for
      multiplayer has the two further anger rules of the original's computer player. From time to
      time (the interval is not established), when it has no enemy house, its anger towards the
      nearest other house that is neither multiplayer-passive nor defeated grows by 1 point of anger (F8) (nearness
      between the houses' base positions, details not established; an allied house gains the
      anger but cannot become the enemy house). At the same times, when its enemy house is
      defeated, its anger towards that house is removed and it has no enemy house until one of
      its anger values next changes. A house whose country is marked passive has neither rule.
    - *(k) Allied held targets (claim 29).* In each of its turns that is not cut short (claim 29,
      *Turns cut short*) and is not a warp turn of claim 29's *Warp members* rule, a holder of
      these houses whose target is allied with it, its own house included, drops the target.
      The drop comes at the place in the turn that claim 29 gives its check (*Place in the
      turn*), ahead of the check where one is made, and it is made whether or not the turn makes
      that check: in a tick of any number and in every behaviour, *capture* and *sabotage*
      included. The drop has the consequences of claim 29's (*The drop*): no timer draw, no
      revert and no change of behaviour by itself. For a passenger of an open-topped transport
      that is under mind control, both the owner test and the alliance are those of the house
      that controls the transport. The target is kept when any of these holds:
      - the holder is berserk (S-0610),
      - the holder is an infantry unit, the target a building, and the holder can occupy it (the
        structures specs own that test),
      - the holder is an infantry unit of an engineer type,
      - the holder's second weapon is an electric-charge weapon (S-0518) and the target is a
        building of an overpowerable type (S-0518).

      A holder of a human-controlled owner has no such drop: in our engine, the holders of every
      player house keep an allied target here.
    - These houses have no team members (S-0215, Terms), and campaign missions are out of scope, so
      the original's rules for those do not arise.
    - Backing: `measurable` through scenario A15, whose part a backs this claim's part f, part b
      the deploy-to-fire and no-auto-fire rules of part b, part c part g (its run 3 the single
      draw, its runs 4 and 5 the objects for which the draw is taken, its run 6 the
      order of the draws across objects), part d the charges row of
      part h, part e parts a, c and d (its run 4 the engineer exception of claim 8), part f the
      threat sum and draining condition of part h (run 3 isolates draining with no target,
      since a drain attempt clears the drainer's target: S-0608 claim 1), part h, case ii,
      that the draw of part g follows the timer draw, and
      part g part i:
      runs 1 and 2 the flag, run 3 the quirk, run 4 the early-stop row,
      run 5 the end of the search with ring *r*, run 6 the airborne candidate, run 7 the
      distributed-fire searcher.
      `unmeasured`: the charges rule of (b), because every setup known gives such a unit either an
      anti-ground weapon, which gives the building class anyway, or no usable weapon, whose firing
      check this spec does not establish; the chosen building of (c), which never takes effect in
      our engine; the two alternatives of (e) that let a target through (an engineer capturing from
      another owner, an occupier), because they need a target whose owner is multiplayer-passive
      and is not the searcher's own house: either the other non-player house, whose alliance with
      the searcher's house belongs to the set-up specs, or a player house whose country the test
      rules mark passive, whose wider effects on a skirmish (victory, the lobby) no spec gives, so
      no setup with a known expected result is known (the rejection by (e) is measured by scenario
      A15, part e, runs 2 and 3); the rule of (i) that the cell offering the first candidate object
      is not itself a wall candidate, because no object stands in a cell that holds a wall, so no
      setup can put both in one cell; the opportunity mark of a wall target in (i), because a wall
      target stays within reach unless the searcher moves, and an order that moves it also replaces
      its target, so no setup known separates an opportunity wall target from a kept one; (j), because the
      interval and the nearness measure are not established, so no expected tick or house can be
      stated. Part k is `measurable` through scenario A6 part p, run p9b, for the drop by a
      vehicle holder of a target of its own house and, by that run's control, for the keep by a
      vehicle holder of a human-controlled owner, and `unmeasured` for its exceptions and the
      transport rule, since no part here gives a holder of these houses an allied target under
      one of them
    - Confidence: medium for parts a, b, e, f, g, i and k (part k's exceptions and transport rule
      included); low for parts c, d, h and j, and for the opportunity mark of a wall target

26. **Initial threat coefficients and layers.** Each of a newly registered type's five threat
    coefficients starts at double-precision 0, independently of the game-wide defaults. Whenever
    a layer reads that type's section, an absent coefficient keeps its held value, except that
    with `quirks.coefficient_zero_replaced` on, a held 0 takes the matching game-wide default as
    it stands at that reading. A nonzero value copied this way is not updated just because a
    later layer changes the game-wide default. With the switch off, a coefficient never supplied
    by any layer stays 0. A type section that is not read in a layer changes no coefficient
    (S-0006 owns when sections are read).
    - Backing: `measurable` (scenario A16)
    - Confidence: high

### A held target that leaves play

27. **A held target is dropped when it is destroyed, goes off the map or is removed (Q-2006).** When
    an object is destroyed, goes off the map or is removed (glossary, *in play … destroyed,
    removed*), every infantry unit, vehicle, building and aircraft that holds it as its target drops
    it at that moment, inside the action that destroys it, takes it off the map or removes it. A
    destroyed object is dropped at the killing hit, also when it stays in play after that hit (a
    sinking or crashing object, an infantry unit playing its death sequence). An object that starts
    to sink without being destroyed is S-0504 claim 5's, which leaves open when a target held before
    sinking is dropped. An object crushed under S-0210 claim 4 is dropped during the crush. These
    drops have no exception for the holder's owner or for what covers the object's cell (claim 28's
    drops have such exceptions). No holder keeps such an object until its own next turn, and none
    fires at the object's last place, unless a revert gives it the object again (below).
    Consequences:
    - A holder whose turn in that tick comes after the drop (S-0203 claim 3), and which no revert
      (below) and no search in its own turn (claim 1) gives a target, has no target in that turn.
      Its firing step asks no firing check and fires nothing (S-0310 claim 9 for a vehicle). This
      is a firing step that begins after the drop. So a target destroyed earlier in the tick by
      another firer, as S-0306 scenario P27 sets up, gives that holder no shot in its later
      firing step. S-0306 claim 3's aim without a target is left to shots that leave while the
      firer holds no current target for another reason (Changes for other specs). Its
      rearm countdown runs on unchanged, and its behaviour then does what it does with no target.
      A building whose animation-delayed shot (S-0301 claim 14) is pending makes, in the turn that
      ends the delay, no firing check and no shot if it then holds no target, and the shot is no
      longer pending. A target that a revert or a search has given it by then is the one that
      check uses.
    - Within a vehicle's or infantry unit's turn, its ordinary movement comes before its separate
      firing step (A18 part a for a crushing vehicle; `unmeasured` otherwise, as rated below). A
      crusher that drives over the object it holds as its target therefore drops it in its movement
      and, unless a revert (below) or a search in its own turn (claim 1) gives it a target, makes no
      shot in that tick's firing step. Unless a revert gives the object back (Open questions), it
      does not fire at the object or at its cell.
    - No holder asks the firing check about a removed object, so no answer for such a target
      is needed (S-0301 claim 9).

    The objects that go off the map include one that boards a transport, open-topped or not, and
    one that enters a building. A vehicle that enters a vehicle bunker stays on the map (S-0606
    claim 7), so its holders keep it. The removals include a dropped unit removed at the end of
    its fall (S-0607 claim 18) and, taken to end like every other removal, erasure by a trigger,
    an exit at the map edge and an engineer used up by its action.
    A member of a superweapon warp group goes off the map at its departure, but this claim's drop,
    its timer reset and its rule for a projectile in flight do not apply to the departure, then or
    while it waits to land (S-0513 claim 39). A member destroyed by its landing check is a destroyed
    object here. Among the holders to which claim 29's rules apply, a holder that holds the member
    through its departure is rare: only claim 29's exceptions, or a re-taking after the holder's
    last check, let such a holder hold the member at departure (S-0513 claim 39).
    A projectile already in flight towards the object loses it as S-0306 claim 17 gives for a
    target that leaves the game or becomes untargetable: its target becomes the cell the object
    stood in, or nothing if the object was in the air. It never flies on towards the object.

    A holder whose targeting timer (S-0215 claim 33) has more than 10 integer ticks left when it
    drops the object resets that timer with one ranged draw from the scenario random stream over
    4 to 8 ticks, both included, as at the phasing-in end of claim 1. With at most 10 ticks left
    no draw is taken. This reset belongs to the drops of this claim and of claim 28. It is not part
    of a search (Observable ordering). When several holders drop the same object at once, they
    take their draws in creation order (Terms), the holder created first drawing first, whatever
    their update order (S-0203 claim 3) or their distances from the object.
    When one crush removes several objects (S-0210 claims 3 and 4), it removes them one at a time
    in the cell layer's listing order (S-0209 claim 13), and every draw of one removed object
    comes before those of the next: its own animation picks if it is a vehicle (S-0210 claim 4),
    then the timer draws of its holders. The draws for different objects never interleave.

    When the drop ends an override of the holder (S-0215 claim 10), the revert can make the same
    object the holder's target again, so a destroyed object that stays in play, or an object off the
    map, can still be held. The removal is expected to drop it again after the revert (medium), so
    that no holder holds a removed object. DECIDED (autonomy, 2026-10-30), for Q-2006 question 3:
    while a holder holds an object that a revert gave it again and that is off the map, or destroyed
    but still in play, our engine treats that target as no target in the holder's firing step: no
    firing check is asked about it and no shot is fired at it or at its place. At the start of the
    holder's next turn it drops such a target, unless the object's removal has dropped it first.
    This drop takes no timer draw and causes no revert (S-0215 claim 10: clearing the target by
    another rule does not revert), and the holder's behaviour then does what it does with no target.
    What the original does with such a target until its removal, and what its firing check answers
    for it, is not established (Open questions). A superweapon warp member that a holder holds
    through its departure is not such a target, since the departure makes no revert. What the
    firing check answers for it while it waits to land and while it is phasing in is open (S-0513
    claim 39). A member that a revert gives back to a holder while it is off the map is such a
    target. A crush gives both drops within the crush, and
    together they take at most one timer draw: after a first draw at most 8 ticks are left. For an
    aircraft holder, the adjustment of its behaviour that follows belongs to the aircraft specs.
    - Backing: `measurable` (scenario A18: part a for the crusher, part b for a removal by another
      unit, run b1 with the crusher's turn first and run b2 with the firer's turn first, part c for
      the killing hit of an object that stays in play, part d for the timer draw). The drop's
      moment, inside the action rather than at the start of the holder's own next turn, is
      `measurable` only by run b1's read of *A*'s target at the end of *C*'s turn, where the
      recorder allows that read: it tells a drop by the end of *C*'s turn from one at *A*'s next
      turn. It does not separate a drop inside the action from one later in *C*'s turn. Parts a to
      c otherwise give the same records and fire events under both moments. The
      off-map target drop: part e, for a target that boards a transport that is not open-topped.
      S-0215 scenario B7, case j, backs the revert and the restoration of the saved target, not a
      drop before it. `unmeasured`: the other off-map cases (an open-topped transport, a
      building), the vehicle bunker, the fall's removal and the removals taken to end like the
      others, since no part of A18 runs them. The order of the timer draws: part d for one
      holder, part f for several holders of one object (run f1) and for two objects in one cell
      (run f2), with no vehicle among the removed objects. The order of a removed vehicle's
      animation picks before its holders' timer draws keeps S-0210 claim 4's `unmeasured` backing:
      part f's removed objects are infantry units, which take no animation pick, and no part of
      S-0210 scenario K1 has a holder of the crushed vehicle. The projectile in flight: S-0306 claim
      17's own backing (its scenario P11, run a for an airborne target removed from the game, run c
      for a target on the ground destroyed by another weapon), and `unmeasured` for a crushed target
      and for a projectile whose object goes off the map: no part of A18 or of P11 fires at an
      object that is then crushed, boards a transport or enters a building. The
      movement-before-firing order is `unmeasured` for infantry
      and for movement other than a crush: A18 part a separates it only for a crushing vehicle.
      `unmeasured` for a building or aircraft holder (a building's pending delayed shot included),
      and for the second drop at the removal of an object the revert named again: no part of A18
      or of B7 has a building or aircraft holder, or removes the object that a revert named again.
      Our engine's handling of a restored object that stays off the map or in play after its
      killing hit is a design rule (DECIDED above), not a claim about the original.
      `unmeasured` for a vehicle or building destroyed by a hit (removed at once, or a building
      staying in play awaiting its removal, S-0312 claim 21) and for a sinking or crashing object
      at its killing hit: parts a and b crush their objects, and the only object killed by a hit,
      in part c, is an infantry unit in its death sequence.
      `unmeasured`: the two passages on a superweapon warp member, since no part of S-0513 judges
      a holder that holds the member at departure (its holders among those to which claim 29's
      rules apply drop it at that claim's check before then, S-0513 claim 39, and a later
      re-taking is recorded but not judged), fires at a member as it departs, records the holders
      of a member destroyed by its landing check, or puts an override in effect for a holder of
      the member.
    - Confidence: high (the drop, and the absence of a shot in the holder's turn after the drop, for
      infantry and vehicle holders of an object that is crushed, as in A18 parts a and b, or of an
      infantry unit killed in its death sequence, as in part c); medium (the drop's moment inside
      the action, until run b1's read is made; the timer draw; the order of the timer draws, by
      creation order and per object, part f; the order of a removed vehicle's animation picks
      before its holders' timer draws, `unmeasured` above; the off-map drop of a target that
      boards a transport, part e; the revert of S-0215 B7 case j; and
      each `unmeasured` case above: a vehicle or building destroyed by a hit, a building awaiting
      removal, a sinking or crashing object at its killing hit, the other off-map cases, the
      vehicle bunker, the fall's removal and the removals taken to end like the others, a
      building or aircraft holder and a building's pending delayed shot, movement before firing
      outside a crush, and the second drop after a revert); high for the projectile in flight
      towards an object removed while airborne or destroyed on the ground (S-0306 P11 runs a and
      c), medium for one whose object is crushed or goes off the map; the handling of a
      restored object that stays off
      the map or in play after its killing hit is a design rule; medium for the two passages on a
      superweapon warp member

28. **Cloaking and going underground drop a held target, with exceptions.** The drops at cloaking
    are S-0516 claim 14's: at the start of cloaking and again when the object becomes fully cloaked
    (a building's completed fade does not repeat the drop), every holder of the object as its target
    drops it, unless the holder's house senses the object's cell or the holder has the same owner as
    the object. When an object of an underground type goes underground, every infantry unit,
    vehicle, building and aircraft that holds it as its target drops it at that moment, inside that
    action, with the same two exceptions. These are the exceptions that S-0215 claim 10 gives for
    these events as revert causes. Each of these drops, at either cloak moment and on going
    underground, resets the holder's targeting timer as in claim 27. A holder that keeps the target
    under an exception takes no timer draw. If claim 29 covers it, it can still lose the target
    later at claim 29's check if the firing check then refuses it, for example by S-0516 claim 24's
    hidden-target refusal. When a drop ends an override, the revert of S-0215 claim 10 follows.
    Without an override the holder is left with no target, as in claim 27's consequences.
    - Backing: the cloak drops at both moments, with the building's single drop and the two
      exceptions: S-0516 claim 14's own backing (`measurable`, S-0516 scenario CS9: parts a and b
      for the house's sensing, part c for the same owner; the building's single drop rests on that
      claim's backing as it stands).
      The timer reset at each cloak moment and on going underground: `unmeasured`: CS9 records
      whether the target is kept, not the holder's targeting timer, and no scenario here makes a
      holder's target cloak or go underground. The underground drop and its exceptions:
      `unmeasured`: no scenario here or in S-0215 has a holder whose target goes underground. The
      revert: S-0215 claim 10's backing.
    - Confidence: medium for the cloak drops at both moments (S-0516 claim 14's level); medium for
      the timer reset at each cloak moment and on going underground; medium for the underground
      drop and its exceptions; the revert at S-0215 claim 10's level

### The periodic held-target check

29. **A held target that the firing check refuses is dropped at a periodic check.** In its own
    turn of every tick whose tick number is a multiple of 16 (a *check tick*), each holder that
    this claim covers asks the firing check about its target, without the range test, for the
    weapon it would use against it (S-0308, which owns the weapon choice, and S-0304 claim 6 for a
    zero versus value). If the answer is *illegal* or *impossible*, it drops the target. Any other
    answer leaves the target held. The claim covers every infantry unit, vehicle, aircraft and
    building that holds a target, whatever its owner, in every current behaviour but *capture*
    and *sabotage* (*Behaviours* below). The check reads only the held target, so it is the same
    whatever gave the target: an order, a search, retaliation (S-0215 claim 18) or a revert
    (S-0215 claim 10). In our engine that is every holder of every house: the player houses, AI
    houses included, and the neutral and special houses (Terms, *Houses in our engine*). A holder
    of a computer-controlled owner has one more drop, of a target allied with it, at this check's
    place in each of its turns, ahead of the check where one is made (claim 25, part k). Its rules:
    - **Phase.** Every object checks in the same ticks. A refusal that begins outside the holder's
      turn, and that holds at the holder's first checking turn (one that is not cut short, in a
      behaviour that checks) at the first check tick in which the holder's turn comes after the
      refusal begins, is acted on in that turn.
      When the holder takes one such turn in every tick, such a refusal that begins in tick *t*
      before the holder's turn is acted on 0 to 15 ticks after *t*, and one that begins after the
      holder's turn in tick *t*, as with an order applied after the turn pass (S-0203 claim 7), 1
      to 16 ticks after *t*. A refusal that the holder's own behaviour processing, a search of its
      own, its own movement or its own firing step begins in its turn of tick *t* comes after that
      turn's check (*Place in the turn* below), so it too is acted on 1 to 16 ticks after *t*. When
      *t* is a check tick, that turn's check does not act on it, and, unless the holder takes a
      second turn in *t*, the next check tick, 16 ticks later, is the first that can. A holder
      that takes a second turn in a tick (S-0203 claim 11) makes the check in that turn too when
      the tick is a check tick, under the same rules: the check belongs to each of the holder's
      turns, not to the tick. So a refusal that begins after its first turn of a check tick *t*,
      its own steps in that turn included, and still holds at its second turn there, is acted on
      in that second turn, 0 ticks after *t*, when that turn checks. With
      `quirks.update_order_removal_skip` off no holder takes a second turn (S-0203 claim 11), and
      the deadlines above hold without this case. A projectile shot's effect on its target (a heal
      or damage) lands in a turn of the projectile, which comes after the holder's turn: in the
      firing tick when the projectile's first turn there reaches the target, and otherwise in a
      later tick, as when a removal costs the projectile its first turn or its flight takes longer
      (S-0203 claims 4 and 5). A shot of a weapon with the railgun flag computes its hits at the
      shot (S-0306 claim 27), so their effect lands inside the holder's firing step, which comes
      after the check (*Place in the turn* below). Either way a refusal that the holder's own shot
      begins comes after that turn's check. A holder that misses its turn in a check
      tick (S-0203 claim 10), or whose turn there is cut short or in a behaviour that makes no
      check, acts on the refusal, if it still holds, at its next checking turn in a later check
      tick instead.
    - **Behaviours.** A holder whose current behaviour is *capture* or *sabotage* makes no check.
      Every other current behaviour makes it: *attack* and *guard*, and also *move*, *hunt*, *area
      guard*, *sticky*, *patrol*, *attack move*, *enter*, *repair*, *harvest*, *return*, *unload*,
      *rescue*, *missile* and the rest, including those in which no weapon is aimed at the held
      target. The question is the same in every behaviour: the firing check for the weapon that
      the holder would choose against the target. Its answer depends on the behaviour only where
      the weapon choice or a reason of S-0301 claim 9 itself does, such as S-0308's area-fire row
      for *unload* (table P, row 9). Aircraft follow the same rule in their own behaviours
      (S-0501), in each turn that is not cut short, after their own drops at the start of a turn
      (S-0501 claim 32, S-0516 claim 25). The behaviour that counts is the one current when the
      check is made (*Place in the turn* below).
    - **Answers.** The answer is S-0301 claim 9's, with its ranks, except that its range row (rank
      10) is not asked. Whether the target is in range plays no part. A reason ranked above the
      refusal gives its own answer, and the target is kept unless that answer is *illegal* or
      *impossible*. So while the holder's rearm countdown runs (rank 5), a target for which only a
      rank-9 reason holds, such as a zero versus value (S-0304 claim 1), is kept, and it is
      dropped at the first check tick after the countdown has run out. A vehicle launching spawned
      aircraft (rank 2, *busy*) keeps its target. A rank-4 refusal, such as an airborne target
      that the projectile cannot hit (S-0306 claim 23), outranks the countdown, so that target is
      dropped at the first check tick whatever the countdown shows. A holder whose chosen weapon
      is not refused keeps the target, even if its other weapon would be.
    - **Place in the turn.** For every kind of holder, the check in its turn comes before its
      behaviour processing, before the searches of that processing and its per-tick search (S-0215
      claim 33), before its movement and before its firing step. For an infantry unit, a vehicle or
      a building it comes after the turn's first per-tick start (S-0215 claim 11), so a queued
      behaviour that this start makes current is the one that decides whether the check is made: a
      queued *capture* or *sabotage* made current there makes none, and a queued *attack* or
      *guard* made current there in place of either makes it. An aircraft's per-tick start follows
      its processing and movement (S-0215 claim 11), so for an aircraft the behaviour current at
      the start of its turn decides. A behaviour that becomes current later in the turn, through a
      processing, at an infantry unit's or vehicle's late start, at a building's second start or
      at any of an aircraft's start points (S-0501 claim 31), decides from the holder's next
      checking turn in a check tick on: its second turn in the same tick when it takes one
      (*Phase* above), and otherwise one in a later check tick. So when the holder's own
      processing, search, movement or firing step, in its turn of a check tick, starts a refusal
      of its held target, that turn's check does not drop it (a second turn's can). For a falling
      infantry or vehicle holder the check comes after its fall step and any landing hit, which
      precede its behaviour processing and its firing check in that turn
      (S-0203 claim 4, S-0301 claim 9, S-0511 claim 10). So in a check tick a vehicle holder still
      falling after that turn's fall step answers *illegal* (S-0301 claim 9, rank 4), and the check
      drops its target, unless an answer that outranks it and keeps the target holds: the *busy*
      of a vehicle launching spawned aircraft (rank 2), the *rearming* of a materialising
      vehicle on which no temporal beam is locked, which comes before every other rank-4 answer
      (*Turns cut short* below), or, when its target is its own gripped unit, the *rearming*
      against that unit, which outranks the falling *illegal* (S-0301 claim 9, *Grip refusals
      within rank 4*). An infantry holder still falling then answers *illegal* or
      *impossible*: which one is S-0310's open question on infantry in a dying or falling
      sequence. The check drops its target on either answer, except for a materialising infantry
      holder on which no temporal beam is locked and for an infantry holder whose target is its
      own gripped unit. Such a holder keeps its target if the falling answer comes after its
      *rearming* (the materialising one, or the one against its gripped unit), and drops it if
      the falling answer is asked first. Which holds depends on the same open question of S-0310
      (the Changes for S-0301 claim 9 below). A holder that lands and survives
      loses the falling refusal in that same turn (S-0301 claim 9), so no check from that turn on,
      including that turn's own in a check tick, drops its target for falling. A building makes the
      check before its pending delayed shot (S-0301 claim 14) counts down in its turn, so in the
      tick that ends the delay the pending shot still answers *rearming* (S-0301 claim 9) and the
      target is kept, as S-0301 scenario F21 requires. The drop leaves the
      holder with no target until something gives it one: a search may pick the same object again
      where table C does not reject it (it does not reject on *impossible*), and the firing check
      is then asked about it as about any target.
    - **The drop.** It sets the holder's target to none, with claim 24's consequences of a setting
      that leaves no target (a spawner's conditional pending-target clear, the cancellation of an
      airstrike). It takes no targeting-timer draw: claim 27's reset belongs only to those drops
      of claims 27 and 28 that give it. It causes no revert (S-0215 claim 10: clearing the target by
      another rule does not revert). It changes no behaviour by itself: the behaviour's next
      processing does what it does with no target (S-0215), so an object in *attack* stops attacking
      at that processing.
    - **Turns cut short.** A turn that ends early makes no check: that of a vehicle or infantry
      unit travelling through a tunnel, the one turn in which a vehicle completes its death
      frames, and that of a building that is being erased or materialising. A vehicle's earlier
      turns of its death frames are not cut short. It makes the check in them, and S-0301 claim 9
      answers *impossible* (rank 1) for a vehicle playing its death frames, so it drops a held
      target at its first checking turn in a check tick while they play. An infantry unit,
      vehicle or landed aircraft that is being erased drops its target instead in each of its
      turns (S-0215 claim 2). So does a materialising infantry unit, vehicle or aircraft that is
      being erased with a temporal beam locked on it (glossary): it drops its target in each of its
      turns, whatever the tick number, and that turn ends there with no check. So does a
      materialising vehicle or aircraft whose temporal beam stays attached while it is not being
      erased, as a superweapon warp member's is after its departure (S-0513 claim 39a). A
      materialising infantry unit in that state makes the check, as one on which no temporal beam
      is locked does (S-0513 `quirks.phase_in_beam_spares_infantry`, default on; off: it drops its
      target as the vehicle does). A
      materialising infantry unit, vehicle or aircraft on which no temporal beam is locked makes
      the check, and S-0301 claim 9 gives a materialising firer *rearming* within rank 4. That
      *rearming* comes before every other rank-4 reason except the *illegal* of a firer bound to
      a slave master, and after the answers that the firer's kind asks first: for a vehicle its
      answers of ranks 1 to 3 and its tube-transit *impossible*, and for an infantry unit those
      that the Changes for S-0301 claim 9 below list. For a vehicle, its *rearming* after ranks 1
      to 3 rests on S-0301 claim 9's order of ranks 1 to 4 among themselves. Where the answer of
      ranks 1 to 3 keeps the target too (the rank-2 *busy* of a vehicle launching spawned
      aircraft), the check keeps it whichever comes first. Where it refuses the target (the
      rank-1 *impossible* of a vehicle playing its death frames, the rank-3 melee-link
      *illegal*), the drop at the check rests on that order. So such a holder keeps a target that is
      hidden from its house
      (S-0516 claim 24), that its projectile cannot hit (S-0306 claim 23) or that is protected
      against its parasite warhead (S-0513 claim 27), and drops it at its first checking turn
      in a check tick after its phase-in has ended, if the refusal still holds then. The
      exception is an answer the holder's kind is asked first. A materialising infantry holder
      whose default weapon is a healing weapon and whose target is at full health gets the
      infantry healer's *illegal* (S-0310 claim 11) in its turns while it phases in, so the
      keep and that drop deadline do not apply to it. It drops the target at the first of its
      own reply to *illegal* in its firing step (S-0310 claim 11) and its first checking turn in
      a check tick, as any infantry healer does. An aircraft
      holder, materialising or not, drops two kinds of target anyway at the start of each of its
      turns, before this check: a target in the air (S-0501 claim 32), and a target that is fully
      cloaked, not allied with it and not sensed by its house (S-0516 claim 25, S-0501 claim 32).
      That drop takes no other target hidden from its house: one allied with it, or a building
      hidden at fade levels 11 to 15 (S-0516, Terms). Such a target is left to this check, as for
      any other holder.
    - **Warp members.** Whether a member of a superweapon warp (S-0513 claim 39) that holds a
      target makes this check in its turns of the warp in the original, from its first turn after
      the launch through its arrival effects, is not settled here: S-0513 claims 48 and 52 give
      what is established about the ordinary work of those turns, and leave it unsettled. Our
      engine does not take these checks in the warp: S-0513 claim 51 makes the warp update the
      whole member turn from its first pending update through its arrival effects, and this check
      is among the ordinary work it suppresses there. In the arrival hold that contract puts the
      check after the hold-end test, under this claim's rules. Both are engine design rules of
      S-0513 claim 51, not reproduction claims, and have no quirk switch. The same holds for
      claim 25 part k's drop of an allied target, which comes at this check's place: in the
      original it is made in a member's warp turns exactly when this check's place is reached,
      which is unsettled with the check. In our engine claim 51's warp update suppresses it
      with the check, since it is not among that claim's intrinsic exceptions (high as design,
      as claim 51 gives it).

    Consequences for the other drops of this spec: claim 19's drop of an opportunity target can
    come earlier than the next search, at a check tick, and claim 4's keep of an *impossible*
    target by a searcher that controls spawned aircraft ends at a check tick, as claim 4 gives.
    This is not the only drop made at the
    start of a holder's turn: claim 25 part k gives one for holders of computer-controlled owners,
    and others are listed in Open questions (*Other drops at the start of a turn*). Nor is it
    the only drop of a target that the firing check refuses: a healer's firing step drops its
    target where its own reply to *illegal* drops it, in each of its turns (S-0310 claim 11's reply
    table, and the Changes entry for it below for infantry healers), so for those targets that drop
    can come first (as that claim rates it for vehicle healers, and medium, `unmeasured`, for
    infantry healers, as that entry gives). That table keeps a target of the kind the healer heals
    that is below full health and still gives *illegal* (a versus value of exactly 0 among them),
    and this check then drops it, at the same levels.
    - Backing: `measurable` (scenario A6, part p: runs p1 and p2 for the cadence, the drop of an
      ordered target on *illegal*, for vehicle and infantry holders, with a drop off a multiple of
      16 before the expected tick falsifying the cadence; run p1, in its runs in which the holder's
      targeting timer has more than 10 ticks left at the drop tick, for the absence of a timer draw,
      and run p2 for the rank-5 keep, for a vehicle holder; run p6 for the shared phase, against a
      phase counted from each holder's creation, for vehicle and infantry holders; run p4 for the
      rank-4 refusal against a running countdown; run p5 for the drop on *impossible*, through a
      hidden target; part m for the drop of an opportunity target by a vehicle holder in *guard* no
      later than the first check tick after the refusal begins; run p7 for a refusal that the
      holder's own shot begins in a check tick, which that tick's check does not act on, against a
      check that acts on it in that tick; run p8 for the check by a vehicle holder in *area guard*
      and in *move*, against no check in those behaviours; run p9a for the check by a vehicle
      holder of a computer-controlled owner in *area guard*; run
      p10 for a materialising vehicle holder, the keep of a target in the air until its phase-in
      has ended, run p10a, and the drop in each turn with a temporal beam locked, run p10b).
      `unmeasured`: the check in a holder's second turn of a check tick, since a recorder cannot
      tell S-0203 claim 11's second turn from a first (that claim's Backing), and no part here
      gives a holder a second turn. Also `unmeasured`: the check's place against the holder's
      behaviour processing, its searches, its movement and its firing step, since no part here
      starts a refusal in those steps of the holder's own turn. A projectile shot's effect lands
      in the projectile's turn, after the holder's (S-0203 claims 4 and 5), and run p7's healing
      weapon is a projectile weapon, so run p7's records are the same for a check before or after
      the firing step. Also `unmeasured`, at medium: that a railgun shot's hits, computed at the
      shot (S-0306 claim 27), land after the check in the holder's turn, which follows from that
      claim and the check's place, since no part here gives a holder a railgun weapon. Also
      `unmeasured`: the check's place after a falling
      holder's fall step and landing hit, its drop of a falling infantry holder's target on
      either answer and the keep of a falling holder that is also materialising or whose target
      is its own gripped unit, since no part
      here gives a falling holder a held target; which behaviour
      decides when a queued behaviour becomes current
      in the holder's turn of a check tick, since no part here gives a holder whose target the
      check refuses a queued behaviour that starts in such a turn; the drop of a
      searched target by an infantry holder, since part m's firer is a vehicle; for a holder in
      *guard*, the absence of a timer draw, the shared phase and the cadence, since runs p1, p2 and
      p6 give their holders *attack*, part m's firer is a vehicle, and part m lets a drop in a tick
      whose tick number is not a multiple of 16, at no search of the firer, decide nothing; the
      absence of the range test, since it shows only for a refusal ranked
      below range (rank 12), which no part here gives; the absence of a revert, since no part here
      puts an override in effect for a holder whose target the check refuses; the behaviour
      exception, since no part here gives a holder in *capture* or *sabotage* a target that the
      check refuses; the rank-5 keep for an infantry holder, since run p2's holder is a vehicle; the
      chosen weapon, since in run p3 the second weapon's rearm countdown can answer *rearming*
      (rank 5) at every check tick and so hide a check that asks about the first weapon; a
      building's check before its pending delayed shot counts down, since no part here or in
      S-0301 scenario F21 ends a delay in a check tick by design; the
      later drop of a holder that misses its turn in a check tick, since no part here makes a holder
      miss a turn; the turns cut short, the check and drop in a vehicle's earlier death-frame turns,
      the drop by a holder that is being erased other than a materialising one, since no part here
      gives a target to a holder in one of those states; that a materialising holder with no
      beam locked keeps its target because its check answers *rearming*, rather than because its
      turn makes no check while it phases in, since run p10a's records are the same for both and
      no part here gives such a holder an answer that outranks *rearming*; the materialising
      cases for infantry and aircraft holders and for targets other than one in the air, since
      run p10's holder is a vehicle and its target an aircraft in the air; the other behaviours
      than *attack*, *guard*, *area guard*
      and *move*, since no part here gives a holder in them a held target that the check
      refuses; a target that a revert gave, since no part here puts an override in effect for such a
      holder; a target that retaliation gave, since its attacker must fire at the holder and then
      be hidden again while the holder still holds it, but an armed attacker within its weapon's
      reach of the holder keeps the holder as its target or takes it again by its own search,
      which bars it from cloaking (S-0516 claim 4), and an attacker sent away from that reach is
      followed by the holder, so no part here gives the holder a retaliation target that the
      check refuses; the end of a spawner's keep at a check
      tick, for the reason given in claim 4's Backing; aircraft holders, since no part here gives an
      aircraft holder a refused target; and building holders, since a building's own attack
      processing (S-0215 claim 32) drops such a target at its next processing, so its records cannot
      tell the periodic drop apart. The original's checks in a warp member's turns are `unmeasured`:
      no part here gives a warp member a held target that the check refuses at a check tick, and
      S-0513 claims 48 and 52 leave them unsettled. Our engine's suppression of them is `measurable`
      in our engine (S-0513 scenario SW23 part f). The *Turns cut short* rule for a materialising
      vehicle, aircraft or infantry unit whose temporal beam stays attached while it is not being
      erased is `unmeasured` (S-0513 claim 39a: no part there holds an infantry member, and SW12
      part l's tank has a hold of one update).
    - Confidence: high (the cadence of the drops at check ticks, the shared phase, the drop on
      *illegal* of a target given by an order and the absence of a timer draw, for human-controlled
      vehicle and infantry holders in *attack* that take a checking turn in every tick; and the
      drop on *illegal* of a target given by a search no later than the first check tick after the
      refusal begins, for such vehicle holders in *guard*), medium (for holders in *guard*, the
      shared phase, the absence of a timer draw and the cadence; the drop on *illegal* of a target
      given by a search, for infantry holders; that a refusal an infantry holder's own shot begins
      in a check tick is not acted on by that tick's check, run p7, which leaves the later drop to
      the healer's own reply (S-0310 claim 11) or the next check, at the level of S-0310 claim 11,
      its premise; the absence of the
      range test; the absence of a revert; the drop on *impossible*, run p5; the end of a spawner's
      keep at a check tick; the examples of the ranks above, which are S-0301 claim 9's and rated
      medium here, with the rank-5 keep for a vehicle holder; the chosen weapon; a building's
      check before its pending delayed shot counts down; the
      behaviour exception; the later drop after a missed or exempt turn; the check in a second
      turn of a check tick, at S-0203 claim 11's level, its premise; the check's place in the
      turn, with its place after a falling holder's fall step, and the behaviour that decides when
      a queued behaviour becomes current in the turn; the keep by a falling vehicle holder that is
      also materialising with no beam locked, as S-0301 claim 9's precedence within rank 4,
      amended below, rates it; the keep by a falling vehicle holder whose target is its own
      gripped unit, as S-0301 claim 9's grip rule within rank 4, amended below, rates it; the
      keep by a materialising vehicle holder that also launches
      spawned aircraft, which holds whichever of its *rearming* and the rank-2 *busy* comes
      first; the turns cut short, the
      check and drop in a vehicle's earlier death-frame turns, the drop by a holder that is being
      erased; the drop or check of a materialising vehicle, aircraft or infantry unit whose temporal
      beam stays attached while it is not being erased; the materialising cases for vehicles,
      infantry and aircraft, runs p10a and p10b for a
      vehicle, with the materialising infantry healer's *illegal* against a full-health target
      and its drop at the check, medium as S-0310 claim 11 rates the infantry rule and its rank,
      its premise; aircraft and
      building holders; the check in every behaviour but *capture* and *sabotage*, run p8 for
      *area guard* and *move*; the check by holders of computer-controlled owners, run p9a; the
      same check for targets that retaliation or a revert gave), low (the rank-5 keep
      for an infantry holder; the materialising infantry healer's drop through its firing step's
      reply to *illegal*, as S-0310 claim 11 rates that the infantry answer shows through its
      reply; the keep-acting *illegal* that comes before a materialising
      infantry holder's *rearming*, as the Changes for S-0301 claim 9 below rate it; the keep or
      drop by a falling infantry holder that is also materialising or whose target is its own
      gripped unit, which rests on S-0310's open
      question; the keep by a falling vehicle holder through the rank-2 *busy*, and the drop by a
      materialising vehicle holder on a refusing answer of rank 1 or 3, which rest on S-0301
      claim 9's order of ranks 1 to 4 among themselves, low there). For the warp
      members: low for the original's checks, which S-0513 claims 48 and 52 leave unsettled, and
      high as design for our engine's suppression of them

## Formulas

Notation: trunc is truncation toward zero (S-0205, claims 2 and 7). ρ₃(*P*, *Q*) and ρ₂(*P*, *Q*)
are ⌊*r*(*n*)⌋ of the table square root (S-0204, claim 5, F1) on the spatial and planar squared
difference of two points, with no building reduction (S-0204, claim 6, does not apply here); `quirks.table_root_distance`
(S-0204) applies. Objects are measured at their reference points
(S-0204, claim 4).

**F1. Radius and rings.** Leptons and cells, integers.

> range search: *R* = *g* when *g* ≠ 0 and the searcher is not an engineer; *R* = 0 otherwise
>
> area search: *G* = *g* when *g* ≠ 0, and *G* = max(*w*₀, *w*₁) when *g* = 0; *R* = min(max(2*G*, 0), 4096); in
> *patrol*: *R* = min(max(2*G*, 1792), 4096)
>
> healer for acquisition (S-0308, claim 15) whose current behaviour is *guard*, in any search that has a radius:
> *R* = 512, instead of the *R* of the two formulas above
>
> *n* = trunc(*R* / 256) when *R* ≠ 0; *n* = trunc(*w* / 256) + 1 + trunc(*b* / 256) when *R* = 0
>
> garrisoned building: *n* = trunc(min(*W*, *H*) / 2) + 1 + *o* (replaces the above)

*g* is the type's guard range, *w*ᵢ the firer's weapon range of S-0307 claim 13 for slot *i*
(for an open-topped transport, the shortest of its own range and the range fields of its passengers'
current weapons, skipping passengers without one), *w* the range of
the weapon of its weapon number for a switching type, else max(*w*₀, *w*₁) (a type that is
underwater, organic and self-healing uses *g* instead), *b* the type's air range bonus, *W* × *H* the
building's foundation and *o* the game-wide occupant range bonus in cells. The max(*w*₀, *w*₁)
inside *G* is always the plain larger of the two slot ranges: the switching-type and underwater
exceptions apply only to *w*. For a healer for acquisition, *R* = 512 gives *n* = 2.

**F2. Order of the cells of ring *r*** (centre cell (*x*, *y*), integer cell coordinates). Ring 0
is the centre cell. For *r* ≥ 1 the 8*r* cells of the ring come in this sequence:

> (*x* − *r*, *y* − *r*), (*x* − *r*, *y* + *r*), (*x* − *r* + 1, *y* − *r*), (*x* − *r* + 1, *y* + *r*), …,
> (*x* + *r*, *y* − *r*), (*x* + *r*, *y* + *r*),
>
> then (*x* − *r*, *y* − *r* + 1), (*x* + *r*, *y* − *r* + 1), …, (*x* − *r*, *y* + *r* − 1), (*x* + *r*, *y* + *r* − 1)

The cells of rows *y* − *r* and *y* + *r* come first, in pairs by column from *x* − *r* to *x* + *r*,
with the row-(*y* − *r*) cell of each pair first. The remaining cells of columns *x* − *r* and
*x* + *r* follow, in pairs by row from *y* − *r* + 1 to *y* + *r* − 1, with the column-(*x* − *r*)
cell of each pair first. Cells that are not legal are left out.

**F3. Distance test of a candidate.** With *A* the searcher and *T* the object:

> δ = ρ₂(*A*, *T*) if *T* is airborne, else ρ₃(*A*, *T*)  (leptons, integer)
>
> *R* > 0: *T* passes when δ ≤ *R*. *R* = 0: an armed *A* needs *T* within the reach of the weapon
> it would use (S-0307); an unarmed *A* needs δ ≤ *g*. Map-wide search: no test.

δ is always measured from the searcher *A*, also in an area search, whose rings are centred on the
guard spot: a candidate must lie within *R* of the searcher itself, not of the guard spot.

**F4. Threat score.** All in double precision, each operation rounded toward zero to 53 bits in the
original (S-0205, claims 4 and 5; `quirks.chop_rounding`). With *v*(*X* → *Y*) the versus value of the
weapon *X* would use against *Y* (0 when *X* has no such weapon or it has no warhead). Every
candidate is an infantry unit, vehicle, aircraft or building:

> *score* = conv( ( *c*<sub>d</sub> × *e* + ( ( *P* + *c*<sub>me</sub> × *v*(*A* → *T*) ) + *c*<sub>sg</sub> × *h*<sub>T</sub> ) ) + 100000 )
>
> *P* = ( *t* + *c*<sub>st</sub> × *s*<sub>T</sub> ) + *B* when *A*'s owner has an enemy house and that house owns *T*, and
> *P* = *t* + *c*<sub>st</sub> × *s*<sub>T</sub> otherwise (no *B* term)
>
> *t* = −( *c*<sub>te</sub> × *v*(*T* → *A*) ) if *T*'s target is *A*, *t* = *c*<sub>te</sub> × *v*(*T* → *A*) otherwise
>
> *e* = max(*d* − *k*, 0)

Every + and × is one double-precision operation rounded toward zero, so the parentheses fix the
result. Swapping the two operands of one sum or product does not change it.

*c*<sub>me</sub>, *c*<sub>te</sub>, *c*<sub>st</sub>, *c*<sub>sg</sub>, *c*<sub>d</sub> are *A*'s type's
own-effectiveness, target-effectiveness, special-threat, strength and distance coefficients,
*s*<sub>T</sub> is *T*'s type's special threat value; *B* the game-wide enemy-house bonus (all
seven are held in double precision as imported, Import mapping),
*h*<sub>T</sub> = health / strength field of *T* (S-0214's health ratio, a double quotient),
*k* = trunc(range field of the weapon *A* would use against *T*, in leptons (S-0307, claim 1, with
no bonus or replacement) / 256), or trunc(*g* / 256) when there is no such weapon (cells).
*d* = trunc(ρ₃(*A*, *T*) / 256) cells in range and area searches and in the aircraft pass of
the map-wide search, but *d* = ρ₃(*A*, *T*) in *leptons* in the ground-layer pass of the
map-wide search, including a landed aircraft considered in that pass (`quirks.hunt_distance_in_leptons`). *e* is an integer converted exactly to a
double. conv is the common conversion of S-0205, claim 7 (truncation toward zero, wrap modulo 2³²,
0 for magnitudes of 2⁶³ or more).

**F5. Weak-target preference** (mode *normal*), on the integer score:

> *score*′ = trunc(*score* / 2) if *x* ≤ 0, and *score* × 2 (32-bit, wraps) if 0 < *x* ≤ trunc(*S*<sub>T</sub> / 2),
> and *score* otherwise

*x* is *T*'s expected remaining health, *S*<sub>T</sub> its type's strength field.

**F6. Suppression.** *N* = trunc(*L* / 256) with *L* the game-wide fire-suppression distance in
leptons; *m* = the number of cells in rings 1 … *N* − 1 around *T*'s cell, inside the map, that hold a
building allied with *A*. *score*′ = conv(*score* × 0.5<sup>*m*</sup>), the product in double
precision; *score* here is the integer after F5. Since *score* is a 32-bit integer and
0.5<sup>*m*</sup> a power of two, this equals trunc(*score* / 2<sup>*m*</sup>), and *score* itself when
*m* = 0. When *N* ≤ 1 there are no rings to count, so *m* = 0.

**F7. Expected remaining health**, integers, exact. With *x* its value before the object's turn in
tick *t*, *h* the object's health and *x*′ its value after that turn:

> *x*′ = *h* if *x* ≥ *h*; *x*′ = max(*x*, −30) + 1 if *x* < *h* and ⌊*t* / 4⌋ is odd; *x*′ = *x* if *x* < *h*
> and ⌊*t* / 4⌋ is even

**F8. Anger increment.** Anger values are 32-bit integers (an addition wraps). One point of anger
is one credit of the victim type's cost, in proportion to the share of its strength lost.

> Δ = conv( ( *d* / *S* ) × *C* )

*d* (damage dealt, health points, negative for a healing hit), *S* (the victim type's strength
field) and *C* (the victim type's *type's cost* of S-0314 claim 5, credits, without house or
country factors) are integers, each converted exactly to a double. The quotient and the product
are each one double-precision operation rounded toward zero (S-0205, claims 4 and 5, and
`quirks.chop_rounding`), and conv is the common conversion of S-0205, claim 7 (truncation toward
zero). The anger value grows by Δ, which may be negative.

**F9. Final score** (claim 13), integers. With *s* the integer of F4:

> *score*<sub>final</sub> = Z(F6(F5(*s*)))
>
> Z(*z*) = *z* if *z* ≥ 1; Z(*z*) = 1 if *z* < 0; *z* = 0 rejects the candidate
> (`quirks.zero_score_rejected` on) or gives Z(0) = 1 (off)

F5(*s*) is *score*′ of F5 with *score* = *s* (the identity when the mode is not *normal*), and
F6(*u*) is *score*′ of F6 with *score* = *u* (the identity when the current weapon has no suppress
flag).

**F10. Wall score** (claim 25, part i). Leptons, integers.

> *wall score* = *w*<sub>c</sub> − ρ₃(*A*, *c*)

*w*<sub>c</sub> is the range of the searcher's current weapon and *c* the centre of the wall cell
taken at height 0, so the height term of the squared difference is the searcher's own height. ρ₃ is
truncated toward zero (Notation above), so the difference is an integer.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Score base | 100000 | score units | F4 |
| Area radius cap | 4096 | leptons | F1; same cap as S-0215 F5 |
| Patrol radius floor | 1792 | leptons | F1 |
| Healer guard radius | 512 | leptons | F1, claim 6; for a healer for acquisition |
| Healing threshold | 1 | health ratio | Claim 6; a fixed engine value, not read from data |
| Cells per ring step | 256 | leptons | F1 |
| Early-stop rings | trunc(*n* / 4), trunc(*n* / 2) | rings | Claim 9 |
| Lowest searchable height | −20 | leptons above the surface | Table C. A height of exactly −20 passes: only heights below it are rejected. S-0214 (claim 6(b), F3) uses the same value but needs a height strictly above it, so a target at exactly −20 can be found by a search but not clicked for attack |
| Suppression factor | 0.5 | per allied building cell | F6 |
| Fire-suppression distance absent-key default | 256 | leptons | F6; gives *N* = 1, no suppression |
| Expected health floor | −30 | health points | F7 |
| Expected health recovery | 1 per tick on 4 of every 8 ticks | health points | F7 |
| Vehicle-thief take-over distance | 3840 | leptons | Claim 5; the destination vehicle must be closer than this |
| Engineer chosen-building distance | 3840 | leptons | Claim 25, part c; the chosen building must be closer than this |
| Disguise draw range | 0 … 99 | random draw value (no unit) | Claim 25, part g; one ranged draw per test |
| Passive-acquire and retaliate flags absent-key default | yes | boolean (no unit) | Import mapping |
| Initial type coefficient | 0 | coefficient (no unit) | claim 26; absent-key replacement is separate |
| Held-target check period | 16 | ticks | Claim 29; checks fall in the ticks whose tick number is a multiple of it |
| Coefficient warning threshold | 10⁶ | coefficient magnitude (no unit) | Edge cases, *Out-of-range scores*; one of our constants, not the original's: the importer warns above it |

## State and ordering

**Table C: candidate tests.** *T* is a candidate for *A* only if no row rejects it.

| Row | *T* is rejected when | Confidence | Scenario |
|---|---|---|---|
| exists | *T* does not exist, has been removed from play, or has 0 health | high | A14, part o (0 health); the other cases are `unmeasured` (claim 12) |
| self | *T* is *A* | high | none (claim 12) |
| legal target | *T*'s type is not a legal target (Import mapping) | high | A14, part e |
| class | *T*'s kind is not among the search classes (claim 5). A search for vehicles also accepts aircraft and any object that counts as a vehicle (S-0214); the scan of airborne objects (claim 9) also accepts infantry and vehicles | high (medium for the two extensions) | A2 (part h for the first extension); A14, part m, for the second |
| firing check | the firing check without range for the weapon *A* would use answers *illegal* | high | none (claim 12) |
| versus | *A* would use a weapon with a warhead against *T*, and that warhead's versus value against *T* is at most τ<sub>a</sub> (S-0304, claim 4). A searcher with no such weapon is not rejected by this row | high | A3 |
| land targeting | *A*'s land-targeting mode (S-0308, the mode that makes table P give slot 1 against land targets) is 2 and *T* is floor-bound (S-0307, Terms: on the map with a height above the surface below 208 leptons) and stands on a cell whose land type is not water | high | A14, part a |
| cloak | *T* is fully cloaked, not on a sensed cell of *A*'s owner and not owned by *A*'s owner | medium | A14, part g |
| playfield | *T* is not on the playfield | medium | A14, part h |
| no-threat | *T*'s current behaviour has the no-threat property (S-0215, claim 16) | high | A14, part f |
| underground | *T*'s height above the surface is below −20 leptons | high | A14, part b |
| zone | *T*'s cell is outside the connected region of *A*'s movement zone that holds *A*'s own cell, in the searches that claims 7 and 11 restrict (compared as claim 7 gives: two cells of no reach region of that zone count as the same region). No rejection by this row when *A*'s own deck read finds no bridge link, as claim 7 specifies | medium (low for the no-link deck exception) | A11, parts b, d and j; the no-link deck exception is `unmeasured` (claim 7) |
| friendly | *T* is allied with *A*, and *A* is neither a healer for acquisition nor an engineer; or *T* is allied with *A*, *A* is a healer for acquisition or an engineer, and *T*'s health ratio is 1. The row does not apply when *A* has a friendly-row exception (Terms: *A*'s type attacks friendlies, *A* is berserk, *A* has the permanent-berserk mark, or *A* is a passenger of a mind-controlled open-topped transport) | high (low for the passenger exception) | A12 (infantry and vehicle healers; a building healer is `unmeasured`, claim 6, backing); A3, part c, and A8, part f (attacks friendlies and berserk); A14, part l (the passenger exception); the permanent-berserk exception is `unmeasured`, since nothing in our engine sets the mark (Edge cases); the engineer exemption by A15, part e, run 2 (an engineer of the neutral house, to which claim 25, part d, offers allied objects) |
| healer | *A* is an infantry unit or vehicle that is a healer for acquisition (so its search has the *friendly* modifier, claim 6) and *T* is not allied with *A*. The row applies whatever friendly-row exception *A* has | high | A12, parts d and e |
| healer aircraft | *A* is a vehicle that is a healer for acquisition, *T* is allied with *A*, and *T* is an aircraft whose height above the surface is above 0 leptons or that stands on a cell holding a building. The row does not apply when *A* has a friendly-row exception | high (medium for the waiver) | A12, part c; the waiver is `unmeasured` (claim 6, backing) |
| harvester immunity | the map sets harvester immunity and *T*'s type is one of the game-wide harvester types | medium | A14, part i |
| distance | F3 fails | high | A1 |
| invisible building | *T* is a building whose type is invisible in game | high | A14, part c |
| passive house | in a skirmish: *T*'s owner is a multiplayer-passive house and *A*'s owner is human-controlled (for an *A* of the neutral or special house, the form of claim 25, part e) | medium | A13; claim 25's form is `unmeasured` (claim 25, backing) |
| insignificant | *T*'s type is insignificant, *T* is not mind-controlled, and *T* is not a building of a non-passive owner, and *A*'s owner is human-controlled (for an *A* of the neutral or special house, the form of claim 25, part e) | medium | A13; claim 25's form is `unmeasured` (claim 25, backing) |
| train | *T* is a train and *A* is a vehicle thief | medium | A14, part j |
| disguise | *T* appears disguised to *A*'s owner and *A*'s type cannot detect disguises (for an *A* of the neutral or special house, unless the draw of claim 25, part g, lets *T* through) | high (medium for claim 25's draw) | A13; A15, part c, for claim 25's draw |
| capture | *A* is a money thief (claim 5), or an engineer of the neutral or special house (claim 25, part c), and *T* is not a building whose type is capturable | low | A2, part c; A15, part e, for the engineer |
| harmless building | *A*'s owner is human-controlled, *T* is a building that does not count as a vehicle, and *T* has no current weapon (as in claim 14) or a threat value of 0; an engineer *A* is not rejected by this row. The row does not apply to an *A* of the neutral or special house (claim 25, part f) | high | A14, part d; A2, part g, for the engineer; A15, part a, for the neutral house |
| engineer | *A* is an engineer and *T* is not a building; or *A* is an engineer, *T* is allied with *A*, and *T*'s health ratio is above the game-wide red condition ratio (the health-bar threshold, Import mapping) or *T*'s type does not accept repair by an engineer of *A*'s owner (the structures specs own that test). Claims 7 and 10 offer an allied object only to a healer for acquisition, to a searcher with a friendly-row exception (Terms) and to an engineer of the neutral or special house (claim 25, part d), so the second clause decides only for an engineer with such an exception, for example one whose type attacks friendlies, and for an engineer of those two houses | low | A2, part g (first clause); A2, part i (second clause, with an engineer type that attacks friendlies); A15, part e, run 2 (an engineer of the neutral house) |
| storage | *A* is a money thief (claim 5) and *T*'s type has no storage capacity | low | A2, part c |
| bridge layers | *A* and *T* both stand on bridge cells and one is on the deck while the other is not | medium | A14, part k |
| controller's allies | *A* is a passenger of a mind-controlled open-topped transport and *T* is allied with the house that controls the transport (Edge cases) | low | A14, part l |
| strong scan | *A*'s target-health scan mode is *strong* and *T*'s expected remaining health is 0 or less | high | A8, part b |
| score | the score of claims 13 and 14 is exactly 0 | high | A7, part f |

A building's *threat value* is defined in Terms.

**Search outcome and target state** (one search, claim 4; excluding the charging break-off
search, whose outcomes belong to S-0518 claim 15).

| Target before | Condition | Result |
|---|---|---|
| none | a candidate exists | the best candidate becomes the target; if the search was the per-tick one of claim 1's first row or the teleport one, it is an opportunity target unless it is the object that was the target before the search (claim 24) |
| none | no candidate | no target |
| opportunity target | firing check *illegal* or *out of range* | not kept; the outcome is that of the two rows with target before "none" |
| opportunity target | firing check *impossible*, searcher controls no spawned aircraft | not kept; the outcome is that of the two rows with target before "none" |
| opportunity target | firing check *impossible*, searcher controls spawned aircraft | kept; spawn controller resets (S-0506 claim 11); released missiles fly on to their fixed aim points (S-0506 claim 12); no candidate is looked for |
| opportunity target | firing check *busy* | kept; no candidate is looked for |
| opportunity target | otherwise | kept; no candidate is looked for |
| any other target (for example one given by an order) | any | kept; no candidate is looked for |

The attack-move searches of claim 23 (a) that run as for an object with no target keep no target,
whatever the target before: for them the two rows with the target before "none" apply, and claim
23 (a) then gives what follows.

These rows are one search's. Between searches, claim 29's periodic check drops any held target
of a holder that claim 29 covers and that makes the check, when the firing check answers *illegal*
or *impossible* for it at a check tick, whether it is an opportunity target or not, the target that
the spawner row above keeps included.

For a searcher of the neutral or special house whose search can have wall candidates, the two
rows with the target before "none" are replaced by the table of claim 25, part i, which decides
between the wall target *W*, the best candidate and no target.

**Observable ordering.**
- Except for the charging break-off search (claim 1), the search's targeting-timer draw is
  its first draw. The charging break-off search takes no targeting-timer reset draw
  (S-0518 claim 15, scenario A17 here). A search takes no other draw, except the
  disguise draws of a searcher of the neutral or special house (claims 12 and 25). In searches
  covered by claim 4 these follow the timer draw; the charging search has no such draw to precede them. The teleport case can additionally take claim 1's conditional draw before the search's
  timer draw (scenario A1, part f). The timer reset at a dropped target (claims 27 and 28) is not
  a search draw: it comes inside the action that destroys, removes, cloaks or buries the object,
  at both cloak moments of claim 28. Its place against the destruction's own draws (S-0312
  claim 1) is open (Open questions). Claim 29's periodic check and its drops take no draw. At a crush the draws follow claim 27's order: per removed
  object in the cell layer's listing order, a vehicle's animation picks (S-0210 claim 4) before
  its holders' timer draws, the holders in creation order. Scenario A18 part f tests the
  holders' order and the order between removed objects (medium); the animation picks' place
  before the timer draws keeps S-0210 claim 4's `unmeasured` backing (medium).
  Each part of the search itself is `measurable`:
  - that a search takes no fixed further draw besides the timer draw: scenario A15, part h, case
    i, where a search tick on which nothing else draws advances the stream by exactly the raw
    values of the timer draw, and the targeting-timer restart shows which value it took,
  - that the draws do not depend on how many objects a searcher of a player house tests or
    scores: scenario A14, part n,
  - that a searcher of the neutral or special house takes exactly one more draw for each blinking
    disguise it tests: scenario A15, part c, run 3,
  - that the disguise draws come after the timer draw, not before it: scenario A15, part h, case
    ii. The two orders give the timer and the disguise test different raw values, so they predict
    different targeting-timer restarts and, with a list entry of 50, often a different pick.

  S-0215 claim 33 places the per-tick search's draws after those of the processing on the same
  tick. This is not an ordering rule for a search inside processing: a charging break-off pick
  can be attacked in that same processing (S-0518 claim 15).
- A target picked by a search is first acted upon as its context's owner says (the per-tick
  search's target by a later processing, S-0215 claim 33; the charging case by S-0518 claim 15).
- Candidates found earlier in the scan win ties (claim 17); an early stop (claim 9) can end a search
  before an outer candidate with a higher score is reached.
- A pick covered by claim 4 lowers the target's expected remaining health at once, subject to
  that claim's inaccurate-projectile and distributed-fire exceptions, so of two searchers that
  search in the same tick, the one that searches second sees the lowered value. For two searches in the same tick, the later-processed search (S-0203) sees the expected remaining health after the earlier search's pick has reduced it; consequently, in modes *strong* and *normal*, the picks can differ from the result of scoring both searches against tick-start health values.
  Scoring every searcher against the values at the start of the tick gives different picks in
  modes *strong* and *normal*.

## Edge cases and quirks

- **One object per cell** (claim 10). The original scores only the first eligible object of each
  visited cell, and looks only at the bridge deck when the deck holds an object. Switch
  `quirks.search_one_object_per_cell` (default on = original). Off: every infantry unit, vehicle,
  aircraft and building of the cell other than the searcher, in every layer, is tested against
  table C and scored (table C's rows on alliance then do the work of claim 10's selection), and
  `quirks.mind_controlled_passengers_cell_choice` has no effect.
- **Airborne scan reads sectors** (claim 9). The scan of airborne objects uses S-0303's coarse
  flying-object search, which can miss a tracked flying object in range that lies in an off-axis,
  off-diagonal sector. Switch `quirks.flying_search_sectors` (S-0303; default on = original), the
  same switch as for area damage. Off: every tracked flying object is looked at (table C still
  decides which are candidates).
- **Map-wide distance in leptons** (F4). In the map-wide search the distance of ground-layer
  candidates is in leptons while the weapon range term stays in cells, so the distance coefficient
  weighs 256 times more there than in other searches. Switch `quirks.hunt_distance_in_leptons`
  (default on = original). Off: *d* in cells as in the other searches.
- **A score of exactly 0 is rejected** (claim 13) while negative scores become 1. Switch
  `quirks.zero_score_rejected` (default on = original). Off: a score of 0 also becomes 1.
- **Passengers of a mind-controlled open-topped transport** (claim 10 and table C; low
  confidence). Such passengers fight for the controlling house: table C rejects the objects
  allied with that house (row "controller's allies") and not those allied with the passengers
  themselves (row "friendly" does not apply to them). The per-cell selection of claim 10,
  however, selects an object allied with the controlling house, which is then rejected, and falls
  back to the cell's last object only when the cell holds no such object. So in a range or area
  search a cell gives such a passenger a candidate only when no object of the cell is allied with
  the controlling house, and then only the cell's last object; an enemy of the controller that
  shares a cell with one of the controller's units is passed over. The scan of airborne objects
  has no per-cell selection and is not affected. Switch
  `quirks.mind_controlled_passengers_cell_choice` (default on = original). Off: such a passenger
  selects, in each cell, the first object that is not allied with the controlling house, and
  table C is unchanged.
- **Distributed fire finds nothing map-wide** (claim 20). A searcher of a distributed-fire type that
  runs a map-wide search (*hunt*) picks nothing: the pick of claim 20 works on the candidates of a
  range or area search, and a map-wide search collects none. Switch
  `quirks.hunt_ignores_distributed_fire` (default on = original). Off: a distributed-fire searcher
  in a map-wide search picks by claim 17 among the map-wide candidates, and its fired-at list is
  empty afterwards (a map-wide search never adds to it). This is our rule for the off setting; the
  original has no such behaviour.
- **A defeated enemy house stays the enemy house** (claim 16). A house keeps a defeated house as
  its enemy house until one of its anger values next changes, so its units' searches give no
  enemy-house bonus to anyone in the meantime. Switch `quirks.defeated_enemy_house_kept` (default
  on = original). Off: when a house is defeated, every house whose enemy house it is works its
  enemy house out again at once by claim 16's rule, which leaves defeated houses out.
- **The map-wide search skips hovering objects** (claim 7). An infantry unit, vehicle or building
  whose render layer is not the ground layer, such as a hovering unit with jumpjet movement, is
  never considered by a map-wide search. Switch `quirks.hunt_ground_layer_only` (default on =
  original). Off: the second pass also considers the infantry units, vehicles and buildings of
  every other render layer, with the zone test of claim 7 on the cell they are over, and scores
  them as the ground-layer objects of that pass (F4); aircraft keep their original eligibility
  in both passes (claim 7), and table C decides as for any other object (so the firing check rejects a hovering
  object for a weapon that cannot hit it).
- **Unused permanent-berserk mark** (Terms). The original has the mark and honours it in claim 10
  (any object is selected, but an allied one is not tested, so it hides the rest of its cell) and
  in table C (it is one of the friendly-row exceptions, which no allied object then reaches), but only a map trigger action and an action of its team scripts set it. Neither is in
  the multiplayer subset of S-0011, and the original's team scripts are out of scope, so in our
  engine nothing sets it. If a later spec adds such an action, the mark has the effects described
  in Terms, claim 10 and table C (low confidence, unmeasured while nothing sets it). No switch.
- **Unused alternative coefficients.** The rules data has a second set of five threat coefficients
  meant for less capable players. The original selects it only for a house created without a
  country, which does not happen, so it never takes effect. Our engine imports and ignores it, and
  the importer gives no warning.
- **Coefficient given as 0.** Each type coefficient starts at 0 (claim 26), so the replacement
  below also applies to the first reading without the key. A coefficient key that is absent from a layer keeps the held value,
  except that a held value of 0 is replaced by the matching game-wide default coefficient (as it
  stands when the type is read). The default is copied in the double precision in which it is
  held, not narrowed (Import mapping). So a coefficient written as 0 holds only until a later layer
  reads the section without that key. Switch `quirks.coefficient_zero_replaced` (default on = original,
  and the importer warns for a type coefficient written as 0 in either setting). Off: an absent key
  always keeps the held value, a 0 included.
- **Out-of-range scores.** A score sum of 2³¹ or more wraps by the common conversion (F4), and one of
  2⁶³ or more converts to 0 and is rejected. Our engine reproduces this, as the common conversion of S-0205 (claim 7) gives it, so there is no
  switch; the importer warns when a coefficient's magnitude exceeds 10⁶.
- **Healer test differs from S-0214.** The searcher is a healer for acquisition when its mean weapon
  damage is below 0 (S-0308, claim 15), while S-0214's *healer* looks at the weapon it would use.
  No switch.
- **Original only: campaign missions and the local player.** In a campaign mission, and only
  there, the original rejects a candidate *T* for a searcher *A* when *A* belongs to the local
  player, *T* is not an aircraft, *T* does not belong to the local player and *T* has not yet been
  discovered by the local player (revealed to that player's house). So the units of the person
  playing a campaign never pick, by themselves, an object that the person has not yet discovered. This is
  not a rule of our engine: it makes target choice depend on the local player, which the glossary
  allows only for what is drawn or heard, and campaigns are a pointer only in the current specs.
  Our table C has no such row, so in every game, a campaign mission included, objects that a house
  has not discovered are candidates like any other. Campaigns are out of scope for now; whether a
  later campaign spec should reproduce this through a rule on discovered objects is left to that
  spec (Open questions).
- **Original only for AI houses (note for AI designers): the computer player.** By the project
  lead's decision of 2026-10-09 an AI house follows the rules of this spec for human-controlled
  owners (Terms, houses in our engine). The original's computer player has the rules of claim 25,
  which our engine keeps only for the neutral and special houses (decision of 2026-10-10). For AI
  houses they are not claims about our engine; they are recorded so that our AI may reproduce any
  of them through ordinary orders. In short, the original's computer player:
  - targets unarmed enemy buildings and buildings whose threat value is 0, which a person's units
    skip (claim 25, part f),
  - retaliates even when it has a target, but not against an attacker whose threat sum is below
    its present target's, and not while it drains something (claim 25, part h),
  - sends engineers after capturable buildings, damaged allied buildings and a building its house
    has chosen, and some infantry after buildings of a multiplayer-passive house to capture or
    occupy (claim 25, parts c to e),
  - shoots an enemy wall that its search reaches before any candidate object, if its difficulty
    row allows it, and then looks no further, so the wall wins even over an object found later in
    the same ring unless that ring is an early-stop ring (claim 25, part i),
  - sees through a blinking disguise with a chance that depends on its difficulty row (claim 25,
    part g),
  - lets infantry with explosive charges search for buildings (claim 25, part b),
  - drops a held target that is allied with the holder at the start of each turn, with a few
    exceptions (claim 25, part k),
  - gives its houses an enemy house before any damage and clears a defeated enemy house (claim 25,
    part j),
  - and, only for the computer player and never for the neutral and special houses, gives team
    members the extra search rules of the script system.
- **A wall cell can win over an object in the same ring** (claim 25, part i). For a searcher of
  the neutral or special house, a wall candidate wins over a candidate object that comes later in
  the same ring, unless that ring is an early-stop ring of claim 9. Switch
  `quirks.wall_cell_over_same_ring_object` (default on = original). Off: when a candidate object
  has been found by the end of ring *r*, the search does not take *W*; it goes on as claims 9 and
  17 give, without further wall candidates (none can arise once a candidate object exists), and
  picks an object. When no candidate object has been found by the end of ring *r*, the search
  takes *W* and ends there, as with the switch on.


## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, type section, key `GuardRange` | *g* (F1, F3, F4) | cells, in the lepton form of S-0307 claim 1: the single-precision decimal of S-0003 claim 6, multiplied by 256 in double precision and truncated toward zero; a value read as exactly −1 keeps the held value | held value; engine default 0 |
| Rules, type section, keys `Underwater`, `Organic`, `SelfHealing` | the underwater exception of F1 (a type with all three uses *g* for *w*); S-0308 and S-0314 own the keys | boolean | default no |
| Rules, type section, key `AirRangeBonus` | *b* (F1) | cells, in the lepton form of S-0307 claim 1, as `GuardRange` | held value; default 0 |
| Rules, the combat-damage section (S-0006), key `OccupyWeaponRange` | *o* (F1) | integer cells | held value; default 5 (as S-0307) |
| Rules, type section, key `CanPassiveAquire` (spelled so in the original) | passive-acquire flag (claim 2) | boolean | held value; default yes |
| Rules, type section, key `CanRetaliate` | retaliate flag (claim 18) | boolean | held value; default yes |
| Rules, type section, key `OpportunityFire` | claim 3 | boolean | default no |
| Rules, type section, key `SprayAttack` | spray-attack type (claim 3; S-0313 owns the key) | boolean | default no |
| Rules, type section, key `LandTargeting` | land-targeting mode (table C, row "land targeting"; S-0308 owns the key) | integer, used as given | held value; default 0 |
| Rules, type section, key `DistributedFire` | claim 20 | boolean | default no |
| Rules, type section, key `AttackFriendlies` | claims 7 and 10, table C (rows "friendly" and "healer aircraft") | boolean | default no |
| Rules, type section, key `DetectDisguise` | table C, row "disguise" | boolean | default no |
| Rules, type section, key `NoAutoFire` | claim 5 | boolean | default no |
| Rules, type section, key `VHPScan` | target-health scan mode: `None`, `Normal`, `Strong` (claim 14) | word | held value; default none |
| Rules, type section, key `SpecialThreatValue` | *s*<sub>T</sub> (F4) | decimal, held in double precision: the double of S-0003 claims 6 and 7, never narrowed to single | held value; default 0 |
| Rules, type section, keys `MyEffectivenessCoefficient`, `TargetEffectivenessCoefficient`, `TargetSpecialThreatCoefficient`, `TargetStrengthCoefficient`, `TargetDistanceCoefficient` | *c*<sub>me</sub>, *c*<sub>te</sub>, *c*<sub>st</sub>, *c*<sub>sg</sub>, *c*<sub>d</sub> (F4) | decimal, held in double precision: the double of S-0003 claims 6 and 7, never narrowed to single | starts at double-precision 0 before any layer; thereafter held value, but with `quirks.coefficient_zero_replaced` on a held 0 is replaced by the matching game-wide default at this section reading, copied without narrowing; off always keeps the held value (claim 26) |
| Rules, `[General]`, keys `MyEffectivenessCoefficientDefault`, `TargetEffectivenessCoefficientDefault`, `TargetSpecialThreatCoefficientDefault`, `TargetStrengthCoefficientDefault`, `TargetDistanceCoefficientDefault` | the game-wide default coefficients | decimal, held in double precision: the double of S-0003 claims 6 and 7, never narrowed to single | held value; engine default 0 |
| Rules, `[General]`, keys `DumbMyEffectivenessCoefficient`, `DumbTargetEffectivenessCoefficient`, `DumbTargetSpecialThreatCoefficient`, `DumbTargetStrengthCoefficient`, `DumbTargetDistanceCoefficient` | unused (Edge cases) | decimal | — |
| Rules, `[General]`, key `EnemyHouseThreatBonus` | *B* (F4) | decimal, held in double precision: the double of S-0003 claims 6 and 7, never narrowed to single | held value; engine default 0 |
| Rules, the combat-damage section (S-0006), key `FireSupress` (spelled so in the original) | *L* (F6) | cells, in the lepton form of S-0307 claim 1, as `GuardRange` (× 256 in double precision, truncated toward zero; exactly −1 keeps the held value) | held value; engine default 256 leptons |
| Rules, weapon section, key `Supress` (spelled so in the original) | claim 14 | boolean | default no |
| Rules, weapon section, key `FireOnce` | claim 21 | boolean | default no |
| Rules, weapon section, key `AreaFire` | claim 3 | boolean | default no |
| Rules, projectile section, keys `AA`, `AG` | search classes (claim 5) | boolean | as the projectile specs give |
| Rules, type section, keys `LegalTarget`, `Insignificant` | table C | boolean | as S-0004 and the type specs give |
| Rules, building type section, keys `InvisibleInGame`, `Capturable` | table C | boolean | default no |
| Rules, type section, key `ThreatPosed`; `[General]`, key `ThreatPerOccupant` | threat value (table C) | integer | default 0 |
| Rules, `[AudioVisual]`, key `ConditionRed` | the red condition ratio (table C, row "engineer"; claims 10 and 25) | decimal, held in double precision as the double of S-0003 claims 6 and 7, and compared exactly with the double health ratio | as the health-bar spec gives |
| Rules, type section, key `Storage` | table C, row "storage" | integer | default 0 |
| Rules, `[General]`, key `HarvesterUnit`; map, special flag `HarvesterImmune` | table C, row "harvester immunity" | type list; boolean | empty; no |
| Rules, type section, keys `BalloonHover`, `IsSimpleDeployer` | claim 3 | boolean | default no |
| Rules, vehicle type section, key `DeployToFire`; building type section, key `Artillary` (spelled so in the original) | claims 5 and 18 | boolean | default no |
| Rules, infantry type section, key `Engineer` | claims 2, 5 and 8, table C (rows "friendly", "harmless building" and "engineer") | boolean | default no |
| Rules, infantry type section, key `Infiltrate` | claim 5 (the unarmed money thief) | boolean | default no |
| Rules, infantry type section, key `VehicleThief` | claim 5, table C (row "train") | boolean | default no |
| Rules, infantry type section, key `Thief` | claim 5, table C (rows "capture" and "storage") | boolean | default no |
| Rules, infantry type section, key `C4` | claim 18 (explosive charges) and the AI notes | boolean | default no |
| Rules, type section, key `IsTrain` | table C | boolean | default no |
| Rules, projectile section, key `Inaccurate` | the exception of claim 4 | boolean | default no |
| Rules, type section, key `AirstrikeTeam` | claim 24 (a type with an airstrike team above 0 can call airstrikes) | integer | default 0 |
| Rules, the combat-damage section (S-0006), key `PlayerReturnFire` | S-0215, claim 18 | boolean | as S-0215 gives |
| Rules, type section, keys `Strength`, `Cost` | *S*<sub>T</sub> (F5); *S* and, through the type's cost of S-0314 claim 5, *C* (F8) | integer | as the type specs give |
| Rules, `[General]`, keys `NormalTargetingDelay`, `GuardAreaTargetingDelay` | targeting timer (S-0215, F6), including claim 23's searches; neither is an arrival timeout | integer ticks | S-0215 |
| No data key | claim 23's arrival delay, memory refresh and removed-target conversion | fixed behaviour and integer 1-tick processing delay; no imported setting | not applicable |
| Rules, `[General]`, key `DisabledDisguiseDetectionPercent` | the disguise-detection list (claim 25, part g) | comma-separated list; each item is read as a plain decimal integer (optional sign and digits, reading stops at the first other byte); the items are in the order of the difficulty rows (easy, normal, difficult), and our engine reads the second | held value. A list without a second item is not covered (the original's result is then undefined): the importer warns, and our engine then lets no disguised target through by the draw (a departure) |
| Rules, difficulty row sections (S-0302, claim 3), key `DestroyWalls` | the destroy-walls flag (claim 25, part i) | boolean | yes |
| Rules, country section, key `MultiplayPassive` | multiplayer-passive house (Terms; table C; claim 25, part j) | boolean | default no |
| Rules, warhead section and overlay type section, key `Wall` | the wall flag of a warhead, and the overlay types that are walls (claim 25, part i) | boolean | default no |
| Rules, infantry type section, keys `Occupier`; building type section, `NeedsEngineer` | claim 25, part e | boolean | default no |


## Measurement scenarios

All scenarios run under the measurement setup with named test files (a test map with override
sections for the test types). Observations: which object the searcher fires at first, read from a
screen capture or a recorder of targets and health; tick numbers from the tick recorder. Unless a
scenario says otherwise, test firers are of a stock armed vehicle type in *guard*, owned by the
human house, and test targets are stationary enemy vehicles of the computer house whose weapons are
removed in the test map (so they never fire and never draw retaliation). Any game speed. Outcomes
that depend on the timer draw are read after at least 60 ticks, so the draw does not matter. Each
part is run 3 times from a fresh game, and a part passes only if all 3 runs give the expected
result; a part that says otherwise (for example A8, part c) gives its own number of runs. A part's
*Falsified if* line names the result that disproves it. Distances and ranges named in a part are
set in the test map's override sections.

**A1: search radius and when searches run (claims 1 to 3, 8).**
- *Part a:* firer with guard range 0 and weapon range *w* (from data); a target placed at
  *w* − 64 leptons and another, alone, at *w* + 64 leptons. *Expected:* the first is attacked, the
  second never. With guard range set to 2 × *w* in the test map: the second is attacked too.
- *Part b:* firer moving along a route past a target within range, opportunity-fire flag off, then
  on. *Expected:* off: no shot while moving, a shot after arriving and resting; on: a shot while
  moving. The same for *harvest*: a harvester type given a weapon in the test map, harvesting in a
  field with an enemy within range, flag off then on. *Expected:* off: no shot while it harvests; on:
  it fires while harvesting.
- *Part c (area-fire exception, claim 3):* an infantry type that deploys to fire, whose deployed
  weapon is its slot-1 weapon with the area-fire flag (S-0308 gives the deployed weapon whatever
  the target), deployed in *guard* with an enemy in range, with the target recorder. *Expected:*
  the recorder never shows a target picked by itself. Control: the same unit not deployed (it would
  use slot 0) picks the enemy. A further run: the same type with the opportunity-fire flag set in
  the test map, deployed in the same way, picks the enemy by itself (the flag lifts the exception).
  *Falsified if:* the deployed unit without the flag picks a target by itself, the control does
  not, or the deployed unit with the flag never picks the enemy.
- *Part d:* passive-acquire flag off: the firer never picks a target by itself in *guard*, but still
  retaliates. *Part e:* an engineer type of the human house, given an anti-ground weapon in its
  first slot in the test map (so that "armed" does not decide), in *guard* next to an enemy
  building that has a weapon and a threat-posed value above 0 (test map), with the target
  recorder. *Expected:* the recorder never shows a target picked by itself. Control: the same
  type with the engineer flag removed in the test map picks the building. *Falsified if:* the
  engineer picks the building by itself, or the control does not.
- *Part f (teleport case, claims 1, 3 and 4):* an armed non-harvester unit with a teleporting
  locomotor, given the opportunity-fire flag in the test map (so that claim 3 holds whatever
  behaviour it is in when phasing in ends), with no target, ordered to move next to an enemy
  candidate. Use the random-state and target recorders, also recording phasing in and the
  targeting timer. Follow S-0207 scenario D22 for the jump and the phasing-in end. Across 30
  runs, classify end ticks by the timer's remaining ticks immediately before the end event,
  repeating until at least five have more than 10 left and five have at most 10 left, and use
  only those where the unit still has no target. *Expected:* on that end tick the enemy becomes
  its opportunity target. With more than 10 ticks left, a draw over 4 to 8 precedes the normal
  search-timer draw of S-0215 F6; with at most 10 left, only the normal search-timer draw occurs.
  The final timer holds the latter draw's delay in both groups. No teleport-triggered search
  occurs on the jump tick. Control: remove the passive-acquire flag, which prevents the search
  without preventing the conditional draw; more than 10 ticks left gives only the 4-to-8 draw,
  and at most 10 gives neither. Further control: add the vehicle harvester flag; there is no
  phasing-in end and no teleport-triggered search or draw, though ordinary timer-expiry searches
  remain possible. *Falsified if:* the search occurs at the jump instead of the phasing-in end,
  any qualifying end tick has the wrong draws or their order, or either control differs. A run
  without the opportunity-fire flag is recorded only: whether it searches depends on its
  behaviour at the end (claim 3).
- *Part g (area cap, claim 8):* a firer placed with the *area guard* behaviour (S-0215, claim 26),
  guard range 0 and a weapon of range 10 cells, so that *R* would be 20 cells without the cap and is
  16 cells with it; one target at 15 cells from the firer and another, alone in another run, at 17
  cells. *Expected:* the first is attacked, the second never.
- *Part h (patrol floor, claim 8):* a firer with the *patrol* behaviour, guard range 0 and a weapon
  of range 2 cells (*G* = 2 cells, so 2*G* = 4 cells is below the floor of 7 cells): a target at 6 cells is
  attacked, another alone at 8 cells never. *Falsified if:* the target at 8 cells is attacked or
  the one at 6 cells is not.
- *Part i (garrison, claim 8, F1):* a 3 × 3 building with the occupant range bonus set to 5 cells
  and an occupant with a weapon of range 4 cells; targets alone in separate runs at 6, 9 and 10
  cells from the building's centre. *Expected:* with *n* = 1 + 1 + 5 = 7 rings, the target at 6 cells
  is attacked (S-0307 claim 4 replaces the occupant weapon's range by 6 cells); the targets at
  9 and 10 cells are never attacked. *Falsified if:* the target at 6 cells is not picked in a
  qualifying search or either farther target is picked.
- *Part j (passive acquisition, claim 2):* an occupiable building of the human house with no
  occupants and an armed type; an enemy in range. *Expected:* it never fires by itself; with an
  occupant inside, it fires. *Falsified if:* the empty building fires, or the occupied one does
  not.
- *Part k (mind controller at capacity, claim 2):* a mind-controlling type of the human house that
  already controls as many objects as it can, in *guard*, with an enemy in range. *Expected:* it
  never picks the enemy by itself. After one controlled object is lost (so it is below capacity)
  it does. *Falsified if:* the full controller picks, or the one below capacity does not.
- *Part l (harvesting slave, claim 2):* a harvesting slave of a slave master (S-0215), given a weapon in the test
  map so that "armed" does not decide, harvesting with an enemy in range, with the target recorder.
  *Expected:* it never picks the enemy by itself while it harvests. Control: a unit of the same
  armed slave type placed alone by the test map, belonging to no slave master, picks the enemy.
  *Falsified if:* the harvesting slave picks the enemy by itself.
- *Part m (arrival, claim 3):* a vehicle type with the balloon-hover flag, then a vehicle type that
  deploys in place, each moved to a cell next to an enemy in range, opportunity-fire flag off.
  *Expected:* no shot while moving; after arriving (own cell as destination, no planned route) it
  picks the enemy by itself. A type with neither property does not.
- *Part n ("armed", claim 2):* a vehicle type whose first weapon slot is empty and whose second
  slot holds an anti-ground weapon (set in the test map), of the human house, in *guard* with an
  enemy vehicle within that weapon's range, with the target recorder. *Expected:* the recorder never
  shows a target (the object is not armed in the sense of S-0214, so it does not pick by itself).
  Control: the same object ordered to hunt (the map-wide search is not limited by claim 2) shows
  the enemy as its target. *Falsified if:* a target appears in *guard*. (If the control shows no
  target either, the part does not decide the rule, and claim 5 or table C should be re-examined.)
- *Part o (spray-attack form, claim 3):* a type with the spray-attack flag whose slot-0 weapon has
  the area-fire flag and whose slot 1 holds no area-fire weapon, in *guard*, with an enemy within
  range and the target recorder. *Expected:* the recorder never shows a target picked by itself
  (with no target, the weapon it would use is the one in slot 0, which has the area-fire flag).
  Control: the same type without the spray-attack flag picks the enemy. *Falsified if:* the
  spray-attack type picks a target by itself, or the control does not.
- *Falsified if:* any expected outcome of parts a to o differs.

**A2: search classes (claim 5, table C class row).**
- *Setup:* firer types with only anti-ground, only anti-air and both projectiles; targets: an
  infantry unit, a vehicle, a building with a weapon and a threat-posed value above 0 (test map, so
  that table C's row "harmless building" does not reject it) and an airborne aircraft, all within
  range.
- *Expected:* each firer attacks only the classes its projectiles allow. An infantry firer whose
  primary warhead has 0 % against *medium* and *wood* ignores the vehicle and the building even at
  close range. A human-owned vehicle type that deploys to fire never attacks by itself.
- *Part b (vehicle thief):* a human-owned unarmed vehicle-thief infantry unit ordered to hunt (an
  unarmed unit does not search in *guard*, claim 2) on a map with enemy infantry, an enemy building
  and an enemy vehicle. *Expected:* it picks the vehicle. With the vehicle removed: it picks
  nothing. *Falsified if:* it picks the infantry unit or the building.
- *Part c (money thief):* an unarmed money-thief infantry type with the infiltrate flag, of the
  human house, ordered to hunt, on a map with an enemy refinery-like building with storage and an
  enemy building without storage, both capturable and both given a weapon and a threat-posed value
  above 0 in the test map (so that the row "harmless building" does not reject them), and an enemy
  vehicle.
  *Expected:* only the building with storage is picked; with it removed, nothing is picked.
  *Falsified if:* the building without storage or the vehicle is picked.
- *Part d (aircraft):* an armed aircraft landed on a pad in *guard*, enemies in range. *Expected:*
  it never picks a target by itself through these searches (its own behaviour searches belong to
  the aircraft specs).
- *Part e (no-auto-fire, claim 5):* a human-owned armed vehicle type with the no-auto-fire flag, an
  enemy in range. *Expected:* it never fires by itself, and fires when ordered to attack.
- *Part f (unarmed infantry, claims 2 and 5):* an ordinary unarmed infantry type (no thief or
  infiltrate flag) in *guard* and, in another run, ordered to hunt, with enemy infantry, vehicles
  and buildings in range or on the map. *Expected:* it never picks a target by itself. The same
  with an unarmed infantry type that has the infiltrate flag only (a plain infiltrator):
  nothing is picked.
- *Part g (engineer, claim 5, table C):* a human-owned engineer type, given an anti-ground weapon
  in its first slot in the test map (so that "armed" does not decide), whose warhead has versus
  values above 0 against every armour class on the map, ordered to hunt on a map
  with enemy infantry and enemy vehicles only, with the target recorder; then with one enemy
  building added that has no weapon (so that only the engineer's exemption from the row "harmless
  building" lets it through). *Expected:* it picks no infantry or vehicle; with the building
  present it picks the building. Control: the same type with the engineer flag removed in the
  test map picks an infantry unit or vehicle, and in a run with the weaponless building as the
  only enemy it picks nothing. A further run: the engineer type with its weapon removed, with the
  building present, picks nothing (an unarmed engineer finds no targets, claim 5). *Falsified if:*
  the armed engineer picks an infantry unit or vehicle, or does not pick the building; the control
  picks the weaponless building or picks no infantry or vehicle; or the unarmed engineer picks
  anything.
- *Part h (search for vehicles, claim 5):* a human-owned vehicle firer whose weapons are
  anti-ground only, with an enemy aircraft landed on open ground (not on a pad) within range and
  nothing else. *Expected:* it attacks the aircraft by itself (a landed aircraft counts as a
  vehicle). *Falsified if:* it never attacks it.
- *Part i (engineer and damaged allied buildings, table C row "engineer"):* the armed engineer type
  of part g, also given the attack-friendlies flag in the test map, of the human house, ordered to
  hunt on a map whose only enemy object is an infantry unit far away (so that the game goes on; an
  engineer never searches for infantry) and with two buildings of the human house, of a type that accepts
  repair by an engineer (structures specs) and has no weapon: *B*₁ at a health ratio below the red
  condition ratio and *B*₂ at full health, no other building of the human house below full health,
  and the target recorder. *Expected:* it picks *B*₁; in a
  run with *B*₁ removed it picks nothing. *Falsified if:* it picks *B*₂, or picks nothing while
  *B*₁ is present.
- *Part j (armed vehicle thief and the combined infantry rules, claim 5):* human-owned infantry
  types made in the test map, each ordered to hunt on a map with one enemy infantry unit, one
  enemy vehicle and one enemy building of a capturable type with storage capacity, given a weapon
  and a threat-posed value above 0 (so that the row "harmless building" does not reject it), with
  the target recorder. Run 1: an engineer type with an anti-ground weapon in its first slot whose
  warhead is infantry-only (versus exactly 0 against *medium* and *wood*, above 0 against the
  infantry unit's armour class). *Expected:* it picks nothing. Control: the same type without the
  engineer flag picks the infantry unit. Run 2: an unarmed type with the infiltrate, vehicle-thief
  and money-thief flags. *Expected:* it picks the building; with the building removed it picks
  nothing (never the vehicle). Run 3: the type of run 2 given an anti-ground weapon. *Expected:* as
  run 2. Run 4: an armed vehicle-thief type without the money-thief flag, with an anti-ground
  weapon, in *guard* with only the enemy infantry unit in range. *Expected:* it picks the infantry
  unit (an armed vehicle thief takes its weapons' classes). *Falsified if:* run 1 picks anything
  or its control picks no infantry unit; run 2 or 3 picks the vehicle or the infantry unit, or
  does not pick the building; run 4 never picks the infantry unit.
- *Falsified if:* a firer attacks a class its projectiles exclude, or ignores one they allow; or
  parts b to j differ.

**A3: one object per cell (claim 10).**
- *Setup:* firer with guard range 0; one cell within range holds two enemy infantry units, *I*₁ of a
  type against whose armour class the firer's warhead is 1 % and *I*₂ of a type against which it is
  100 %. Two runs, with the two units entering the cell in either order. Control: *I*₂ alone in a
  cell.
- *Expected:* in at least one of the two runs the firer attacks neither unit (when *I*₁ is the
  object the cell offers first, it fails the versus row and *I*₂ is not looked at). The control is
  attacked. Which run it is shows the order in which a cell holds its objects (Open questions).
- *Falsified if:* *I*₂ is attacked in both runs while the control is attacked.
- *Part b (bridge deck, claim 10):* the same two infantry units as in the main setup in one bridge
  cell, *I*₁ (1 %) on the deck and *I*₂ (100 %) under the bridge. *Expected:* the firer attacks
  neither (only the deck object is looked at, and it fails the versus row), and *I*₂ alone under
  a bridge with nothing on the deck is attacked. *Falsified if:* *I*₂ under the bridge is attacked
  while *I*₁ stands on the deck.
- *Part c (berserk searcher, claim 10 and table C row "friendly"):* a firer of the human house with
  only an allied vehicle (of the human house) within range, made berserk by a hit from a weapon
  whose warhead makes its victims berserk (S-0302, Terms). *Expected:* while berserk it attacks the
  allied vehicle by itself; before the hit, and after the berserk state ends, it does not.
  *Falsified if:* it attacks the allied vehicle while not berserk, or never attacks it while
  berserk.

**A4: ring order and early stops (claims 9 and 17).**
- *Setup (parts a to c):* firer with guard range 8 cells (*n* = 8, early stops after rings 2 and 4)
  and a type with distance coefficient 0 and own-effectiveness coefficient positive.
- *Part a:* target *X* (versus 50 %) in ring 2 and target *Y* (versus 100 %) in ring 3. *Expected:*
  *X* is attacked although *Y* scores higher.
- *Part b:* *X* moved to ring 3 as well. *Expected:* *Y*.
- *Part c:* *X* and *Y* equal in score and both in ring 1. *Expected:* the one earlier in F2's order
  is attacked (around centre (*x*, *y*): (*x* − 1, *y* − 1), (*x* − 1, *y* + 1), (*x*, *y* − 1),
  (*x*, *y* + 1), (*x* + 1, *y* − 1), (*x* + 1, *y* + 1), (*x* − 1, *y*), (*x* + 1, *y*)); the test
  is run with *X* and *Y* swapped between the two cells as well, 3 runs each.
- *Falsified if:* *Y* is picked in part a, or ties go the other way in part c.
- *Part d (airborne scan, S-0303's F7):* firer with an anti-air weapon and guard range 8 cells on a map of at least 20
  cells in each direction, and two identical hovering aircraft of the enemy, one in a sector on the
  firer's row and one in an off-axis, off-diagonal sector, both within range and at equal score.
  *Expected:* only the first is ever picked (with `quirks.flying_search_sectors` on); with both
  on the firer's row, the one in the sector nearer the centre sector in S-0303's F7 order. *Falsified if:*
  the off-axis aircraft is ever picked.
- *Part e (rings from the weapon range, claim 8, F1):* as parts a and b, but with guard range 0 and
  a weapon range of 7 cells, so that *n* = 7 + 1 = 8 (early stops after rings 2 and 4). *X* in
  ring 2, *Y* in ring 3. *Expected:* *X* (without the + 1 of F1, *n* = 7 and the stops would fall
  after rings 1 and 3, so *Y* would win). *Falsified if:* *Y* is picked.
- *Part f (variants of F1's ring count):* as part e, with *X* in ring 2 and *Y* in ring 3, in
  separate runs: (1) weapon range 6 cells and an air range bonus of 1 cell, so *n* = 6 + 1 + 1 = 8:
  *expected X* (without the bonus, *n* = 7: *Y*); (2) a switching type whose weapon of its weapon
  number has range 7 cells and whose other slot has range 11 cells, so *n* = 8: *expected X* (with
  the larger range, *n* = 12, stops after rings 3 and 6: *Y*); (3) a type that is underwater,
  organic and self-healing, with guard range 0 and weapon range 7 cells, so *n* = 0 + 1 = 1 and
  only ring 0 is looked at: *expected* neither is picked. *Falsified if:* any run gives the other
  result.

**A5: retaliation conditions (claim 18).**
- *Setup:* defender in *guard*, attacked by an enemy vehicle beyond its own range but within its
  sight; variants: retaliate flag off; defender a harvesting slave (S-0215); defender a slave
  master; defender of a spawner type whose spawned aircraft are all docked, none launching (and,
  as a second run, the same while they are launching); defender with mean weapon
  damage 0 (S-0308, claim 15); an attacker for which the firing check of the defender's weapon
  answers *illegal* (for example an aircraft attacking a defender whose weapon is anti-ground
  only), and one for which it answers *impossible* (a test weapon that cannot be fired at that
  attacker); an attacker allied with the defender (a unit of the defender's own house whose type
  has the attack-friendlies flag in the test map, ordered to attack it); attacker disguised as a
  friendly unit (an enemy spy-like type); human defender with
  explosive charges attacked by a building; a human defender of a type without the
  explosive-charges flag whose veteran ability list (test map) holds the explosive-charges ability,
  placed at veteran rank (S-0311, claim 11) and attacked by a building, and the same defender at
  rookie rank as its control, which retaliates; a mind-controlling defender that already controls as
  many objects as it can; a vehicle defender that deploys into an artillery building; a defender
  with a parasite attached, the parasite being the attacker. In every variant but the
  mean-damage one, a defender whose stock type has no weapon, or a weapon of mean damage 0, is
  given an ordinary weapon in the test map, so that only the named condition differs from the
  plain variant.
- *Expected:* retaliation only in the plain variant and in the rookie control of the
  veterancy-ability variant.
- *Falsified if:* any other variant retaliates, or the plain one or the rookie control does not.
- *Part b (targeting modes):* a stock powered anti-air defence building, of the human house and,
  as a second set, of a computer house, with no aircraft about. An enemy ground vehicle attacks
  it on clear land from inside the building's weapon range, and in a second run enemy infantry
  does. Choose attackers whose armour the building's warhead does not wholly fail against (a
  versus value above τ<sub>r</sub>, S-0304 claim 3, checked in the rules file before the runs),
  so that only the targeting mode can refuse. Check in the same file, before the runs, that the
  stock building's type has land-targeting mode 1 or the SAM flag (S-0215 claim 32); if it has
  neither, the stock building's runs are void and the record says so, since claim 18 then
  predicts no refusal for it. Check as well that its type's retaliate flag is set; if it is not,
  its runs are void. Every defender, stock or test copy, meets claim 18's other conditions and
  S-0215 claim 18's: its type's retaliate flag is set (in the test file for the copies), it is
  armed, it is in *guard*, it holds no target and has not fired before the first hit, and its
  sight range is at least its weapon range, so that an attacker inside its weapon range is
  within its sight limit. Variant 1: a test copy of that building type with
  land-targeting mode 0 and the SAM flag off (S-0215 claim 32), whose weapon uses a test warhead
  with every versus value at 100%, otherwise unchanged, attacked the same way. Variant 2: a test
  vehicle type whose only weapon's projectile is anti-air only and whose warhead is that test
  warhead, in two copies with land-targeting modes 0 and 1, attacked the same way. Control: a
  stock anti-ground defence building of the same house. Record whether the defender fires at the
  attacker within 300 ticks of the first hit, 5 runs each. Retaliation does not restart the
  defender's processing timer (S-0215 claim 10), so its first processing in *attack* can wait
  until the *guard* processing delay then running ends. Before the runs, check in the behaviour
  data (S-0215 Import mapping) that the upper bound of that delay (S-0215 claim 6) is below 300
  ticks; if it is not, the deadline is that bound plus 300 ticks. The defender has not fired
  before the first hit, so no rearm countdown adds to it. *Expected:* no shot from the stock
  building or the mode-1 vehicle copy; the mode-0 building copy of Variant 1 and the mode-0
  vehicle copy of Variant 2 fire back; the control fires back. *Falsified if:* the stock
  building or the mode-1 vehicle copy fires, or a mode-0 copy does not (a
  refusal outside the firing check and S-0215 would then exist); if the control does not fire,
  the run is void. Variant 3 (out of range): Variant 1's building copy, of the computer house,
  attacked by an enemy ground vehicle whose weapon outranges it, with the target recorder. The
  computer house is an ordinary player house, not a test-only house (claim 25). The copy's
  sight range is set in the test file at least 2 cells above its weapon range, and the attacker
  fires from a cell beyond the copy's weapon range but within its sight limit (S + 0.5 cells,
  S-0215 claim 18). The copy has not fired before, so no rearm countdown runs. *Expected:* once the
  copy takes the attacker as its target, it fires no shot and has dropped the target by the first
  processing in *attack* after it took it (the answer is then *out of range*). The attacker
  keeps hitting the copy, so retaliation may take it again at once: the recorder must show the
  drop, with no shot, before any re-take. *Falsified if:* it fires, or the recorder shows no drop
  of the attacker at or before that processing; the run is void if it never takes the attacker
  as its target.

**A6: keeping and replacing targets (claims 4, 19, 21, 23, 24, 29).**
- *Part a:* a firer whose type has the attack-friendlies flag (set in the test map), with *X* and
  *Y* of the firer's own house, so that the person can move them, and no other object within the
  firer's range. The firer picks *X* by itself; *X*
  is then moved out of range by an order, while *Y* stands in range. *Expected:* at the next search the firer switches to *Y* (an opportunity target is
  replaced). *Part b:* the same with *X* given by an attack order: the firer follows or keeps *X*,
  never switching to *Y* by itself.
- *Part c:* a test weapon with the fire-once flag: after one shot the firer has no target and picks
  again at its next search.
- *Part d (attack-move row of claim 1):* an attack-move order with a destination 20 cells away,
  along a route that passes an enemy target within the unit's weapon range, with the target
  recorder and the random-state recorder (which shows the targeting-timer draws). *Expected:* at a
  targeting-timer draw that falls while the target is within the unit's reach, the recorder shows
  that target as the unit's target. Whether the unit then stops or walks on is not part of this
  part (scenario A6, part g). *Falsified if:* the unit has timer draws while the target is within its
  reach and never has it as its target.
- *Part e:* order a unit with a locomotor-warhead primary to attack a vehicle inside a tank bunker:
  it fires at the bunker. The same with a unit whose weapon is a temporal weapon (it fires at the
  bunker) and with an ordinary unit (it fires at the vehicle, or cannot).
- *Part f (spawner, busy, impossible and re-pick cases, claims 4 and 24):* firers whose types have
  the attack-friendlies flag (test map), with targets *X* and *Y* of the firer's own house, so that
  the person can move them, no other object within the firer's range, and the target recorder
  running. *Busy:* a firer of a spawner type (S-0301) picks *X* by itself; on every search made
  while its spawned aircraft are launching, the target stays and the spawned aircraft keep theirs.
  *Impossible:* a firer that is not of a spawner type, whose weapon is given a rate-of-fire delay of
  200 ticks in the test map, picks *X* by itself and fires at it once; a run in which *X* is
  destroyed decides nothing. *X* is unarmed and of a type
  that does not cloak by itself, and there is a cloaking building of the firer's house whose field
  lies within the firer's range, and no sensor of the firer's house on the map. While the countdown
  runs, the person moves *X* onto a cell at the edge of the field, where it stops and cloaks, and by
  scripted input orders it one cell out of the field in the tick in which it becomes fully cloaked.
  The span from that tick to the start of its uncloak is the hidden span; a run in which a tick
  whose tick number is a multiple of 16 falls in it decides nothing, since claim 29 would drop *X*
  there. While it is fully cloaked, *X* is hidden from the firer's house and the firing check
  answers *impossible* for it (S-0516 claim 24, the firer's mean weapon damage being above 0).
  S-0516 claim 14 keeps it at both cloak moments, since the holder has the same owner. *Expected:* a
  search of the firer within the span drops *X*, and the same search picks *X* again (table C's
  cloak row does not reject an object of the searcher's own house, and table C does not reject on
  *impossible*), so the recorder shows *X* throughout; *X* is then no longer an opportunity target
  (claim 24). Once *X* is out of the field and seen again, it is moved out of range with *Y* in
  range: the firer keeps *X* and never switches to *Y* by itself (compare part a). A run decides
  this case only if the firer's search falls within a hidden span that holds no tick whose tick
  number is a multiple of 16. The run is repeated until a search falls within such a span, as
  part m repeats for its window; no setup here is shown to give such a span, so claims 4 and 19
  rate this case without it (`unmeasured`). Record the cloak state of *X*
  and the tick numbers of the span. *Falsified if:* the spawner's target changes while it is
  launching, or in a deciding run the firer switches to *Y*, or the recorder shows no target at the
  end of the search tick in the hidden span. (A spawner's keep on *impossible* and its controller
  reset have no case here: claim 4's Backing.)
- *Part g (attack-move, claim 23):* an attack-move order on a cell 20 cells away, through an enemy
  standing 6 cells from the route and within weapon range. Record position, target, current and
  queued behaviour, memory, engaged mark and targeting timer every tick. *Expected:* after the
  first successful search the unit stays engaged across tick starts and attacks the stationary
  enemy; timer expiries while it is within reach take no search draw. When the enemy is removed
  and attack processing falls back, it resumes the remembered destination and ends the engaged
  mark, rather than resuming on the tick immediately after acquisition. Run 2, with a memory
  holding only a target: order an attack move on enemy *X* 20 cells away, with a second enemy
  *Y* 6 cells from the route and within weapon range, where *X* is beyond the search radius and
  reach. *Expected:* a search picks *Y* and the engagement survives tick starts; on the later
  fall-back it resumes *X*, provided *X* still exists. A run without *Y* keeps *X* while it lives:
  an unsuccessful search restores *X* within the same tick. A third run moves the engaged target
  beyond reach and leaves another candidate within the search: the next expiry searches again
  and picks that candidate. *Falsified if:* an ordinary next-tick start ends the engagement,
  in-range expiries take search draws, or a qualifying fall-back fails to resume the memory.
- *Part h (redirects, claim 24):* order a unit to attack itself (its target becomes its own cell),
  order a unit to attack a target that dies in the same tick (the unit ends with no target); order
  an infantry unit to attack another that is in a death sequence (no target). Sinking target: an
  enemy naval vehicle that meets the sinking conditions of S-0312 (claim 10) is killed on water and
  sinks (it keeps 1 health point and stays in play); while it sinks, a unit is given an attack
  order on it by scripted input, and in a second run a firer in *guard* has the sinking vehicle as
  the only enemy within its range. Target recorder in every run. *Expected:* as claim 24: the unit
  ordered to attack itself has its own cell as its target, and in the other runs the target
  recorder never shows the dead, dying or sinking object as a target. If the scripted order on
  the sinking vehicle produces no order (S-0214), that run does not decide the rule, and the
  *guard* run decides it only if no other row of table C rejects a sinking vehicle. *Falsified
  if:* the self-attack gives the unit itself or no target, or the recorder shows a dead, dying or
  sinking object as a target in any run.
- *Part i (spawner pending targets, claims 4, 23 and 24):* use the ordinary carrier setup,
  timed shot-to-pass input and recorder of S-0506 scenario R19, under this scenario's common
  settings. Record own, controller and pending targets and the next adoption. Between a shot
  and the next pass, arrange separately: a nonempty replacement with no further shot, an
  unusable opportunity-target drop, a failed search while already targetless, and an
  attack-move search that does not retain its present target. Repeat with empty and nonempty
  controller targets. A redirected dead-target assignment is tested only if normal input can
  supply it as in part h; otherwise that run is void. Expected: direct nonempty replacement
  and an already-targetless failed search leave pending unchanged. An opportunity-target drop
  or a redirected assignment leaving no target clears pending only with a nonempty controller
  target; it does not immediately clear the attacking aircraft's targets. An attack-move
  search that ignores the present target leaves pending unchanged, including when it finds
  nothing or restores the remembered target. Keep all referenced targets alive through the
  observation except the explicitly tested dead target, to separate loss notifications.
  Run each reachable case three times. Falsified by different pending changes or adoption,
  after excluding another shot, a target-loss notification or a controller reset.
- *Part j (airstrike redirect, claim 24):* a unit of a type that can call airstrikes (it has an
  airstrike team) is ordered to attack an enemy vehicle that sits in a tank bunker, with no
  airstrike in progress. *Expected:* the target recorder shows the bunker building, not the vehicle,
  as its target. Control: a unit of a type without an airstrike team, a temporal weapon or a
  locomotor-warhead primary, given the same order, has the vehicle as its target. Run 2, with an
  airstrike in progress: the same unit first calls an airstrike on another enemy object (the
  airstrike action of S-0214) and, while the aircraft are on their way, is ordered to attack the
  vehicle in the bunker. *Expected:* the bunker building is its target in both runs. *Falsified
  if:* the airstrike-capable unit's target is the vehicle in either run.
- *Part k (engineer exception of F1, claim 8):* the armed engineer type of scenario A2, part g,
  with a guard range of 10 cells and a weapon range of 1 cell (test map), of the human house, given
  an attack move to a cell 20 cells away along a route whose nearest point is 5 cells from an
  enemy building that has a weapon and a threat-posed value above 0 (test map), with the targeting
  delays (S-0215, F6) set short in the test map (for example 15 ticks) so that several targeting-timer
  draws fall while the unit is within 10 cells of the building, and the target recorder.
  Further along the route a second enemy building of the same kind stands next to the route,
  within the engineer's 1-cell reach. *Expected:* the engineer never has the first building as
  its target (its range search uses *R* = 0, so a candidate must be within its weapon's reach),
  and it has the second one as its target when it passes it (which shows that its attack-move
  searches run). Control: the same type with the engineer flag removed in the test map, given the
  same order, has the first building as its target at one of those draws (*R* = 10 cells,
  *n* = 10). *Falsified if:* the engineer has the first building as its target, the engineer
  picks neither building, or the control never has the first one.
- *Part l (targets of attack-move searches are not opportunity targets, claim 23):* a vehicle
  firer whose type has the attack-friendlies flag (test map), with an attack-move order to a cell
  20 cells away; a vehicle *Y* of the firer's own house (so that the person can move it) stands
  next to the route within the firer's weapon range, and no other object is within the firer's
  search radius along the route. Target recorder and random-state recorder. When the recorder
  first shows *Y* as the firer's target, *Y* is ordered away, out of the firer's reach and search
  radius. Also record the opportunity-target and engaged marks. *Expected:* the acquisition
  sets the engaged mark but not the opportunity-target mark. While engaged, the next timer
  expiry beyond reach runs a search as for an object with no target, irrespective of the absent
  opportunity-target mark. With no candidate it is left targetless, then its fall-back resumes
  the remembered destination. *Falsified if:* the search marks *Y* as an opportunity target,
  or the engaged out-of-reach case keeps *Y* without searching.
- *Part m (an opportunity target that becomes illegal, claim 19):* a human-owned vehicle firer
  whose weapons are anti-ground only and whose type has the attack-friendlies flag (test map), in
  *guard*; an aircraft *X* of the firer's own house landed on open ground within range (so the
  firer picks it, as in scenario A2, part h), and a vehicle *Y* of the firer's own house within
  range, later than *X* in the scan order, with the target recorder. *X* is of an aircraft type
  that can stay in the air over one place. After the firer has picked *X* by itself, *X* is
  ordered to take off and to fly to a cell within the firer's range, where it stays in the air.
  Record, with tick numbers, *X*'s height, the firer's target and each of the firer's searches.
  *Expected:* the firer drops *X* (the firing check answers *illegal* against an aircraft in the
  air) in a tick whose tick number is a multiple of 16 (claim 29), or at one of its searches,
  and no later than the first tick after the tick in which *X* reaches its flying height whose
  tick number is a multiple of 16. That deadline follows claim 29's phase rule whether the
  refusal begins before or after the firer's turn. Its first search after the drop picks *Y*.
  The part decides only if *X* stays in the air within the firer's range from the tick in which
  *X* reaches its flying height to the firer's first search after the drop, and the firer takes
  its turn in every tick of that span (S-0203 claim 10). A drop in a tick whose tick number is not
  a multiple of 16 and at no search of the firer is recorded with its tick and decides nothing
  (Open questions, *Other drops at the start of a turn*). *Falsified if:* the firer still holds
  *X* at the end of that first tick, or it has no target after its first search after the drop.
- *Part n (uncontested arrival, Q-0162 questions 1 to 3):* a tiny flat skirmish map, one
  human-owned ordinary armed infantry unit and one ordinary armed vehicle, tested separately,
  plus one bystander computer player kept away. Stock types, no engine add-on and no mods (the named test files of a scenario excepted), game speed
  6, no routes, guard spot, contact, deployment or enemy within the traversed search area.
  Script an attack move to an empty cell. Record position, movement, current destination,
  target, memory, engaged mark, current and queued behaviour, processing timer and targeting
  timer every tick, from the order to 300 ticks after arrival. *Expected:* the memory survives
  travel; the first ordinary arrival fall-back while not engaged and in *move* removes it,
  and that processing chooses 1 tick. It queues the resting behaviour of S-0215 claims 14 and
  15, which becomes current at a later permitted start, rather than an indefinite sequence of
  1-tick *move* processings with memory. Record the imported rest fields to predict which
  resting behaviour. Repeat with an enemy beyond both the weapon reach and the resulting
  resting search area, weaponless to prevent retaliation: it does not prompt a new advance.
  *Falsified if:* the memory disappears during ordinary travel without a fall-back, survives
  that qualifying arrival fall-back, or the actor remains in *move* with it for the whole
  300-tick observation. A run with another fall-back on the arrival tick is recorded separately
  for the once-per-tick qualification, not counted as the first-fall-back test.
- *Part o (remembered target removed, Q-0162 question 4):* the same map and recording, with
  a weaponless enemy vehicle *X* as the attack-move target, outside weapon reach. A separate
  human unit destroys *X* before the attack mover arrives. Choose a target type removed at
  once on destruction (S-0312), and record its final position before removal. *Expected:*
  removal changes the remembered target to the cell of that final position, without itself
  removing the memory or changing the engaged mark. Run 1 removes *X* before an intervening
  successful automatic search: the attack mover is not engaged, and a fall-back in *attack*
  clears the converted memory by part (b); ordinary rest rules decide its behaviour. Run 2
  first lets the mover acquire a second enemy *Y* within reach, then removes *X* while the
  mover is engaged on *Y*. After *Y* is also removed, the engagement's fall-back resumes a
  move to *X*'s last cell. With no further targets, arrival clears the memory as part n gives,
  with a 1-tick arrival-processing delay, not an indefinite arrival loop. Keep the two
  removal cells distinct and farther apart than the search radius so the destination is
  distinguishable. *Falsified if:* removal alone erases the memory or engages the actor,
  the converted destination is not *X*'s last cell, or a qualifying non-engaged fall-back
  in *attack* preserves it. Repeat each run three times; a run without the specified engaged
  state at removal does not decide that case.
- *Part p (the periodic held-target check, claim 29):* the test warhead *H*, the test weapon *W*
  and the defender *D* of S-0304 scenario V1 (a stock vehicle type whose single weapon is
  replaced by *W*), with *H*'s value 0 % against the armour class of the type of the target *X*
  and 100 % against every other class. *X* is an enemy vehicle of the computer house whose weapon
  is replaced by one with 0 % against every class, so that it never fires, standing 3 cells from
  *D* on open ground that *D* can reach, within *W*'s range (as in S-0304 V1 run 2). Record every
  tick, with tick numbers: *D*'s target, shots, rearm countdown, targeting
  timer and current behaviour, and the random-state recorder's draws with their ranges.
  - *Run p1:* *D*, which has not fired, is ordered by scripted input to attack *X* with
    force-fire held. The order is applied in four series, in ticks whose tick numbers leave the
    remainders 1, 5, 9 and 13 when divided by 16, three runs each, and the tick of application
    is recorded for each run. Repeat with an infantry *D* of a stock type whose single weapon is
    replaced by *W*. For run p1 the test map gives the normal targeting delay (S-0215, F6) a
    value of 200 ticks, so that *D*'s targeting timer, restarted by its last search in *guard*
    before the order, usually has more than 10 ticks left at the drop tick, where claim 27's
    reset would draw. A run in which *D*'s targeting timer has 10 ticks or fewer left at its turn
    of the drop tick decides nothing for the absence of a timer draw, and each series is repeated
    until it has at least one run, for each holder kind, that decides it. *Expected:* *D* makes
    no shot at *X* and drops it in its turn of the first
    tick whose tick number is a multiple of 16 after the tick in which the order is applied, so
    15, 11, 7 and 3 ticks after that tick in the four series. A run in which *D* misses its turn
    in that tick (S-0203 claim 10) decides nothing.
    No draw over 4 to 8 ticks falls in *D*'s turn of that tick, and its targeting timer counts
    on without a reset there. *D*'s behaviour stays *attack* up to its next processing and then
    does what S-0215 gives with no target.
  - *Run p2:* as run p1 with the vehicle *D*, whose weapon *W* is given a rate-of-fire delay of
    40 ticks in the test map. *D* first fires at a second enemy vehicle *Y*, an armed-and-legal
    target, and is ordered onto *X* with force-fire while its countdown from that shot runs, as
    in S-0304 V1 run 4b. *Expected:* *D* keeps *X* at every tick whose tick number is a multiple
    of 16 at which its countdown has not run out at its turn, and drops *X* at the first such
    tick at which it has. A run in which no tick whose tick number is a multiple of 16 falls
    after the order's tick and before the tick in which the countdown runs out decides nothing.
  - *Run p3 (the chosen weapon):* a stock vehicle type with two weapons for which S-0304 claim 6
    chooses the second against *X*, with its first weapon replaced by *W* and its second by a
    weapon with 100 % against every class whose damage cannot kill *X* within the run, is ordered
    onto *X* as in run p1. *Expected:* it keeps *X* at every tick whose tick number is a multiple
    of 16 for 100 ticks and fires at it with its second weapon. A run in which *X* is destroyed
    decides nothing.
  - *Run p4 (a rank-4 refusal against a running countdown):* S-0306 scenario P16 part i's
    landed enemy aircraft and test vehicle, without the test building, the vehicle's weapon given
    a rate-of-fire delay of 200 ticks in the test map. The vehicle is ordered to attack the
    aircraft, fires at it once, and the aircraft's owner then orders it up as in that part, to a
    cell of open ground that the vehicle can reach, within the vehicle's range. Record, every
    tick, the aircraft's height and the cell under it, and the tick in which the vehicle's
    countdown runs out. A run
    counts only if the countdown has not run out at the end of the first tick whose tick number
    is a multiple of 16 after the tick in which the aircraft reaches its flying height.
    *Expected:* the vehicle drops the aircraft, while its countdown still runs, in a tick whose
    tick number is a multiple of 16, no later than that tick, as claim 29's phase rule gives
    whether the refusal begins before or after the vehicle's turn. A run that does not count, or
    in which the vehicle misses its turn in a tick of that span (S-0203 claim 10), decides
    nothing.
  - *Run p5 (the drop on *impossible*):* the vehicle *D* of run p2, with its rate-of-fire delay of
    200 ticks, and an enemy vehicle *X* of the computer house that is unarmed, of a type that does
    not cloak by itself and whose armour class is not that of run p1's *X*, so that *H* gives it
    100 % and the firing check refuses it for no versus reason. *X* stands 3 cells from *D*, on
    open ground that *D* can reach, within *W*'s range, in the field of a cloaking building of its
    owner, fully cloaked. A sensor vehicle of *D*'s house
    stands next to *D*, so that *X*'s cell is sensed by *D*'s house, and no other object is within
    *D*'s scan reach. *D* first fires at an armed-and-legal enemy vehicle *Y*, which is then
    removed by the test map's trigger, and is ordered by scripted input to attack *X* while its
    countdown runs. While the sensor is next to it, *X* is shown to *D*'s house, the attack cursor
    appears on it without force-fire, and the order is accepted (S-0304 claim 2 does not apply,
    since *X* is not illegal). Record that *D* holds *X* from the order's tick. The sensor is then
    moved away by scripted input, starting in the tick after the order is accepted, until *X*'s
    cell is no longer sensed by *D*'s house. Record the refusal's tick, the first tick at the end
    of which *X*'s cell is not sensed by *D*'s house (the recorder reads the cell's sensed state
    for that house each tick; *X* is no longer drawn for it from the next frame), and the tick in
    which *D*'s countdown runs out. A run counts only if the refusal's tick is at least 17 ticks
    before *D*'s countdown runs out, so that the first tick after it whose tick number is a
    multiple of 16 comes while the countdown still runs. From then on *X*
    is hidden from *D*'s house, the firing check answers *impossible* for it (S-0516 claim 24,
    rank 4, above the countdown's rank 5), and no search of *D* can pick it again, since table C's
    cloak row rejects it. *Expected:* *D* drops *X* in a tick whose tick number is a multiple of
    16, no later than the first such tick after the refusal's tick, and holds no target at the
    end of that tick or of any later tick of the run; it makes no shot at *X*. Were the hiding no
    refusal, *D* would keep *X* and fire at it once its countdown had run out. The run lasts until
    50 ticks after that countdown has run out. *Falsified if:* *D* still holds *X* at the end of
    the first such tick after the refusal's tick, or makes a shot at *X*. A run in which *D* does
    not hold *X* after the order, or that does not count, decides nothing. A run in which *X* starts
    uncloaking or *D*'s house senses *X*'s cell again before that check tick decides nothing. An
    earlier drop is treated as *Earlier drops* below gives.
  - *Run p6 (the shared phase):* run p1's map with the vehicle *D*, and for *D*'s house a war
    factory and the credits to build vehicles of *D*'s type, whose weapon is *W* in the test map.
    *D*'s house builds such a vehicle *E*. Record the tick *c* in which *E* first appears in the
    recorder's list of objects. Repeat the build until two values of *c* are recorded whose
    remainders when divided by 16 are not 0 and differ from each other. After each such build,
    once *E* has left the factory and stopped, *D* and *E* are ordered together by scripted input
    to attack *X* with force-fire held, the order being applied in a tick whose tick number
    leaves the remainder 1 when divided by 16. *Expected:* *D* and *E* drop *X* in the same tick,
    15 ticks after the order's tick, whose tick number is a multiple of 16. *Falsified if:* *E*
    drops *X* in the tick that a check counted from its creation tick would give (the first tick
    after the order's tick whose tick number leaves *c*'s remainder when divided by 16), or drops
    it earlier in another tick, as *Earlier drops* below gives. A run in which *D* or *E* misses
    its turn in that tick (S-0203 claim 10) decides nothing.
    Then repeat run p6 with run p1's infantry *D* in place of the vehicle, and for *D*'s house a
    barracks and the credits to train infantry of *D*'s type in place of the war factory. The
    house trains such an infantry unit *E*, and the record of *c*, the repeats until two non-zero
    remainders that differ, the order once *E* has left the barracks and stopped, and the
    *Expected:* are as above. *Falsified if:* the infantry *E* drops *X* in the tick that a check
    counted from its creation tick would give, or drops it earlier in another tick, as *Earlier
    drops* below gives. A run in which either misses its turn in that tick decides nothing.
  - *Run p7 (a refusal begun by the holder's own shot):* an infantry *D* of a stock type
    whose default weapon is a healing weapon (S-0310, Terms), and a damaged infantry unit *F* of
    *D*'s house next to it on open ground. The test map gives the healing weapon a rate-of-fire
    delay of 200 ticks, a healing amount larger than *F*'s missing health and no railgun flag, so
    that *D*'s first shot brings *F* to full health and its heal lands in the projectile's turn.
    *D* is ordered by scripted input to heal *F*, the order
    applied in a tick whose tick number leaves the remainder 15 when divided by 16 (series B) or
    the remainder 7 (series A), three runs each. Record every tick, with tick numbers: *D*'s target
    and shots, and *F*'s health. A run counts only if *D*'s healing shot is made in its turn of the
    tick after the order's tick, *F* is below full health at the end of the order's tick and at
    full health at the end of the shot tick (the tick of that turn), so that the refusal begins in
    the shot tick.
    Each series is repeated until it has three runs that count. A full-health infantry target
    gives an infantry healer *illegal* whatever its rearm countdown shows (S-0310 claim 11), so
    from the shot onwards the check refuses *F*. *Expected:* in series B, whose shot tick is a
    check tick, *D* still holds *F* at the end of the shot tick: the heal lands in the projectile's
    turn, after *D*'s (S-0203 claims 4 and 5), so the refusal begins after that turn's check. In
    both series *D* drops *F* later, no later than the first check tick after
    the shot tick (16 ticks after it in series B), whether by its own reply (S-0310 claim 11) or
    by the check. *Falsified if:* in a series B run *D* holds no target at the end of the shot
    tick while in every series A run it still holds *F* at the end of its shot tick. A series A run
    in which *D* holds no target at the end of the shot tick shows that another rule drops *F* in
    that turn, and run p7 then decides nothing. A run in which *D* misses its turn in the shot tick
    (S-0203 claim 10) decides nothing. *Earlier drops* below does not apply to this run, whose
    target is of the holder's own house. The records are the same whether the check comes before
    or after *D*'s firing step, so the run does not decide that place.
  - *Run p8 (other behaviours):* part m's firer, aircraft *X* and vehicle *Y*, the test map
    giving the firer's weapon a range of at least 10 cells, and both the normal and the
    area-guard targeting delay (S-0215 claim 33) a value of 200 ticks. *X* stands landed on open
    ground within the firer's range, as in part m, so that the firer, whose weapons are
    anti-ground only, can pick it. Series A: the firer is given an area-guard order
    by scripted input on its own cell, and picks *X* by itself after it, with *area guard*
    current. Series B: the firer's type also has the opportunity-fire flag (claim 3), and the
    firer is ordered by scripted input to move to a cell of open ground 8 cells away, along a
    straight path on which it stays within its range of *X*'s ground cell and of the cell where
    *X* will stay in the air. The move order leaves the firer with no target, and it picks *X* by
    its per-tick search while it moves. In both series *X* is then ordered up as in part m. Record,
    every tick, with tick numbers: the firer's target, current behaviour
    and searches, and *X*'s height. A run counts only if, at the end of the tick before the first
    check tick after the tick in which *X* reaches its flying height, the firer holds *X* with
    *area guard* (series A) or *move* (series B) current, and the firer makes no search from the
    tick in which *X* reaches its flying height to the end of that check tick. Each series is
    repeated until it has three runs that count, or for at most ten runs, after which a series
    with no run that counts decides nothing (for example because the firer does not pick *X*
    in *move*) and the record says why. A drop of *X* before that check tick decides nothing,
    since the processing of these behaviours is not shown here to keep such a target.
    *Expected:* in a run that counts, the firer drops *X* in its
    turn of that check tick. Were the behaviour to make no check, the firer would still hold *X*
    at the end of that tick. *Falsified if:* in a run that counts, the firer still holds *X* at
    the end of that check tick. A run in which the firer misses its turn in that tick decides
    nothing.
  - *Run p9 (owners and targets):*
    - *p9a (a holder of a computer-controlled owner):* run p8's series A with the firer of the
      computer player's house and *X* of the human's house, and no *Y*. The firer is put on the
      map by its map entry with *area guard* as its initial behaviour, as scenario A11 places its
      actors, and is given no order: it picks *X* by itself. No other object of either house
      stands within the firer's range. The human orders *X* up by scripted input, as in part m.
      Record what run p8 records. A run counts under run p8's condition for series A, and its
      *Expected* and *Falsified if* are run p8's. A run in which the firer's behaviour is not
      *area guard* at the end of a tick from its pick of *X* to the end of that check tick, or
      in which it holds a target other than *X* in that span, or in which another object of the
      computer player's acts on the firer or on *X*, decides nothing (scenario A11's rule for
      computer activity). So does one in which the firer misses its turn in that check tick.
    - *p9b (an allied target, claim 25 part k):* as p9a, but in place of *X* a vehicle *Z* of
      the firer's own house, of a type against whose armour class *H* gives 100 %, stands 3 cells
      from it, and no other object of either house stands within its range. The firer's type
      has part m's attack-friendlies flag, as in run p8. Control: the same with the firer
      and *Z* of the human's house, the firer put in *area guard* by its map entry and given *Z*
      by an attack order on *Z*, given with force-fire held by scripted input. Call *n* the tick
      at whose end the firer first holds *Z*: in the test, the tick of its own pick, and in the
      control, the tick in which the order is carried out. In place of p9a's counting condition,
      a run, test or control, counts only if the tick number of tick *n* + 1 is not a multiple
      of 16 and the firer makes no search in tick *n* + 1.
      *Expected:* the computer-controlled firer holds no target at the end of tick *n* + 1,
      having dropped *Z* in its first turn after its pick, and makes no shot at *Z* in tick
      *n* + 1. The control's firer holds *Z* at the end of tick *n* + 1. *Falsified if:* in a run
      that counts, the computer-controlled firer still holds *Z* at the end of tick *n* + 1, or
      makes a shot at *Z* in tick *n* + 1. A shot in tick *n* is recorded and judges nothing.
      After tick *n* + 1 the run controls no search of the firer: a later pick of *Z* by its own
      search, and a shot at *Z* in that pick's tick, are recorded with their ticks and judge
      nothing, as the shot in tick *n* does. A control run that counts, in which the firer does
      not hold *Z* at the end of tick *n* + 1, falsifies part k's keep by a holder of a
      human-controlled owner. A control run whose firer never holds *Z* after the order decides
      nothing and is recorded. A run in which the computer-controlled firer does not hold *Z*
      within 300 ticks decides nothing: whether its search takes an object of its own house is
      not settled here. A test run also decides nothing in which, from the firer's pick of *Z*
      to the end of tick *n* + 1, the firer's behaviour is not *area guard* at the end of a tick,
      it holds a target other than *Z*, or another object of the computer player's acts on the
      firer or on *Z* (scenario A11's rule for computer activity). In test and
      control alike, a run also decides nothing when, in tick *n* + 1, the firer misses its turn
      or leaves play, *Z* leaves play, is destroyed or is hidden from the firer's house, or the
      firer is given any input (in the control, any after the order). Each of these is read from
      the records, not from the firer's target.
  - *Run p10 (a materialising holder):* a test vehicle type *T* of the human's house:
    a stock vehicle type given the teleporting locomotor (S-0207) and, in place of its weapons,
    run p4's vehicle weapon (anti-ground only) with a rate-of-fire delay of 200 ticks and a range
    of at most 10 cells, both set by the test data. The test map makes the phase-in after a
    teleport last at least 48 ticks. Record every tick, with tick
    numbers: *T*'s target, cell and shots, and whether *T* is drawn phasing in.
    - *p10a (no beam):* an aircraft *A* of run p4's type, of the human's own house, landed on
      open ground 15 cells from *T*, beyond *T*'s range. The human orders *T* by scripted input,
      with force-fire held, to attack *A* (S-0214 claim 6, part (a)), and *T* teleports toward it.
      In the tick in which *T* is first drawn phasing in, the human orders *A* up by scripted
      input, as part m orders its aircraft up, to a cell of open ground within *T*'s range of
      *T*'s new cell. Both orders go to the human's own objects, as the committed records' scripted
      orders do. *A* is of *T*'s house, so no hit gives either a retaliation (claim 18), and
      claim 25 part k's allied drop does not apply to a human-controlled holder.
      Record
      also *A*'s height and the cell under it. A run counts only if *A* reaches its flying
      height at least 17 ticks before the last tick in which *T* is drawn phasing in, stays in
      the air within *T*'s range until the first check tick after that last tick, and that check
      tick comes at least 3 ticks after that last tick, so that a drawing that lags or leads the
      phase-in by a tick does not decide. From the
      tick in which *A* reaches its flying height the firing check refuses *A* (S-0306 claim 23,
      rank 4). *Expected:* *T* holds *A* at the end of every check tick in which it is still
      drawn phasing in, and drops *A* at the first check tick after the last tick in which it is
      drawn phasing in. Were the refusal to come before the materialising *rearming*, *T* would
      drop *A* at the first check tick after *A* reaches its flying height, while *T* still
      phases in. *Falsified if:* *T* holds no target at the end of a check tick in which it is
      drawn phasing in and *A* is in the air, or still holds *A* at the end of the first check
      tick after the last tick in which it is drawn phasing in. In p10a and p10b, a run in which
      *T* fires at *A* before it teleports, or does not teleport, decides nothing. A run in which
      *A* moves other than
      by the scripted orders, in which *T* misses its turn in a check tick of that span, or in which
      an object of the computer player's acts on *T* or *A* (scenario A11's rule for computer
      activity), decides nothing. A turn that makes no check while *T* phases in gives the same
      records, so this run measures the keep and its end, not that the keep is the check's
      *rearming* (claim 29, backing).
    - *p10b (a temporal beam locked):* as p10a, but *A* is run p4's landed enemy aircraft,
      the computer player's, on open ground 15 cells from *T*, beyond *T*'s range, which stays
      landed and is given no order, *T* is
      ordered to attack it without force-fire, and
      an infantry unit *C* of *T*'s own house, of a stock type whose weapon is a temporal weapon
      (S-0514), stands within its range of *T*'s arrival cell. In the tick in which *T* is first
      drawn phasing in, the human orders *C* by scripted input, with force-fire held, to attack
      *T* (S-0214 claim 6, part (a)). A lock has no owner condition (S-0514 claims 1 to 4).
      Record also the first tick at the end of which *C*'s beam is drawn on *T* (the lock's
      tick), and *T*'s cell each tick. A run counts only if the lock's tick is at least 2 ticks
      before the last tick in which *T* is drawn phasing in, and neither the lock's tick nor the
      tick after it is a check tick. *Expected:* *T* drops *A* no later than the end of the tick
      after the lock's tick, in its first turn after the lock, and holds no target at the end of
      any later tick while the beam is drawn on it. No rule gives *T* a target in those ticks:
      each of its turns ends at its drop (claim 29), *C* is allied with it, so *C*'s hits give
      no retaliation (claim 18), and the run gives *T* no order. *Falsified if:* in a run that
      counts, *T* still holds *A* at the end of the tick after the lock's tick, or holds a target
      at the end of a later tick while the beam is drawn on it. A run in which *C* never locks on
      *T* while *T* phases in decides nothing, and so does one in which *C* moves or takes
      another target, one in which *A*'s house moves *A*, and one in which an object of the
      computer player's acts on *T*, *A* or *C* (scenario A11's rule for computer activity). A
      run in which *T* leaves its cell before the end of the tick after the lock's tick decides
      nothing, since a scatter request (S-0215 claim 19) would then explain the drop of *A*. A
      later move is recorded and judges nothing. So does a run in which *T* misses its turn
      (S-0203 claim 10) in the lock's tick or the tick after it, whether or not that tick is a
      check tick.
  - *Earlier drops.* In runs p1 to p6 and p10 the holder is not berserk, has a weapon that does
    damage, and its target is an enemy on open ground that it can reach, or, in run p4, an
    aircraft above such ground, and in run p10a an aircraft of its own house above such ground,
    held by force-fire. It is in *attack* and human-controlled in all of them. No
    start-of-turn drop other than claim 29's reaches such a holder in *attack* and its target
    (medium, `measurable` by runs p1 to p6 and p10a). So in those runs a drop of the
    target in a tick whose tick number is not a multiple of 16, no later than the tick by which the
    run expects the drop (in run p3, in any tick of the run), falsifies claim 29's cadence. In
    runs p8 and p9a, whose holders are in other behaviours (*area guard*, *move*), an earlier drop
    decides nothing and is recorded with its tick, as in part m, since S-0215's processing of those
    behaviours is not shown here to keep such a target. The exceptions are run p4 while the cell
    under the aircraft is one the vehicle cannot reach, such as the pad it lifts off from, and run
    p10a in the same case: there the drop for a target the holder can neither reach nor hit from
    where it stands may apply (Open questions, *Other drops at the start of a turn*), so a drop then
    is recorded with its tick and decides nothing. Runs p7, p9b and p10b expect drops of their own
    and do not use this rule.
  - *Falsified if:* *D* fires at *X* in run p1, or a holder still holds its target at the end of
    the tick in which the run expects the drop, or run p2's *D* drops *X* in a tick whose tick
    number is a multiple of 16 while its countdown runs. In run p3 a drop of *X* in such a tick
    falsifies it, and in a run p1 that decides the absence of a timer draw, a draw over 4 to 8
    ticks in *D*'s turn of the drop tick or a reset of its targeting timer there. Runs p5 to p10
    have their own falsifiers above.
- *Falsified if:* any part differs.

**A7: threat preference and distributed fire (claims 13, 17, 20).**
- *Part a:* two targets at the same distance, one of a type with a high special threat value in the
  test map. With a positive special-threat coefficient the high-value one is picked, with a negative
  one the other.
- *Part b (distance term):* a firer with a guard range of 20 cells and a weapon range of 1 cell
  (test map: *n* = 20, so *r*₁ = 5 and both targets lie within the first early-stop band; *k* = 1),
  and two identical targets on the firer's row at 2 and 5 cells (rings 2 and 5, *d* = 2 and 5, so
  *e* = 1 and 4), with the target recorder. The firer type's distance coefficient is −1, then +1,
  and its other coefficients are equal for both targets (test map). *Expected:* with −1 the
  recorder shows the near target as the first target picked, with +1 the far one (the far target
  lies beyond the weapon's reach, so the observable is the target picked, not a shot).
  *Falsified if:* the far target is picked with −1, or the near one with +1.
- *Part c:* a distributed-fire vehicle type in *guard* facing three targets: successive shots go
  to three different targets before any is shot twice. Run 2 (the condition on *attack*): the same
  vehicle ordered to attack one of the three targets: every shot goes to that target. Run 3 (a
  building): a defensive building of a type given the distributed-fire flag in the test map, with
  the same three targets in range, ordered to attack one of them: its successive shots still go
  to three different targets before any is shot twice (a building adds to its fired-at list in
  any behaviour). *Falsified if:* run 1 or run 3 repeats a target before all three are hit, or
  run 2 shoots a target other than the ordered one.
- *Part d (map-wide, claim 20):* the same distributed-fire type ordered to hunt with the three
  targets far away. *Expected:* it never picks a target by itself (with
  `quirks.hunt_ignores_distributed_fire` on); an ordinary type with the same weapon in the same
  map picks one at once.
- *Part e (sign of the danger term, claim 13, F4):* two targets of the same type at equal distance
  with weapons that can hit the firer, one ordered to attack the firer and the other ordered to
  attack another unit of the firer's house; the firer's target-effectiveness coefficient
  positive, then negative in the test map, and its other coefficients zero as far as the data
  format allows. *Expected:* with the positive coefficient the target that is not aiming at the firer
  is picked, with the negative one the target aiming at it. *Falsified if:* the preferences are the other way round.
- *Part f (zero score, claim 13):* a firer and one target whose coefficients and versus values the
  test map sets, by evaluating F4, so that the converted score is exactly 0; then so that it is
  −1. *Expected:* with `quirks.zero_score_rejected` on, the target at score 0 is never picked and
  the target at −1 is (its score becomes 1). *Falsified if:* the target at 0 is picked or the one
  at −1 is not.
- *Part g (map-wide distance units, F4, `quirks.hunt_distance_in_leptons`):* a hunting vehicle with
  two ground targets, *Y* 20 cells away and *X* 30 cells away. The test map sets the coefficients
  so that every term but two is equal for both: the distance coefficient is −1 and the special-threat
  term favours *X* by 100 score units. *Expected* with the switch on: *Y* (the distance term then
  favours *Y* by 2560 units); with the switch off: *X* (it favours *Y* by only 10 units).
  *Falsified if:* *X* is picked with the switch on.
- *Part h (order of the modifiers, claim 13):* a firer with target-health scan mode *normal* and a
  suppressing weapon in slot 0, and a fire-suppression distance of 3 cells in the test map (*N* = 3,
  so rings 1 and 2 around a target are counted). Target *X* at 40 % health has one building of the
  firer's house with a 1 × 1 foundation within its rings 1 and 2; it stands in a cell that comes
  earlier in F2's order than the cell of a second target *Y* of the same type at full health, with
  no building of the firer's house within 2 rings of it. The test map sets the firer type's
  strength and distance coefficients to 0, so that both targets have the same converted score *s*,
  and the other coefficients so that *s* is odd. *Expected:* *X* scores *s* (doubled, then halved) and wins the tie by scan order, so
  *X* is picked; under the opposite order *X* would score *s* − 1 and *Y* would be picked.
  *Falsified if:* *Y* is picked.
- *Falsified if:* the preferences are reversed, part c repeats a target before all three are hit,
  or the distributed-fire type in part d picks a target.

**A8: weak-target preference and expected health (claims 14, 15).**
- *Part a:* firer type with target-health scan mode *normal* and strength and distance
  coefficients 0 (test map), so that only F5 separates two targets of the same type; the one at
  40 % health stands later in the scan order than the one at full health. *Expected:* mode
  *normal*: the damaged one is picked; mode *none*: the one earlier in the scan order. Run 2 (a
  value above the health becomes the health, claim 15, F7): the same, but the damaged target starts
  at full health and is brought to 40 % before the firer arrives by area damage from a unit of the
  human house that force-fires at a neighbouring cell (the target is not shot at, so no expected
  damage is subtracted from it, and its expected remaining health stays at its strength until its
  own turn brings it down to its health). *Expected* in mode *normal*: the damaged one is picked.
  *Falsified if:* run 2 picks the target at full health.
- *Part b:* mode *strong*, two firers and one target that one shot kills: the second firer does
  not pick the target while the first firer's shot is in flight, and picks it again once the
  expected health has recovered (about 2 × the shortfall ticks later) if the target survived.
- *Part c (same tick, claim 4):* as part b, with the tick recorder: in runs where the targeting
  timers of the two firers run out on the same tick (5 runs, or more until 3 such runs are seen),
  the firer that searches second does not pick the target.
- *Part d (inaccurate projectile, claim 4):* as part b with the firers' projectile having the
  inaccurate flag. *Expected:* the second firer picks the target at once, because no expected
  damage was subtracted.
- *Part e (distributed fire, claim 4):* as part b with firers of a distributed-fire type.
  *Expected:* the second firer picks the target at once (a distributed-fire pick lowers no
  expected remaining health).
- *Part f (repairs and other healing, claim 15):* a firer of the human house whose type has the
  attack-friendlies flag and target-health scan mode *normal*, with strength and distance
  coefficients 0 and a guard range of 2 cells (test map: *n* = 2), and no object other than *X* and
  *Y* within 3 cells of its final position when it arrives; two vehicles *X* and *Y* of the human
  house and of the same type, with their weapons removed and a strength field of at least 2000 (test map), in ring 1 of the firer's
  final position, *Y*'s cell before *X*'s in F2's order. *Y* is at full health throughout. The
  firer starts out of range and is moved into place (opportunity-fire flag off, so it searches
  only after it arrives). Runs, each from a fresh game with *X* placed at 30 % health:
  - run 1: *X* is healed to at least 90 % by a healing vehicle of the human house. *Expected:* *X*
    is picked (its expected remaining health stayed at about 30 %, so its score is doubled),
  - run 2: *X* is serviced by a repair building of the human house up to about 70 % and then moved
    off. *Expected:* *Y* is picked (the repair steps raised the expected value too, so there is no
    doubling, and *Y* wins the tie),
  - run 3: *X* is healed to at least 90 % as in run 1, then serviced by the repair building up to
    full health. *Expected:* *Y* is picked (the full repair set the expected value to the strength
    field).
  Timing of run 1. By F7 the expected value starts to recover, by 1 point on 4 ticks of every 8,
  as soon as the first healing step raises *X*'s health above it, and it needs at least
  2 × (trunc(*S* / 2) − 0.3 × *S*) ≈ 0.4 × *S* ≥ 800 ticks (*S* the strength field) to climb from
  30 % to the doubling threshold of F5. The firer's first search after it arrives must therefore
  fall within 400 ticks of the first healing step. To that end the test map gives the healing
  vehicle's weapon a healing amount per shot and a rate of fire that bring *X* from 30 % to 90 %
  within 200 ticks, sets the targeting delays (S-0215, F6) below 100 ticks, and the firer is moved
  so that it arrives as the healing ends; a run whose first search falls later is discarded and
  repeated. In runs 2 and 3 the timing does not matter: a repair step raises the expected value
  with the health, and F7 never lifts it above the health.
  *Falsified if:* run 2 or 3 picks *X*. If run 1 picks *Y*, the statement that other healing
  leaves the expected value unchanged is wrong.
- *Part g (halving at 0 or less, claim 14, F5):* as part b, but both firers have target-health
  scan mode *normal*, and a second target *T*₂ of the same type at full health stands at the same
  distance from the second firer as the first target *T*₁, later than *T*₁ in its scan order, with
  strength and distance coefficients 0, and a slow projectile for the first firer (test map), so
  that its shot is in flight for many ticks. The second firer searches while the first
  firer's pick of *T*₁ has brought *T*₁'s expected remaining health to 0 or less (*T*₁ at a health
  that one shot kills). *Expected:* the second firer picks *T*₂ (*T*₁'s score is halved). Under
  the reading in which a value of 0 or less counts as "at most half the strength", *T*₁ would be
  doubled and picked. *Falsified if:* the second firer picks *T*₁ in a run in which its search
  falls before *T*₁'s death.
- *Part h (floor and recovery, claim 15, F7):* one immobile firer, a defensive building of the
  human house with target-health scan mode *strong*, a guard range of 10 cells and a weapon range
  of 3 cells (test map), so that it can pick a target it cannot reach; one enemy vehicle *T* alone,
  6 cells away, at a health *h* such that the expected damage *E* of the firer's weapon against
  it (S-0302, claim 18) is at least *h* + 100; targeting delays (S-0215, F6) set in the test map
  so that the longest timer is *D* ticks, with *D* at most 30. Target recorder and tick recorder.
  *Expected:* the firer picks *T* (tick *t*₀); at a later search it drops *T*, which is out of
  range (claim 4), and does not pick it again while *T*'s expected remaining health is 0 or less.
  It picks *T* again at a tick between *t*₀ + 58 and *t*₀ + 63 + *D*. From a value of −30 or less
  the expected remaining health needs 31 recovery steps of F7 to reach 1; the first step falls on
  one of the ticks *t*₀ to *t*₀ + 5, depending on the tick number and on the order of the two
  objects' turns, and the 31st comes 58 or 62 ticks after the first, depending on where in its
  block of four ticks the first step falls. Without the floor of −30 at least
  101 steps would be needed, and the re-pick would come at least 200 ticks after *t*₀.
  *Falsified if:* the firer picks *T* again before *t*₀ + 58, or not by *t*₀ + 63 + *D*.
- *Falsified if:* part a ignores the health, part b shows the second firer committing at once, part
  c shows it committing, part d shows it refusing, part e shows it refusing, or part f differs as
  it says.

**A9: enemy house bonus and anger (claims 13, 16).**
- *Setup (all parts):* a human firer's house *H* and two computer houses *P* and *Q*, none allied.
  Two targets of the same type, one of *P* and one of *Q*, at equal distances from the firer, with
  the firer type's distance and strength coefficients 0 and the enemy-house bonus *B* large
  (test map), so that the bonus decides between them. The firer is kept from searching (out of
  range, then moved into place with the opportunity-fire flag off) until the moment named.
- *Part a (increase):* *Q* before *P* in the house order (Terms: *Q* in an earlier lobby slot than
  *P*), and *Q*'s target first in the scan order.
  *P* alone attacks units of *H* repeatedly. Two runs: in the first the firer is moved into place
  before any attack, in the second after the attacks. *Expected:* the first run picks *Q*'s target
  (whether or not the set-up made the houses enemies, see part b); the second picks *P*'s target. *Falsified if:* *Q*'s target is picked after the attacks.
- *Part b (record only):* as part a, but with *P*'s target first in the scan order, and a search
  before any attack. *P*'s target means no enemy house at the start; *Q*'s target means the set-up
  made *H* an enemy of both houses (anger 1 each, Open questions). Either result is consistent
  with claim 16.
- *Part c (a defeated enemy house stays the enemy house):* four houses, *H* (human) and the
  computer houses *P*, *Q* and *R*, none allied. Anger is raised by placement, not by the AI's
  choice: the person moves a weaponless unit of *H* next to an armed unit of *Q* (which attacks it
  by itself) for a single hit and then away, and afterwards next to an armed unit of *P*, whose
  weapon the test map makes much stronger, for several hits, so that *H*'s anger towards *P* is
  above its anger towards *Q*, which is above 0. The person then destroys every object of *P*,
  so that *P* is defeated. No object of *R* ever damages an object of *H* (*R*'s objects have
  their weapons removed in the test map), so *H*'s anger towards *R* stays 0. Targets of the same
  type, one of *Q* and one of *R*, stand at equal distances from the firers' final position, *R*'s
  first in the scan order, and *Q*'s armed unit stands outside both firers' search radius.
  *Expected:* a firer moved into place after *P*'s defeat picks *R*'s target (the enemy house is
  still *P*, which owns nothing, so no target gets the bonus and the tie goes by scan order); it
  is moved away again as soon as the target recorder shows its pick, before it can damage the
  target. Then the weaponless unit of *H* is moved next to *Q*'s unit for one more hit; a second
  firer, of the same type and moved to the same final position, picks *Q*'s target (that change
  of anger made *Q* the enemy house). *Falsified if:* the first firer picks *Q*'s target, or the
  second picks *R*'s.
- *Part d (tie rule, and *C* independent of the attacker's owner):* *Q* before *P* in the house
  order (*Q* in an earlier lobby slot than *P*); *P* and *Q* are computer houses at the same difficulty, so that in the original their
  units' damage factors are the same (the original gives a computer house the row of its lobby
  difficulty, S-0302 claim 3; our engine gives every house the normal row). The armed units
  of *P* and *Q* are of the same type, with
  the same weapon and veterancy. Anger is raised by placement as in part c: the person moves a
  weaponless unit *V* of *H* (strength at least three times one hit) next to *P*'s armed unit for
  a single hit, then away, and then next to *Q*'s armed unit for a single hit, then away, with the
  health recorder on *V*. A run in which *V* takes more than one hit from either unit, or two hits
  of different damage, is discarded and repeated. *P*'s and *Q*'s armed units stand outside the
  firer's search radius, so that the only candidates are the two targets. Then the firer is moved
  into place, with *P*'s target first in the scan order. *Expected:* the firer picks *Q*'s target: the two hits gave
  equal anger towards *P* and *Q* (the same *d*, *S* and *C*), and the tie goes to *Q*, the earlier
  house. An equal starting anger from the set-up (part b) adds the same amount to both and does
  not change this. *Falsified if:* the firer picks *P*'s target (that would show either that a
  tie keeps the present enemy house or goes to the later house, or that the two increments
  differed, for example because *C* depends on the attacker's owner).

- *Part e (anger changes when alliances change):* use houses *H* (human), *P* and *Q* (computer), two candidate objects of the same type owned respectively by *P* and *Q*, and a human searcher whose enemy-house bonus is positive and large enough to decide between equal-score candidates. Arrange the scan order so that *Q*'s object is first. Set the alliance-order settings required by S-0414, and ensure the proposed orders satisfy its conditions. In the first run, *H* and *P* start allied, *Q* is not allied with *H*, and neither anger value is positive; the person issues the alliance-breaking order from *H* to *P*. *Expected:* the anger increment when *P* becomes an enemy makes *P* the enemy house, so the searcher picks *P*'s object despite *Q*'s earlier scan position. In a separate fresh run, *P* and *Q* start as enemies of *H*; damage raises *H*'s anger towards *P* above 0, while anger towards *Q* stays 0. The person then issues a permitted alliance order from *H* to *P*, with *Q* remaining an enemy. *Expected:* *H*'s anger towards the new ally *P* becomes 0 and *P* ceases to be the enemy house; the searcher picks *Q*'s object by scan order. *Falsified if:* either pick contradicts the stated enemy-house bonus and tie outcome. A run in which S-0414 refuses the alliance order does not decide the corresponding rule.

**A10: suppression (claim 14).**
- *Setup (all parts):* a firer of the human house, of a type that is not a multi-weapon type, with
  a suppressing weapon in slot 0, target-health scan mode *none*, strength and distance
  coefficients 0, and a guard range of 8 cells (test map: *n* = 8, early stops after rings 2 and
  4); a fire-suppression distance of 3 cells (*N* = 3, so rings 1 and 2 around a target are
  counted). With the firer's cell as (0, 0), two enemy vehicles *X* and *Y* of the same type, at
  full health and with their weapons removed, stand in ring 2: *X* at (−2, −2), the first cell of
  ring 2 in F2's order, and *Y* at (2, 2), later in that order. Without suppression they have
  equal scores, and *X* wins the tie (claim 17). Buildings named below belong to the human house,
  have no weapon, and stand where no other building does; every other cell within 2 cells of *X*
  or *Y* holds no building.
- *Main part (one building cell):* one building with a 1 × 1 foundation at (−3, −3), within 2
  cells of *X* and more than 2 cells from *Y*. *Expected:* *Y* is picked (*X*'s score is halved).
  Control: the same without the building: *X* is picked. *Falsified if:* *X* is picked with the
  building present, or *Y* in the control.
- *Part b (a building counted once per cell):* one building with a 2 × 2 foundation covering
  (−4, −4) to (−3, −3), all four cells within 2 cells of *X*; and two buildings with a 1 × 1
  foundation at (3, 3) and (4, 4), within 2 cells of *Y*. No building cell is within 2 cells of
  both. *Expected:* *Y* is picked: *X* has *m* = 4 (score divided by 16) and *Y* has *m* = 2
  (divided by 4). If a building were counted once, whatever its size, *X* would have *m* = 1 and
  be picked. *Falsified if:* *X* is picked.
- *Part c (switching weapon number):* repeat the main part with a gunner switching vehicle
  whose numbered slots 0 and 2 differ only in the suppress flag: slot 0 off, slot 2 on.
  Give its first passenger mode index 2 before either target is within range, then move the
  vehicle into the setup with opportunity fire off. Record its weapon number and first target.
  *Expected:* number 2 and *Y* picked; with the passenger absent, number 0 and *X* picked.
  *Falsified if:* a qualifying number-2 search picks *X*, or the number-0 control picks *Y*.
- *Part d (gattling current weapon is not candidate-specific):* named test map data gives a
  gattling vehicle a positive turret count and two populated stage-0 slots, both able to hit
  ground and air, with identical versus values and damage. Its slot 0 lacks suppression and
  slot 1 has it. The vehicle has weapon number 0, stage 0, no passenger and no earlier firing.
  All threat coefficients and the enemy-house bonus are explicitly 0, target-health scan mode
  *none*, guard range 8 cells and fire-suppression distance 3 cells. Enemy *X* is a hovering
  aircraft in a tracked sector reached by the search, over cell (4, 0); enemy *Y* is a vehicle
  in ring 1. Both have no weapon. A human-owned weaponless 1 × 1 building at (5, 0) suppresses
  only *X*; no other building lies within 2 cells of either. Introduce both targets before the
  vehicle's first search and record its weapon number, stage and first target before a shot.
  *Expected:* *X* wins the equal-score tie, since the current slot 0 has no suppress flag,
  although the firing weapon chosen against *X* is slot 1. Control: exchange the two suppress
  flags without changing anything else. *Expected:* *Y*, because current slot 0 now suppresses
  *X*. Candidate-specific suppression predicts the opposite pair. *Falsified if:* either
  qualifying pick differs; a search after firing or a stage or number change does not decide.
  Parts c and d use game speed 6, no engine add-on and no mods (the named test files of a scenario excepted),
  the common bystander void rule, and three fresh runs per setting.

**A11: map-wide search (claims 1, 7, 11, 18, 22).**
Use tiny purpose-made skirmish maps, game speed 6 and the common three fresh runs per setting.
Place actors with their named initial behaviours in the map; no player order to a computer-owned
actor is assumed. Use the read-only target and tick recorders, adding current behaviour,
processing and targeting timers, position and render layer for parts j and k. Keep unrelated
computer activity outside the observed area; void a run if it acts on a test object. For a
predicted absence of acquisition in j or k, observe 300 ticks after the first qualifying search,
with repeated search opportunities and the stated conditions maintained. No qualifying search
makes a run inconclusive. A qualifying pick different from the prediction is a falsifier, not
an excluded run. Named test data gives targets enough health to survive through the observation.
- *Part a:* a unit ordered to hunt on a map with targets far away: it moves towards a target out of
  sight range.
- *Part b:* an infantry unit hunting, with the only enemy on an island it cannot reach: it does not
  pick it, and stays idle. A hovering or amphibious unit picks it.
- *Part c (claims 7 and 22):* a human-owned vehicle with the driving locomotor in *hunt* (guard
  range 0 in the test map, so the radius of a range search comes from the weapon's reach, F1)
  approaching its only enemy target, with the target recorder, its position, destination and
  current behaviour. Runs:
  - (c1) a route search fails while the vehicle is within 1 cell of its destination cell and
    the target is not within its weapon's reach, for example because a map trigger closes the
    destination cell. *Expected:* at that failure the vehicle drops the target and stays in
    *hunt*; its next search is a range search covering only its weapon's reach, which finds
    nothing, so the recorder shows no target after it; the search after that is a map-wide
    search again (the empty search cleared the mark) and the recorder shows the far target again.
  - (c2) as c1, with a second enemy placed within the weapon's reach of the place where the
    vehicle stops, its health low enough for the vehicle to destroy it. *Expected:* the first
    search after the drop picks the second enemy; after it has left play the next search finds
    nothing, and only a later search picks the far target again.
  - (c3) as c1, but the passage is closed while the vehicle is more than 2 cells from its
    destination, so that the failure gives the move up (S-0212 claim 23, row 2). *Expected:* the
    vehicle drops the target at that failure and queues *guard*, which becomes current at a
    permitted start (S-0215 claim 11), on the failure tick or later; after that the vehicle
    never shows *hunt* again.
  A run of c1 or c2 decides nothing if, at the failure, the cell ahead of the vehicle holds a
  unit or building of an allied house (S-0207 claim 15's give-up can then apply), or if its
  destination is not a cell. *Falsified if:* in c1 or c2 the vehicle keeps the target when the
  route search fails, in
  c1 its first search after the drop picks the far target or no later search ever picks it
  again, in c2 the search right after the second enemy leaves play picks the far target, or in
  c3 the vehicle keeps the target, *guard* never becomes current, or a behaviour other than
  *guard* follows *hunt*.
- *Part d (area search skips other zones, claim 11):* an infantry unit placed with *area guard*, a
  weapon of range 8 cells, and an enemy on an island across water 5 cells away, inside both the area
  radius and the weapon range. *Expected:* the unit never picks it.
- *Part e (range search does not, claim 11):* the same unit in *guard* with the same enemy.
  *Expected:* it picks and fires at it.
- *Part f (ground layer only, claim 7):* a vehicle whose weapon is both anti-air and anti-ground,
  ordered to hunt, on a map whose only enemy is an infantry unit with jumpjet movement, kept
  hovering far from the hunter (out of its sight), with the target recorder. *Expected:* the hunter
  picks nothing while the enemy hovers; once the enemy lands, it picks it. *Falsified if:* it
  picks the hovering enemy.
- *Part g (aircraft pass first, claim 7):* a hunter as in part f, with distance coefficient 0
  (test map) and a warhead whose versus values are the same against the armour classes of both
  targets; two enemies with their weapons removed, at full health and of types with the same
  special threat value: an aircraft landed on open ground and a vehicle. *Expected:* the aircraft
  is picked in every run (equal scores, and the aircraft pass comes first). *Falsified if:* the
  vehicle is picked in any run.
- *Part h (landed aircraft, second-pass distance):* use the common measurement setup on
  flat connected ground. A hunting vehicle has an anti-ground weapon without anti-air capability,
  and two enemies outside its weapon reach: a landed aircraft on open ground and a vehicle.
  Set test coefficients and the targets' special threat values so that F4 gives the vehicle
  a score strictly between the aircraft's hypothetical cell-distance score and its lepton-distance
  score; use a positive distance coefficient. All other score terms and modifiers are controlled,
  and both targets pass table C. Record the first chosen target before either moves.
  *Expected:* the aircraft wins, using leptons in the second pass, even without an aircraft class.
  Repeat with an anti-air-capable weapon as well: the aircraft still wins because its larger
  second-pass score competes after its smaller first-pass score. *Falsified if:* the vehicle wins
  either run, or the ground-only hunter finds no aircraft when it is the sole target.
- *Part i (landed aircraft, first pass retained):* repeat part h with a negative distance
  coefficient and test threat values giving the vehicle a score strictly between the aircraft's
  two positive scores. *Expected:* the anti-air hunter picks the aircraft by its cell-distance
  first-pass score, while the ground-only hunter picks the vehicle by its lepton-distance score.
  *Falsified if:* either pick differs. Together h and i distinguish two-pass scoring from either
  a single-pass rule or a rule using cells in both passes. Repeat both with the aircraft on a pad,
  recording its render layer and connected region to exclude layer or zone confounds.
- *Part j (cells of no reach region, claims 7, 11 and 18; Q-1502):* a test map with a cliff edge
  whose upper and lower cells the infantry and ground vehicle movement zones cannot use (S-0212,
  recorded as cells of no reach region), with computer-controlled test houses and the target
  recorder. Runs: (1) an enemy infantry unit placed on such a cell, unarmed, and a hunting armed
  vehicle on open ground 6 cells away. *Expected:* the vehicle never picks it and the infantry unit
  never leaves its cell. (2) As run 1, with the vehicle in *guard* and the infantry unit within its
  guard range and its weapon's reach. *Expected:* it picks and fires at it (range search, no
  region test). (3) A hunting
  armed infantry unit placed on such a cell, with one enemy on open ground and one enemy infantry
  unit, unarmed, on another such cell at least 30 cells away (out of sight). *Expected:* it picks
  the far cliff enemy, never the open-ground one. (4) A human-owned vehicle on open ground
  in *guard*, targetless with passive acquisition disabled in the test map, and an enemy
  computer-owned armed infantry unit on a cliff cell in *guard*. Place them within both weapons'
  reach and within the vehicle's sight; the infantry acquires and shoots the vehicle by its
  range search. Enable the retaliate property of *guard* in the named test data, and satisfy
  every other retaliation condition of claim 18 and S-0215 claim 18. Record the first hit,
  the vehicle's target before and after it, and its first return shot. *Expected:* the vehicle
  takes the cliff infantry as its target after the hit and fires back, despite that target's
  cell having no reach region. Control: disable only the vehicle's retaliate flag; it remains
  targetless after the hit. Passive acquisition stays disabled in both runs, so a range-search
  pick cannot masquerade as retaliation. *Falsified if:* run 1 picks the cliff unit or the
  unit moves, run 2 does not pick it, run 3 picks the open-ground enemy or nothing, or run 4
  fails to retaliate under the stated conditions or its control takes the attacker as target.
  (5) Repeat run 3 with the searching infantry placed in *area guard*, its guard spot at its
  initial cliff cell, and both enemies within the area radius and weapon reach. Place the
  second cliff enemy in a visited ring and leave every earlier visited cell without an eligible
  enemy. *Expected:* the area search picks the cliff enemy, not the open-ground one.
  *Falsified if:* its first qualifying area search picks the open-ground enemy or nothing.
  Stop each run after its first qualifying pick or return shot, and before a pursuit can
  obscure that pick. A run without the specified initial cell, behaviour and region states
  is inconclusive, not confirmation.
- *Part k (hovering jumpjet, claims 4, 7, 9, 18 and 24; Q-1504):* a vehicle with an anti-air
  weapon, computer-owned and placed in *hunt* by the test map, and one human-owned enemy
  vehicle with jumpjet movement. Give orders only to the human-owned jumpjet. Record targets,
  the hunter's behaviour, and the jumpjet's height and render layer every tick. Keep the
  hunter targetless until the qualifying search in each run. Runs: (1) the jumpjet unarmed,
  ordered to a cell within the hunter's weapon reach and
  given no further order once it hovers there; count only runs whose first hunt search occurs
  after it is hovering. A longer hunt processing delay in the named test data may provide the
  input window.
  *Expected:* the hunter never picks it while it hovers. (2) As run 1, but leave the jumpjet
  on the ground until the hunter's target recorder shows its acquisition, then order lift-off.
  *Expected:* the hunter picks it on the ground and keeps it as target after it rises,
  firing at it while it hovers.
  (3) As run 1 with the jumpjet armed with a disc-laser weapon (S-0615) and attacking the hunter,
  once with the *hunt* behaviour's retaliate property off and once with it on (test data).
  *Expected:* with it off the hunter never targets the hovering jumpjet; with it on it targets it
  after the first hit (retaliation, no layer test of its own). Count only hits for which the hunter is
  targetless, still in *hunt*, and satisfies every other retaliation condition of claim 18
  and S-0215 claim 18; absence of such a hit is inconclusive. (4) As run 1 with the hunter
  placed in *guard* by the test map and the jumpjet within its guard range and its weapon's reach. *Expected:* it picks it (airborne
  scan of the range search). *Falsified if:* run 1 or the property-off run of 3 picks the hovering
  jumpjet, run 2 drops it when it rises, the property-on run of 3 does not retaliate, or run 4
  does not pick it. For run 4 place the jumpjet in a tracked sector reached by claim 9's
  search, so a sector omission does not decide the result. Any run in which the recorded
  render layer of the jumpjet becomes the ground layer while it attacks is recorded separately:
  ordinary low-height cases can qualify for the map-wide search, but firing does not force
  that transition (S-0502 claim 32). Such a run does not test the outside-ground-layer exclusion.
  For run 3 use a settled hover over flat terrain that remains at least 208 leptons above the
  surface throughout charge-up and damage, as in S-0502 claim 32's measurement scenario.
  Ground-layer entry at that height is contrary to that claim, not an expected attack descent.
- *Falsified if:* part b picks the unreachable target, part a does not move, part c differs as it
  says, part d picks the island enemy, part e does not, or parts f to k differ as they say.

**A12: healers (claim 6).** A healing infantry type next to two damaged allied infantry and one
damaged allied vehicle: it heals the infantry only, the more damaged first when the strength
coefficient is negative. A healing vehicle type next to a damaged allied vehicle and a damaged
allied infantry unit: it heals the vehicle only.
- *Part b (radius, claim 6):* the healing infantry type in *guard* with its weapon range set to 8
  cells; a damaged allied infantry unit alone in a cell adjacent to the healer's (ring 1) is
  healed, another alone 3 cells away (ring 3) is never healed.
- *Part c (aircraft, claim 6):* the healing vehicle type, with a weapon whose projectile is both
  anti-air and anti-ground (test map, so that the firing check does not decide), next to one
  damaged allied aircraft at a time: (1) landed on open ground: healed; (2) landed on a pad:
  never healed.
- *Part d (healer in hunt, claim 7, table C row "healer"):* the healing infantry type ordered to
  hunt, with one damaged allied infantry unit far away (out of its sight), no other damaged allied
  object, and a damaged enemy infantry unit, its weapon removed in the test map, nearer to it than
  the ally, with the target recorder.
  *Expected:* it picks the allied unit and goes to heal it. *Falsified if:* it picks nothing, or
  picks the enemy.
- *Part e (allied condition with the attack-friendlies flag, table C row "healer"):* the healing
  vehicle type, given the attack-friendlies flag in the test map, ordered to hunt on a map whose
  only damaged objects are a damaged enemy vehicle and, in a second run, also a damaged allied
  vehicle, with the target recorder. *Expected:* in the first run it picks nothing; in the second
  it picks the allied vehicle. *Falsified if:* it picks the enemy vehicle in either run.
- *Falsified if:* a healer for acquisition picks an undamaged friend, an enemy, or the wrong
  class; the healer heals the unit 3 cells away in part b, or not the adjacent one; in part c the
  aircraft on a pad is healed, or the one on open ground is not; or parts d and e differ.

**A13: disguised, passive-house and insignificant targets (table C).** Each object is alone in a
human firer's range, in its own run.
- *Disguise:* a disguised enemy infiltrator is never attacked by itself (it is attacked when
  ordered). Control: the same infiltrator undisguised is attacked by itself.
- *Passive house:* a building of the neutral house (placed by the test map, S-0009 claim 27),
  given a weapon and a threat-posed value above 0 in the test map, with the test rules marking the
  neutral house's country passive for multiplayer: never attacked by itself. Control: the same
  setup with the test rules marking that country not passive: attacked. If the game shows the
  neutral house allied with the human house, both runs are made with the special house instead.
- *Insignificant:* a vehicle of the enemy computer house whose type is insignificant in the test
  map is never attacked by itself. Control: the same type without the flag is attacked.
- *Falsified if:* an object is attacked by itself while its condition holds, or a control is not
  attacked.

**A14: further rows of table C.** Each part has the firer of the general setup and an object in
range; the part says which object.
- *Part a (land targeting):* a firer whose land-targeting mode is 2, with an enemy vehicle on land
  and an enemy ship in water, both in range: it attacks the ship only. With mode 0: it attacks the
  vehicle.
- *Part b (underground):* an enemy tunnelling unit underground in range is never attacked while
  below the surface and is attacked after it surfaces.
- *Part c (invisible building):* an enemy building whose type is invisible in game, given a weapon
  and a threat-posed value above 0 in the test map, next to the firer, is never attacked by itself,
  and the same building with the flag off is attacked.
- *Part d (harmless building):* a human-house firer with a defenceless enemy building (no weapon)
  in range never attacks it by itself; it attacks an enemy building with a weapon and a
  threat-posed value above 0 in range, and never one with a weapon and a threat-posed value of 0.
- *Part e (legal target):* an enemy vehicle whose type is not a legal target in the test map, in
  range: never attacked by itself; with the flag restored, attacked.
- *Part f (no-threat):* an enemy in a behaviour that has the no-threat property (S-0215, claim 16),
  in range: never attacked while in it, attacked once it leaves it.
- *Part g (cloak):* an enemy vehicle of a type that cloaks (or covered by a cloaking building of its
  owner), fully cloaked, in range. *Expected:* never attacked by itself; when a sensor of the
  firer's house (S-0516 claims 21–23) covers its cell,
  it is attacked.
- *Part h (playfield):* an enemy vehicle that arrives from beyond the map edge (a reinforcement
  that a map trigger sends in, S-0011 claim 30), with the firer standing at the edge of the usable
  area so that the vehicle is in range before it enters. *Expected:* it is not attacked before it
  enters the usable area, and is attacked after.
- *Part i (harvester immunity):* the map sets harvester immunity; an enemy vehicle of one of the
  game-wide harvester types in range. *Expected:* never attacked by itself; on a map without the
  setting, attacked.
- *Part j (train):* a human-owned unarmed vehicle-thief infantry unit ordered to hunt, on a map
  whose only enemy is a vehicle whose type has the train flag (test map). *Expected:* it picks
  nothing; with the flag removed, it picks the vehicle.
- *Part k (bridge layers):* the firer on the deck of a bridge, and an enemy vehicle in range under
  the bridge, on another bridge cell whose deck holds nothing. *Expected:* never attacked by itself; the
  same vehicle on the deck is attacked.
- *Part l (passengers of a mind-controlled open-topped transport, claim 10, table C rows "friendly"
  and "controller's allies"):* an enemy open-topped transport carrying armed passengers, taken
  over by a mind-controlling unit of the human house *H* (the transport's original house is *E*; a
  third computer house *F* is allied with neither), standing still with the target recorder on the
  passengers. In separate runs, the only objects within the passengers' range besides the
  transport and the controller are: (1) a vehicle of *H*, alone in its cell; (2) a vehicle of *E*,
  alone in its cell; (3) a vehicle of *F*, alone in its cell; (4) one cell holding an infantry
  unit of *H* and an infantry unit of *F*, run twice with the two entering the cell in either
  order. All these objects have their weapons removed (test map). *Expected* with
  `quirks.mind_controlled_passengers_cell_choice` on: (1) never picked; (2) and (3) picked by
  themselves; (4) neither unit picked in either order. With the switch off, (4) gives the unit of
  *F*. *Falsified if:* (1) is picked, (2) or (3) is not, or in (4) a unit is picked with the switch
  on.
- *Part m (airborne objects of other classes, table C row "class"):* a firer with an anti-air
  weapon and guard range 8 cells, and an enemy infantry unit with jumpjet movement hovering in
  range. *Expected:* it attacks the hovering unit (the scan of airborne objects accepts infantry
  as well as aircraft). Run 2 (allied airborne objects, claim 9): the hovering unit belongs to the
  firer's own house, with the firer type's attack-friendlies flag off, then on (test map).
  *Expected:* off: never attacked by itself; on: attacked. Run 3 (no one-object-per-cell rule in
  the scan of airborne objects, claim 9): two enemy jumpjet infantry units hovering over one cell,
  one of a type against whose armour the firer's warhead is 1 % and one at 100 %, run with the two
  lifting off in either order. *Expected:* the 100 % unit is attacked in both orders (contrast
  scenario A3). If the two cannot be made to hover over one cell, run 3 does not decide.
  *Falsified if:* the firer never attacks the enemy hovering unit, attacks the allied one with the
  flag off or never with it on, or in run 3 attacks neither unit in some order.
- *Part n (no draws by the candidate tests, claim 12, State and ordering):* the firer is a
  defensive building of the human house with a 1 × 1 foundation, target-health scan mode *none*, a
  guard range of 8 cells and a weapon range of 1 cell (test map: *n* = 8, so ring 2 lies within the
  first early-stop band, and nothing in ring 2 is within its reach, so it never fires). Ten enemy
  vehicles of one type, weaponless and stationary (test map), stand each alone in a cell of ring 2.
  Random-state recorder, tick recorder and target recorder. Two runs from the same seed and the
  same test files, differing only in the targets' type's legal-target flag: set in run 1, so that
  the firer's first search tests and scores all ten, and cleared in run 2, so that table C rejects
  every one of them. The search tick is the tick on which the target recorder of run 1 first shows
  a target. *Expected:* the stream state at the end of that tick is the same in both runs.
  *Falsified if:* it differs, in particular if run 1 is ahead of run 2 by a number of draws that
  grows with the number of targets (checked with a third pair of runs using five targets). A pair
  whose stream states already differ at the start of the search tick does not decide and is
  repeated with another seed.
- *Part o (0 health, row "exists"):* on a tiny flat skirmish test map, put a stock building
  whose type has the explodes flag in the human house, and an armed ground searcher of the
  neutral house in *guard* nearby. Keep the bystander computer player's objects away from the
  observed area. Use game speed 6, with no engine add-on and no mods (the named test files of a scenario excepted), and target, health and tick
  recorders. The neutral searcher avoids the human owner's harmless-building rejection; choose
  positions where the living building passes every other candidate row and is picked in a
  control run. In the test run, destroy the building by a separate human-owned unit's
  force-fire before a search by the targetless searcher, while the building still awaits removal
  under S-0312 claim 21. Repeat with fresh seeds until three runs contain a search in that
  interval; a run with no such search does not decide. Record the searcher's new target, the
  building's health and whether it still occupies its foundation cells at that search.
  *Expected:* the building is picked in the living control, but never newly picked at 0 health
  while awaiting removal. *Falsified if:* a qualifying search newly picks the building at
  0 health before removal. Merely retaining a target picked while it was alive does not decide
  this candidate test (claim 4).
- *Falsified if:* any part differs as it says.

**A15: the neutral and special houses (claim 25).** Objects of the neutral house are placed by
the test map (S-0009, claim 27); in every part the neutral house is not allied with the human
house, and a part whose game shows otherwise is run with the special house instead. Each part
also runs its control: the same object owned by the human house, which must behave as the other
claims give for a human-controlled owner. Target recorder in every part.
- *Part a (row "harmless building" does not apply, (f)):* an armed vehicle of the neutral house in
  *guard*, with an unarmed building of the human house as the only object within its range.
  *Expected:* it picks the building by itself. Control: not picked (scenario A14, part d).
- *Part b (deploy-to-fire and no-auto-fire, (b)):* a vehicle of the neutral house whose type
  deploys to fire, and in another run one whose type has the no-auto-fire flag (test map), each in
  *guard* with an enemy vehicle of the human house within range. *Expected:* each picks the enemy
  by itself. Control: never (scenario A2, main part and part e).
- *Part c (disguise draw, (g)):* an armed vehicle of the neutral house in *guard* whose type
  cannot detect disguises, with a disguised infiltrator of the human house within range as the
  only object, whose disguise is made to blink (the disguise specs own how; for example by a hit).
  Two settings of the test rules: every entry of the disguise-detection list 100, then every entry
  −1, so that the result does not depend on which entry the house uses. *Expected:* with 100 the
  infiltrator is picked while its disguise blinks and never while it does not; with −1 it is never
  picked. If no blink can be arranged, the part does not decide. *Falsified if:* with −1 the
  infiltrator is picked, or with 100 it is picked while not blinking or never while blinking.
  Run 3 (one draw per test): two runs from the same seed, every entry −1, and the searcher's type
  able to detect disguises in run 2 only (test map), with the random-state recorder and screen
  capture. In both runs the disguise is made to blink in the same way while the infiltrator stands
  beyond the searcher's search radius, and the infiltrator is then moved into range during the
  blink. The search tick is the first tick on which run 2's target recorder shows the infiltrator
  as the target (its search then sees through the disguise without a draw). *Expected:* at the end
  of that tick run 1's stream is exactly one draw ahead of run 2's. *Falsified if:* the two streams
  are equal, or differ by any other number of draws. A pair whose stream states already differ at
  the start of that tick, or in which the screen capture shows the blink over before that tick,
  does not decide.
  Runs 4 and 5 (for which objects the draw is taken): pairs of runs as in run 3, but with the
  infiltrator left where a row of table C rejects it in both runs of the pair, while its disguise
  blinks: in run 4, with the searcher's guard range set to 8 cells (*n* = 8) and its weapon range
  below that (test map), in the cell (7, 7) relative to the searcher's cell, which lies in ring 7,
  a ring the search looks at when nothing nearer is a candidate, but about 9.9 cells away, beyond
  *R* (row "distance", which prevents the draw); in run 5 under a bridge on a bridge cell whose deck holds nothing, with
  the searcher on the deck within range (row "bridge layers", one of the six rows that do not
  prevent it). *Expected:* in run 4 the two streams stay equal through the blink, and in run 5 run
  1's stream is ahead of run 2's by the end of the blink. *Falsified if:* run 4's
  streams differ, or run 5's stay equal. A pair whose streams differ before the blink does not
  decide.
  Run 6 (the order of the draws across objects): the searcher's guard range set to 8 cells
  (*n* = 8) in the test map, every entry of the disguise-detection list 50, and two disguised
  infiltrators of the human house, one in ring 1 and one in ring 2 of the searcher's cell, made to
  blink together while beyond the search radius and then moved into range during the blink, with
  the random-state recorder. On the search tick the predictor gives, for each order of the two
  disguise draws, which infiltrators pass and so which one is picked. *Expected:* the stream
  advances by the timer draw and two ranged draws, and the pick is the one the order "ring 1, then
  ring 2" predicts. Only search ticks on which the two orders predict different picks count.
  *Falsified if:* the pick is the one the other order predicts. Runs: until 10 counted ticks.
- *Part d (charges row of claim 18, (h)):* an armed infantry type with the explosive-charges flag
  (test map), owned by the neutral house, in *guard* with no target, attacked by a defensive
  building of the human house beyond its own range but within its sight. *Expected:* it
  retaliates. Control: never (scenario A5). *Falsified if:* the neutral unit does not retaliate.
- *Part e (engineers, (a), (c) and (d)):* an engineer type given an anti-ground weapon in its first
  slot (test map), owned by the neutral house, in *guard*, with two enemy buildings of the human
  house within range, both with a weapon and a threat-posed value above 0 (test map), one of a
  capturable type and one not. *Expected:* it picks the capturable building by itself; with that
  building removed it picks nothing. Run 2: the same engineer in *area guard*, with no enemy within
  range and a building of the neutral house of a capturable type that accepts repair by an
  engineer, at a health ratio below the red condition ratio, within range. Run 3 (the map-wide
  search): the same engineer placed by the test map in *hunt* (S-0009 places an object with a
  behaviour), with that damaged building out of its sight and no enemy capturable building on the
  map. Runs 2 and 3 are each made twice, with the test rules marking the neutral house's country
  passive for multiplayer, then not passive, because the building's owner is then the searcher's
  own house and the row "passive house" in the form of claim 25, part e, decides. *Expected:* with
  the country passive the building is never picked (it is neither captured from another owner nor
  occupiable by the engineer); with it not passive the building is picked, and in run 2 a
  building at full health is not. Control: never picks by itself (scenario A1, part e).
  *Falsified if:* the engineer never picks the capturable enemy building, picks the other one, or
  in runs 2 and 3 picks the building with the country passive, never picks the damaged one with
  it not passive, or picks the one at full health.
  Run 4 (F1's engineer exception in the per-tick search, claim 8): the engineer of the main run,
  whose warhead has versus values above 0 against the building's armour class, with a guard
  range of 10 cells and a weapon range of 1 cell (test map), in *guard*, with the capturable
  enemy building 5 cells away (inside the guard range, beyond the reach) as the only object
  within 10 cells; in a second run the building stands next to it instead (within the reach).
  *Expected:* it never picks the building at 5 cells (its range search uses *R* = 0, so a
  candidate must be within its weapon's reach) and picks the one next to it. Control: the same
  type with the engineer flag removed in the test map picks the building at 5 cells (*R* = 10
  cells). *Falsified if:* the engineer picks the building at 5 cells, does not pick the one next
  to it, or the control does not pick the one at 5 cells.
- *Part f (threat sum, (h)):* a vehicle of the neutral house in *guard* that has picked *X* by
  itself, an enemy vehicle of the human house in range whose weapon is removed (test map). An enemy
  vehicle *Y* of the human house, of a type whose special threat value the test map sets, then
  attacks it from beyond its range but within its sight. The neutral vehicle's type has every
  threat coefficient 0 except the special-threat coefficient. Run 1: *Y*'s special threat value
  above *X*'s; run 2: below. *Expected:* run 1: it retaliates against *Y*; run 2: it keeps *X*.
  *Falsified if:* run 1 does not retaliate or run 2 does.
  Run 3 (draining without a target): let a neutral-house drainer, placed over an enemy drainable building of the human house, acquire it automatically and form a drain link (S-0608 claims 1 and 2). Once the attempt clears its target and it is in *guard*, have a
  human-owned anti-air attacker damage it from beyond its weapon range but within its sight.
  Record the link, target, behaviour and attacker's distance at the hit and the target after
  it. Void and repeat if the drainer has acquired any target before the hit, its link has ended,
  or another retaliation condition of claim 18 or S-0215 claim 18 fails. Control: the same
  targetless drainer in *guard*, not draining, attacked from the same distance. Run each three
  times under the general setup. *Expected:* the drainer, while draining, keeps no target and does not
  retaliate; the control takes the attacker as its target. With no held target, the threat-sum
  comparison does not apply. *Falsified if:* the drainer, while draining, retaliates or the control does
  not, with the stated conditions satisfied.
- *Part g (enemy walls, (i)):* the test rules set the destroy-walls flag in every difficulty row
  (so the result does not depend on the house's row). A vehicle of the neutral house in *guard*,
  with a guard range of 8 cells (*n* = 8, early stops after rings 2 and 4) and a weapon of range 6
  cells whose warhead has the wall flag and whose projectile is anti-ground (test map). With the
  firer's cell as (0, 0), walls and enemy vehicles of the human house (weaponless, at full health)
  are placed as each run says; nothing else besides these stands within 8 cells.
  - Run 1: one wall at (−3, 0), no vehicle. *Expected:* its target becomes the wall's cell.
  - Run 2: as run 1, with the flag cleared in every row. *Expected:* no target.
  - Run 3 (the quirk, ring 3 is not an early-stop ring): a wall at (−3, −3), the first cell of
    ring 3 in F2's order, and a vehicle at (3, 3), later in that ring. *Expected:* the wall's cell.
  - Run 4 (the early-stop row): a wall at (−2, −2) and a vehicle at (2, 2), both in ring 2.
    *Expected:* the vehicle.
  - Run 5 (the search ends with ring *r*): a wall at (−3, 0) and a vehicle at (5, 0), in ring 5
    and within the weapon's reach. *Expected:* the wall's cell. Control: the vehicle alone is
    picked.
  - Run 6 (an airborne candidate): the weapon's projectile both anti-air and anti-ground and its
    warhead's versus values above 0 against every armour class on the map (test map), a wall at
    (−1, 0), and an enemy aircraft of a type that can stay in the air over one place, hovering
    over (4, 0). *Expected:* the aircraft.
  - Run 7 (distributed fire): as run 1, with the firer's type given the distributed-fire flag
    (test map). *Expected:* no target.
  *Falsified if:* run 1, 3 or 5 does not give the wall's cell, run 2 or 7 gives any target, run 4
  gives the wall's cell, run 6 gives the wall's cell, or the control of run 5 picks nothing.
- *Part h (the search's draws, claim 4 and State and ordering):* the searcher is the armed vehicle
  of the neutral house of part c, in *guard*. The recorder also reads, at the start of every
  tick, the full state of the scenario random stream (the random-state recorder of the
  measurement setup), and the searcher's processing timer and targeting timer as in S-0215's
  measurement scenarios. From the recorded state, S-0202's generator and its range rule (claim 4)
  predict the values that a tick's draws would receive in a given order. Only *search ticks*
  count: ticks on which the searcher's targeting timer runs out and its search runs, and on which
  the searcher does not process, so that no draw of its processing falls on the tick. *N* is the
  normal targeting delay (S-0215, F6), whose draw is R(0, 2).
  - Case i (no fixed further draw): no object within the searcher's search radius, so its search
    finds no candidate. On each search tick the claim predicts that the stream advances by exactly
    the raw values that one draw R(0, 2) takes from the recorded state, with result *t*, and that
    the targeting timer restarts at *N* + *t*. A search tick on which the stream advances further
    is set aside, since another rule may have drawn on it. Runs: until 30 search ticks that
    advance by exactly that amount, or 300 search ticks. *Expected:* such ticks occur, and on
    each of them the restart is *N* + *t*. *Falsified if:* none of the 300 search ticks advances
    by exactly that amount (a fixed further draw would make every search tick advance further),
    or a tick that does restarts the timer at a value other than *N* + *t*.
  - Case ii (the timer draw before the disguise draw): every entry of the disguise-detection list
    set to 50 in the test rules, and the disguised infiltrator of part c, made to blink, within
    range as the only object. For a search tick during the blink, the predictor gives the
    readings (*t*, *p*) when the timer's R(0, 2) is drawn first and the disguise draw R(0, 99)
    second, and (*t*′, *p*′) for the other order. A search tick is valid when the stream advances
    by exactly the raw values that these two draws take in at least one of the two orders, and
    only valid ticks on which the two orders give different readings count. *Expected:* the
    targeting timer restarts at *N* + *t*, and the target recorder shows the infiltrator as the
    target at the end of the tick exactly when *p* ≤ 50. *Falsified if:* on such ticks the
    restart is *N* + *t*′, or the pick follows *p*′ and not *p*. Runs: until 10 counting ticks
    (once the infiltrator is picked, the searcher keeps it and tests no disguise, so each blink
    gives at most a few counting ticks).
  - Control: the same as case ii with the searcher owned by the human house. Its disguise test
    rejects the infiltrator without a draw (table C, row "disguise"), so on its search ticks the
    stream advances by exactly the raw values of one R(0, 2), with result *t*, and the timer
    restarts at *N* + *t*, as in case i. *Expected:* such ticks occur, every one restarts at
    *N* + *t*, and the infiltrator is never picked. *Falsified if:* a search tick that advances by
    exactly that amount restarts at another value, the infiltrator is picked, or none of 300
    search ticks advances by exactly that amount.
- *Falsified if:* any part differs as it says.

Every sub-rule that has no scenario part above is marked `unmeasured`, with its reason, in its
claim's backing.


**A16: initial and layered coefficients (claim 26; Q-0061).** Use the common measurement
setup, a flat map, one human hunter and two stationary enemy vehicles, outside firing reach,
with the target recorder. Start each run in a fresh session. Named test rules remove the chosen
type coefficient from every layer and set its game-wide default nonzero in base rules. The
searcher's type section exists in base rules. Choose candidate properties so F4 predicts different
first targets with that coefficient equal to 0, the base default, and a distinct map default.
Repeat separately for each of the five coefficients, holding the other terms fixed. For the
own-effectiveness term use different armour classes; for target-effectiveness use different
candidate weapons; for special-threat use different special threat values; for strength use
different health ratios; for distance use different distances.
- *Run 1:* no later default change. *Expected:* the pick follows the base default, not 0.
- *Run 2:* the map changes only the game-wide default and contains the type section without its
  coefficient. *Expected:* the pick still follows the nonzero base default, not the map default.
- *Run 3:* base default 0, map default nonzero, with that same type section present in the map.
  *Expected:* the pick follows the map default. Control: omit the map's type section altogether
  and ensure no intermediate layer supplies it; the pick follows 0.
- *Run 4:* explicitly give the type coefficient 0 in base rules, with a nonzero base default,
  and include its section without that coefficient in the map. *Expected:* the later reading
  replaces 0 by the default then in force. Control: omit that later section and ensure no other
  intermediate section reads it; the pick follows 0.
- *Observable/tool:* record the first chosen target with the read-only recorder, using scripted
  hunt orders, game speed 2, no engine add-on and no mods (the named test files of a scenario excepted). Keep bystander
  activity away from the test area under the common void rule. *Falsified if:* any valid run's
  first target differs from F4 with the coefficient stated above. On our engine repeat with
  `quirks.coefficient_zero_replaced` off: runs 1 to 4 all retain 0 unless a layer explicitly
  supplies a nonzero type coefficient. This off-setting check is a test of our chosen alternative,
  not a measurement of the original.

**A17: charging break-off search and targeting timer (claim 1).** Use S-0518 scenario TC3's
infantry charging setup on a tiny flat skirmish map, one human house and one bystander computer
player kept away. Game speed 6, no engine add-on and no mods (the named test files of a scenario excepted). Use an
ordinary, non-distributed-fire infantry charger that passes claims 2 and 3, with no attack-move
memory. Give the normal targeting delay a large test value so that several attack processings
occur before it expires. Record current behaviour, charging mark, target, targeting timer and
scenario random state each tick, and identify attack processings using the processing timer.
- *Part a:* move an eligible enemy into range while the charger still has positive ticks on
  its targeting timer. Record three fresh runs containing such a processing. *Expected:* the
  next attack processing replaces the charging building with the enemy and clears the mark,
  without restarting the targeting timer. A target held before the search does not prevent
  replacement. Repeat with the timer already expired at that processing: replacement still
  does not restart it.
- *Part b:* leave no eligible enemy within range for three attack processings in each of three
  fresh runs. *Expected:* the charging building and mark remain, and none of those processings
  restarts the targeting timer. Positive remaining ticks decrease normally; an expired timer
  remains expired rather than receiving the normal targeting delay.
- *Part c:* repeat part a in named test-map overrides with passive acquisition disabled, then
  with opportunity fire disabled while no other condition of claim 3 passes in *attack*.
  Keep the charger armed and otherwise eligible. Record three qualifying processings per
  variant. *Expected:* replacement and mark clearing still occur. As a control, enable the
  no-auto-fire restriction for the human-owned charger: expect no result and retention of
  both charging target and mark. Separately use S-0518 claim 15's scenario TC3 same-result
  setup, allowing allied candidates and making the held building the sole eligible result.
  Record three qualifying processings. *Expected:* the target remains the same object but
  the mark clears, so a later enemy does not trigger this special search. Confirm candidate
  eligibility under claims 5 to 17; failure to realise it makes that run inconclusive.
- *Draw observable:* for each counted processing, predict the scenario-stream draws of its
  ordinary behaviour delay under S-0215 claim 21 and any firing under S-0301. Use a weapon
  without random firing effects and keep all unrelated actors away. Compare the recorded state
  with that prediction and with the prediction containing an additional targeting-delay draw.
  Count only ticks with no unrelated random event; repeat until three isolated processing ticks
  per part are recorded. *Expected:* only the ordinary processing/firing draws, no search-timer
  draw. Do not discard a tick merely because its observed draw count differs from the prediction.
- *Falsified if:* a qualifying positive-timer processing waits for expiry to replace the target,
  a search restarts the timer in either part, a failed search discards the charging target or
  mark, passive acquisition or the opportunity gate blocks part c's otherwise eligible
  search, the no-auto-fire control returns a candidate, a confirmed same-object result retains
  the mark, or an isolated processing tick contains the additional targeting-delay draw.

**A18: a held target that is destroyed or removed (claim 27; Q-2006).** A tiny flat skirmish map,
one human house and one computer house, game speed 6, no engine add-on and no mods (the named test
files of a scenario excepted). Record each tick, at the end of each object's turn where the recorder
allows it and otherwise at the end of the tick: the target of each test object, its fire events
(tick, weapon and what was aimed at), its rearm countdown, its weapon's facing (its turret's, for
a turreted type), its targeting timer and the scenario
random state, and the tick on which the victim is crushed (the crush sound), takes the killing
hit or leaves the map. Each part runs 5 times from a fresh game, unless it says otherwise.
- *Part a (crusher, S-0210 scenario K7 cases 5 and 10):* set up K7 case 5 (computer-owned armed
  crusher in *guard*, which drives over the victim by K7's crushing approach, the victim object
  itself being its destination, and an infantry victim with raised health) and case 10 (human-owned
  crusher ordered to attack a crushable enemy vehicle with raised health). Record from 30 ticks
  before the crush to 30 ticks after it. In both cases give the crusher a weapon whose rearm delay
  is 1 tick in the test map, as part b does for *A* and *A*′. Also give the crusher's type the
  mobile-fire flag and that weapon the fire-while-moving flag in the test map, so that driving onto
  the victim does not refuse its shots (S-0310 claim 7). Then a crusher whose firing step came
  before its movement could fire in the crush tick. A run in which the crusher does not hold the
  victim as its target in the tick before the crush, does not fire at the victim in that tick (so
  its aim or another refusal could hide a shot), whose recorded facing is not on the victim at the
  start of the crush tick, or whose recorded rearm countdown would not let it fire in the crush
  tick, decides nothing. *Expected:* at the end of the crush tick the crusher
  holds no target, it has no fire event in the crush tick or later aimed at the victim or its cell,
  and its rearm countdown goes on from its value before the crush without a restart in that tick.
  *Falsified if:* the crusher still holds the victim after the crush tick, or fires in the crush
  tick or later at the victim or its cell.
- *Part b (removal by another unit, both turn orders):* a human-owned infantry firer *A* ordered
  to attack a stationary enemy vehicle *V* whose health is raised so that neither the firer nor
  *C* can kill it, and a human-owned armed crusher *C* set up as in S-0210 scenario K7 case 10
  (crusher and omni-crusher flags, ordered to attack *V*), so that *V* is crushable and does not
  move away. A run in which *C* does not crush *V* decides nothing. In run b1 *C*, a vehicle,
  takes its turn before the infantry *A* in every tick, because map-placed vehicles enter play
  before map-placed infantry (S-0203 claim 3). In b1 place *V* after *C* among the map's vehicle
  entries, so that crushing *V* in *C*'s turn removes an object standing after *C* and costs no
  object its turn (S-0203 claim 10). A b1 run in which *A* takes no turn in the crush tick decides
  nothing. Where the recorder allows a read between two objects' turns, b1 also records *A*'s
  target at the end of *C*'s turn in the crush tick, before *A*'s own turn. Run b2 replaces *A*
  by a human-owned armed vehicle
  *A*′ without the crusher flag, standing within range of *V* and ordered to attack it, and places
  *A*′ before *C* among the map's vehicle entries, so that *A*′ takes its turn first (same claim,
  entry order within one kind). A b2 run in which *A*′ moves decides nothing. Give *A* and *A*′ a
  weapon whose rearm delay is 1 tick in the test map, so that rearm never decides whether they
  fire. *Expected:* b1: *A* has no fire event in the crush tick or later, and holds no target at
  the end of its turn in the crush tick; where read, *A* already holds no target at the end of
  *C*'s turn in that tick. b2: *A*′ may fire in the crush tick, before the crush,
  and has no fire event after it. In both runs the firer holds no target after the crush.
  *Falsified if:* the firer fires after the crush, in b1 in the crush tick, or still holds *V*
  after the crush tick. The drop's moment inside the action is falsified if, where read, *A* still
  holds *V* at the end of *C*'s turn in the crush tick.
- *Part c (killing hit of an object that stays in play):* *A* as in part b, ordered to attack an
  enemy infantry *V* of a type whose death sequence leaves it in play for at least one tick after
  the killing hit, and a human-owned armed
  vehicle *K* ordered to attack *V*. Give *A* a weapon with a small nonzero damage in the test map,
  and set *V*'s health above the total damage of *A*'s shots before *K*'s first hit and at most the
  damage of one shot of *K*, so that only *K*'s first hit kills it. A run in which *A* has no fire
  event at *V* before the killing hit decides nothing. *Expected:* after the killing hit *A* holds
  no target and fires no shot at *V* or its cell, although *V* is still in play during its death
  sequence. *Falsified if:* *A* holds *V* or fires at it or its cell on any tick after the killing
  hit.
- *Part d (timer draw):* part b, run b1, but *A* has no order: it starts in *guard* with *V* in
  range and picks it by its own search (claims 1 to 4), with the normal targeting delay given a test
  value of 100 ticks. Start *C* at different distances so that the crush ticks fall at different
  points of *A*'s targeting timer. A run in which *C*'s targeting timer has more than 10 ticks left
  immediately before the crush is void, so that *A* is the only holder whose drop can take a draw.
  Classify crush ticks by the timer's remaining ticks immediately before the crush, repeating until
  at least five have more than 10 left and five have at most 10 left. This overrides the general
  count of 5 runs. Give *V*'s type empty explosion and destruction-animation lists in the test
  map, so that its crush takes no animation pick (S-0210 claim 4). *A* is present in every run,
  and no control run is used: the added draw is told by its range and by *A*'s timer. The
  recorder lists the scenario-stream draws taken in *C*'s turn in the crush tick, each with its
  range, where it allows that read, and otherwise those of the whole crush tick, with *A*'s own
  draws in its own turn, which the other specs predict, set aside. It also reads *A*'s targeting
  timer immediately before the crush and at the end of the crush tick. *Expected:* with more
  than 10 ticks left, the list has exactly one draw over 4 to 8 that no other spec predicts, and
  *A*'s timer then holds its value. With at most 10 left, the list has no such draw, and *A*'s
  timer goes on counting down from its value before the crush. Any other unpredicted draw in
  the list is reported as a discrepancy. *Falsified if:* the draw over 4 to 8 is missing or
  present against these conditions, or *A*'s timer after the crush differs from that draw's
  value (first group) or from its countdown (second group).
- *Part e (a target that boards a transport):* a computer-owned armed vehicle *A* in *guard*,
  and a human-owned infantry *V* within *A*'s range, next to a transport of the human house that
  is not open-topped. *V*'s health is raised so that *A* cannot kill it. Once *A* holds *V*, the
  human player orders *V* into the transport. Record from 30 ticks before the boarding to 30
  ticks after it. *Expected:* from the tick on which *V* leaves the map, *A* no longer holds *V*
  and has no fire event at *V* or its cell. *Falsified if:* *A* still holds *V* at the end of
  that tick, or fires at *V* or its cell after it.
- *Part f (order of the timer draws, Q-2114):* a human-owned crusher *C* with no weapon, ordered
  to a cell beyond a cell *X* of a one-cell-wide corridor, and enemy infantry of a crushable type
  in *X*, unarmed (test map if needed) and with raised health. Two holders *H*1 and *H*2,
  human-owned armed infantry ordered to
  attack, with the normal targeting delay given a test value of 100 ticks and weapons that
  cannot kill the victims. *H*1 is placed by the map, and *H*2 is made later in the run by a map
  trigger (a reinforcement), so *H*1 was created first. After *H*2 has entered play, *H*1 is
  ordered into a transport that is not open-topped and out again, so that it enters play again
  after *H*2 and takes its turn after it (S-0203 claim 3). Only then are both ordered to attack.
  *H*1 stands farther from *X* than *H*2.
  - (f1) one victim in *X*, held by both holders.
  - (f2) two victims in *X*, *V*1 entering *X* after *V*2, so that *V*1 is listed first (S-0209
    claim 13). *H*1 holds *V*2 and *H*2 holds *V*1.
  A run decides nothing if either holder's targeting timer has 10 ticks or fewer left
  immediately before the crush, if the recorded update order does not put *H*2 before *H*1, if
  the two draws over 4 to 8 in the crush tick give the same value, or if an event shown by its
  own record to be unrelated to the crush (such as a holder's search in the crush tick) takes a
  draw over 4 to 8 there. Infantry victims take no animation pick (S-0210 claim 4). Any other
  draw in the crush tick that the specs do not predict does not void the run: it is reported as
  a discrepancy. Five valid runs per part. Each holder's timer after the crush
  holds the value of its own draw, which tells which draw was whose. *Expected:* in f1 *H*1 has
  the first of the two draws, in f2 *H*2 has it. *Falsified if:* in f1 *H*2 has the first draw
  (update order or distance order), or in f2 *H*1 has it (the draws ordered by holder, not by
  removed object).

## Open questions

- Q-2112 question 1: which occasion gives the hunting vehicles of the games that the request
  cites their hovering-jumpjet targets after their first targets die. Their own map-wide search
  never finds one (claim 7). The occasions that can are a target picked while the jumpjet was in
  the ground layer and still held (claim 7), the limited-scan mark after a failed approach
  (claim 22), and a give-up of the move, after which the search of *area guard* reaches it
  (claim 7). Which of them operates in those games is not established (low). A re-read of one
  of those games would decide it: for each hunting vehicle, every change of its behaviour, target
  and destination with its tick, each route failure and whether it gave the move up, and for
  each jumpjet its render layer and height on those ticks. S-0215 claim 19's
  scatter from damage, which the original sends such a hunter whatever the player-scatter
  setting says, reaches a player-house hunter of our engine only under that setting or the
  scatter veterancy ability; without them it gives that hunter no new route search.
- The order in which a cell holds its objects (claim 10): which object the search offers first
  in a shared cell. Scenario A3 shows it; the occupancy specs (S-0209) may already fix it.
- The airborne order of claim 9 is S-0303's F7 order. What the registration order inside one sector
  is for objects that lift off in the same tick belongs to S-0303 (claim 6 and its F7); it decides
  only ties between airborne candidates (scenario A4, part d).
- The corrected fall-back refresh and arrival rules of claim 23 (Q-0162) still need the oracle
  runs of scenario A6, parts g, n and o. They no longer leave arrival or the refresh context
  unspecified.
- Whether the skirmish set-up makes houses enemies of each other in a way that gives each an anger
  of 1 before any damage (claim 16); scenario A9, part b, records it, and the set-up specs own it.
- Whether any rule besides damage and alliance changes alters a house's anger (claim 16): not
  every situation in which the original makes houses allies or enemies is known, and in one
  situation that was not studied, making a house an enemy can add a second increment.
- The passengers of a mind-controlled open-topped transport (claim 10, table C and Edge cases) rest
  on one reading. Scenario A14, part l, tests it. The fall-back of claim 10 to the cell's last object rests on the
  same reading.
- What a building that is a healer for acquisition picks (claim 6): the firing check of a building
  with a healing weapon is not given by the combat specs, so whether the fall-back of claim 10 can
  give it a non-allied candidate is open.
- Whether a target that one search drops and then picks again counts as an opportunity target
  (claim 24 gives that it does not, because the object is the same as before the search). No
  scenario here is shown to reach it: A6, part f, decides it only if the firer searches within a
  hidden span that holds no multiple of 16 (low).
- The limited-scan mark of claim 22 rests on one reading of the driving locomotor; the hovering
  and ship locomotors' conditions were not examined.
  Which behaviour a unit is in when its phasing in ends, and so whether the opportunity
  gate of claim 3 passes for a type without the opportunity-fire flag, is not established
  (scenario A1, part f, records it).
- The vehicle-thief take-over of claim 5 rests on one reading and has no scenario (claim 5,
  backing); the engineer and storage rows of table C rest on one reading each, and scenario A2,
  parts c, g and i, test them.
- Which other behaviours run searches (claim 1, last paragraph), with which kinds and centres.
- S-0307 claims 4 and 13 now own the garrison range replacement and open-topped shortest range
  used here. Claim 13 specifies passengers' current weapons, including a multi-weapon passenger's
  weapon-number slot, as F1 here does. No clarification to S-0307 is outstanding.
- The order of the ground-layer objects among themselves in the map-wide search (claim 7): which
  of them it considers first. Claim 17 breaks equal scores of a map-wide search by this order,
  which is an internal order of the original that this spec does not establish.
- Campaign visibility rule (Edge cases, *Original only: campaign missions*): campaigns are out of
  scope for now (project lead, 2026-10-10). When they get their own spec, it should decide whether
  to reproduce the rule as a rule on objects discovered by the acting house, or to follow table C
  as it stands.
- The neutral and special houses (claim 25): S-0412 claim 5 establishes that their difficulty index is 1, selecting the second entry of per-difficulty lists; no difficulty row is applied to them. Scenario A15, parts c and g, avoid needing to distinguish list entries by setting every entry alike. Still open: whether the original ever makes these two houses choose a building for their engineers (part c), and whether they are allied with each other (part e).
- Q-1763 question 2 beyond claim 18's third case: whether any other object fires back at an
  attacker that both its automatic searches and a person's attack order refuse. Only the
  projectile flags, the targeting modes and the SAM flag were compared between the search, the
  order rules and retaliation; the other conditions of claims 5 and 18 and of S-0214 table B
  were not compared case by case.
- The stock anti-air-only defence building of the Q-1763 play report: whether its type falls in
  claim 18's first case (land-targeting mode 1), its second (the SAM flag) or neither is not
  established. Scenario A5 part b reads the stock type's flags before its runs and is void if it
  has neither, so until it runs Q-1761 question 1 is answered per case, not for the stock
  building.
- Other drops at the start of a turn. Claim 29's check is not the only rule that can drop a held
  target at the start of a holder's turn. The others, beyond those S-0215 gives (its claims 18,
  26 and 33, for behaviours other than *attack*), and beyond claim 25 part k's drop of an allied
  target by a holder of a computer-controlled owner, are not given here (low). They concern
  holders and targets outside claim 29's measured runs: a holder whose weapon heals, a holder
  whose berserk state ends, a holder in a behaviour other than *attack*, *guard*, *area guard*
  and *move* (runs p8 and p9a measure the last two), and a holder
  that cannot reach its target's place, or rides in an open-topped transport, while its target
  is out of its weapon's range. Scenario A6 part p assumes that none of them reaches the holders
  and targets of its runs, and rests its falsifiers on that (part p, *Earlier drops*, which
  lists them and rates the assumption). A scenario that sees a held target
  dropped in a tick that claim 29 and the other claims here do not give should record the
  holder's owner, its target's owner and the holder's behaviour.
- Q-2006 question 3, the original's side of one case. A held object that a revert restores after
  it is destroyed or goes off the map (claim 27): what the original's holder does with it while
  it stays in play as a dying object or stays off the map, and what the firing check answers for
  it, is not established. Our engine's rule for the case is the design rule of claim 27.
- The place of claim 27's timer draws at a killing hit against the destruction's own draws in
  S-0312's ordering table (claim 1 there). They are believed to come before the debris draws (S-0312
  precedence 1) and every later row, since the drop comes with the killing hit's damage, before the
  death effects begin; low, `unmeasured`: scenario A18 part d crushes its object, so no killing hit
  with death-effect draws is in its runs. Their place against the rows before precedence 1
  (destruction raises and a slave master's slaves) is not established.
- A route to a shot that leaves while its firer holds no current target (S-0306 claim 3's aim
  without a target). Claim 27 gives no such route for a target that leaves play: a firing step
  that begins with no target fires nothing. Whether the original reaches that aim by some other
  route, for example a shot whose target is supplied apart from the firer's current target, and
  so whether S-0306's quirk can be reached at all, is not established here.
- Q-1761 question 3 (the attack action offered when a person hovers an aircraft, flying or
  landed, with an anti-air-only building selected, with and without force-fire) is order
  legality: S-0214 row B7, which this spec does not change. Row B7 is rated low in S-0214, so
  Q-1761 stays only partly settled: question 3 rests on that row.

## Changes for other specs

S-0212 claim 4: its backing should distinguish its own pathfinding scenarios from the
new acquisition fixture in A11 part j here. Map-placed cliff infantry provide a proposed
measurement of two cells of no reach region being compared; the statement that no such
placement is known is no longer current. This is proposed backing, not a measured result.
No change is needed to its no-link deck-region definition: claims 7 and 11 here explicitly
omit that comparison when the searcher's own deck read has no link.

S-0306 claim 23 gives the firing check's water and land rows, which depend on the firer's
type, and ranks them at rank 4 of S-0301 claim 9; claim 18 here relies on the land row and on
that rank. Its first
row's *rearming* answer is limited to the firer's own gripped unit, as S-0301 claim 9 and S-0607
claim 11 give, so scenario A6 part m here (a held target that lifts off is dropped) agrees with
it.

S-0301 claim 14: the turn that ends the delay could say that a building which then holds no
target makes no firing check and no shot, and the shot is no longer pending. Claim 27 here makes
the case reachable: a held target destroyed or removed while the shot is pending.

S-0301 claim 9: no change needed. A sentence after the rank table could say that the firing
check is never asked about a removed target, because claim 27 here drops it at the removal.

S-0301 claim 9, precedence within rank 4: the claim gains a paragraph after *Deactivated firers
within rank 4*: "**A materialising firer within rank 4.** A materialising firer gets *rearming*
for every target and weapon. It yields only to the *illegal* of a firer bound to a slave master,
for vehicles to their answers of ranks 1 to 3 (the *impossible* of a vehicle playing its death
frames, the launching-spawn *busy* and the melee-link *illegal*) and the earlier tube-transit
*impossible*, and for infantry to the answers that an infantry firer is asked first: the
infantry healer's *illegal* (S-0310 claim 11), the *illegal* of an infantry firer whose type has
the keep-acting flag (S-0214, Terms, *melee-linked*) against a vehicle target that is
melee-linked to an object other than the firer, and any answer that S-0310's open question on
dying and falling sequences settles as asked of an infantry firer before them. It outranks
every other rank-4 answer, among them a falling vehicle's *illegal*, the hidden-target
*impossible* (S-0516 claim 24), the projectile refusals of S-0306 claim 23 and the *illegal*
for a parasite warhead against a protected object (S-0513 claim 27), and the own-gripped-unit
*rearming*, which this claim's *Grip refusals within rank 4* already puts below the
materialising firer's *rearming*. The healer's *illegal* outranks every answer of the general
part, the materialising *rearming* among them (S-0310 claim 11). So a firer that meets all three
answers gets the healer's *illegal* first, then the materialising *rearming*, then the
gripped-unit *rearming*. The three meet only in an infantry firer that heals and holds a unit
it seized, since the healer and keep-acting answers are asked only of infantry firers and the
gripped-unit *rearming* only of a firer holding a unit it seized (S-0607)." Its *Grip refusals
within rank 4* sentence "*rearming* applies with any weapon and outranks every other rank-4
answer except the materialising firer's *rearming* and the *illegal* refusal of a firer bound to
a slave master." becomes "*rearming* applies with any weapon and outranks every other rank-4
answer except the materialising firer's *rearming*, the *illegal* refusal of a firer bound to a
slave master and, for an infantry firer, the answers asked of it first: the infantry healer's
*illegal* (S-0310 claim 11, which ranks above the general part), the keep-acting *illegal*
below and any answer that S-0310's open question on dying and falling sequences settles as
asked of an infantry firer before them." With the list so amended, such a firer whose target
is the unit it seized gets the
healer's *illegal* at claim 29's check here whenever S-0310 claim 11 gives it, and drops the target
there by claim 29's rule for *illegal*. Its firing step replies to the same answer by S-0310 claim
11's reply table, as the Changes for S-0310 claim 11 below amend it for an infantry unit whose
default weapon is a healing weapon. A materialising one drops the target at the first of its
reply to *illegal*
in its firing step and its first checking turn in a check tick (claim 29). The claim's answer list
gains that infantry *illegal* as an answer of its own, asked of an infantry firer before the
other rank-4 reasons: "An infantry firer whose type has the keep-acting flag gets *illegal*
against a vehicle target that is melee-linked to an object other than the firer." Its sentence
"Apart from these rules and a building's pending delayed shot (above), no other precedence
within rank 4 is specified here." stays after the new paragraph. Claim 29 here relies on the
paragraph for a materialising holder. Level: medium, and low for the place of the materialising
*rearming* after a vehicle's answers of ranks 1 to 3, which rests on this claim's order of ranks
1 to 4 among themselves, rated low here, and low for the infantry keep-acting
*illegal* and its place, which are reached only through S-0214's link, whose creation that spec
rates low, and low, `unmeasured`, for the order given for an infantry firer that heals and holds
a seized unit and its drop at the check, which follow S-0310 claim 11, this claim's grip rule as
amended and claim 29, a case no part here sets up, and low for a materialising one's drop
through its firing step's reply, as S-0310 claim 11 rates that the infantry answer shows through
its reply. The paragraph gives no answer for an infantry
unit in a dying or falling sequence: that stays S-0310's open question. So whether a
materialising infantry firer that is falling gets *rearming* or its falling answer rests on that
question (claim 29 here). Backing: `unmeasured`.
For a vehicle firer and a target in the air, scenario A6 part p, run p10a, here measures the keep
until the phase-in has ended, but a firer whose check is not made while it materialises gives the
same records, and no part here gives a materialising firer an answer that outranks *rearming*. The
other refusals and infantry and aircraft firers are `unmeasured` as claim 29 here gives. The
infantry keep-acting *illegal* and its place before the materialising *rearming* are `unmeasured`:
no part here or in S-0214 gives an infantry firer of such a type a melee-linked vehicle target.

S-0504 claim 10: its sentence that the start of the killing-hit sinking state "does not clear
every other object's already-held target" should say that the drop of a held target at the
killing hit is claim 27's here: the killing hit drops it, and the start of sinking adds no
further drop.

S-0504 claim 5: for a vessel sinking from a killing hit, its case "while it is still asked about
as a target" meets only a holder that a revert gave the vessel again (claim 27 here), since every
other holder dropped it at the killing hit. Its sentence that it "does not itself establish when a
target already held before sinking is cleared" can point to claim 27 here for the killing-hit
case.

S-0215 claim 10: its pointer for what the actor does with a target restored by a revert can
name claim 27 here. Claim 27 gives that the removal of such an object drops it again, and our
engine's rule for such a target while it stays off the map or in play after its killing hit.
Claim 28 here points the cloak drops to S-0516 claim 14 and gives the drop on going underground
for any holder, with the same exceptions as that claim's revert causes.

S-0215 F6 and table T2: the targeting timer's draws there could add the drop's reset of claims
27 and 28 here: when a holder drops its target with more than 10 ticks left on that timer, one
ranged draw over 4 to 8 ticks inside the action that drops it, not at a search.

S-0312 claim 1: its ordering table could gain a row for the holders' timer draws of claim 27
here at a killing hit. Their place is open here (Open questions); the leaning is ahead of the
debris draws of precedence 1.

S-0306 claim 3 and scenario P27: under claim 27 here, a target destroyed earlier in the tick by
another firer is dropped before the firer's firing step, which then fires nothing; a firer whose
turn comes first fires at the live target. P27's setup therefore gives no shot with an empty
current target, and needs another way to reach one, or a restatement.

S-0210 claim 4 points to claim 27 here for the holders of a crushed object, and gives a crushed
vehicle's animation picks, which come before its holders' timer draws (claim 27). Scenario A18
part a here reuses S-0210 K7 cases 5 and 10 and records target, fire events and rearm countdown
per tick around the crush.

S-0212 claim 23, row 2: the give-up also drops the unit's target, which the row does not say
(claim 7 here, scenario A11 part c run c3). Claim 22 here sets the limited-scan mark only at
failures that do not give the move up.

S-0215 claim 33 already includes the charging break-off exception and its backing through
scenario A17 here and S-0518 scenario TC3; its F6 also excludes that search from timer resets.

S-0304 claim 2: for every actor, whatever its owner (claim 29 here covers every holder of every
owner, in every behaviour but *capture* and *sabotage*, whatever gave its target), the drop
comes at claim 29's periodic check here, for a building in *attack* also at its own attack
processing (S-0215 claim 32), and for a healer also at its firing step, where its own reply to
*illegal* drops the target (S-0310 claim 11's reply table, as claim 29 here gives), whichever
comes first. The attack
processing's drop is a building's only, for every owner, so a unit of the neutral or special
house that is not a healer has no drop at the attack processing or the firing step: it drops
the target at the periodic check, besides claim 29's drop in each turn of a holder being erased
and the other target-loss rules. An actor of a
computer-controlled owner whose target is allied with it drops it earlier, whatever gave it the
target: in its first turn after the order is applied (for a target from an order), or after it
took the target (for one it picked itself), that is not cut short and not a warp turn (claim 25
part k, claim 29 here), unless one of claim 25 part k's exceptions here holds: it is berserk,
it is an infantry unit that
can occupy the target building, it is an infantry unit of an engineer type, or its second weapon
is an electric-charge weapon and the target a building of an overpowerable type. With such an
exception it keeps the target to the first of those same occasions: the periodic check, a
building's attack processing or a healer's firing-step drop. Its sentence "It drops the object
as its target, and stops the attack, at the first check of its attack behaviour whose answer
is *illegal*." becomes "It drops the object as its target at the first occasion at which the
answer is *illegal*: the periodic held-target check of S-0309 claim 29, in its turn of a tick
whose tick number is a multiple of 16, for a building also a processing of its *attack*
behaviour (S-0215 claim 32), and for a healer also its firing step, where its own reply to
*illegal* drops the target by S-0310 claim 11's reply table. That table keeps a target of the
kind the healer heals that is below full health and still gives *illegal*, for instance by a
versus value of exactly 0, so for such a target the firing step is no such occasion and the
periodic check drops it. An actor that drops it at the periodic check stops the attack at its attack
behaviour's next processing after the drop." Its sentence "So the target is
dropped at the first check of the attack when none of those answers holds then, and otherwise
at the first check after they have ended (for the countdown,
after it has run out)." becomes "So the target is dropped at the first such occasion at which none
of those answers holds. For a vehicle or infantry unit that takes a checking turn in every tick,
that is 1 to 16 ticks after the tick in which the order is applied (orders are applied after the
turn pass, S-0203 claim 7) when none holds then, and otherwise the first tick whose tick number is
a multiple of 16 after they have ended (for the countdown, after it has run out); a healer can
drop it earlier, at its firing step, where its reply table drops it. A building,
which processes *attack* every tick or every other tick (S-0215 claim 32), normally drops it at
that processing first." In scenario V1, run 4b's expected "drops it at the first check after that"
becomes, for its vehicle and infantry defenders, "drops it at the first tick after that whose tick
number is a multiple of 16", with the tick numbers recorded. Level: high for the drop at the first
such tick when no higher answer holds, for vehicle and infantry actors; medium for the part that
rests on the ranks (*rearming*) for vehicle actors, which claim 29 here rates medium, low for that
part for infantry actors, as claim 29 here rates the rank-5 keep for an infantry holder, and medium
for aircraft and building actors, including a building's drop at its own processing (claim 29
here rates building holders medium), and medium for actors of computer-controlled owners, with
the earlier drop of an allied target and its exceptions (claim 29 here, run p9a, and claim 25
part k here, run p9b, for the drop of a target the actor picked itself; the drop of a target
from an order `unmeasured`, since scripted input reaches only the human's objects and no part
here orders a computer-controlled actor; the exceptions `unmeasured`, as claim 25 part k
gives), and
the healer's firing-step drop, and its keep row's target left to the periodic check, at the levels
claim 29 here gives for them (S-0310 claim 11's for vehicle healers; medium, `unmeasured`, for
infantry healers). Backing: `measurable` (scenario
A6 part p here, runs p1 and p2,
which reuse V1 runs 4 and 4b with tick numbers, for vehicle and infantry actors, and in which a drop
off a multiple of 16 before the expected tick falsifies the cadence). For buildings, the drop at the
building's own processing rests on S-0215 claim 32's backing, and the periodic check for a building
or an aircraft is `unmeasured` (medium) as claim 29 here gives. Claim 2's Confidence line "follows
the attack behaviour's handling of an illegal target and has not been observed" becomes "follows
S-0309 claim 29's periodic check, and for a building also S-0215 claim 32's attack processing", and
its Open questions line "Claim 2's handling after the force-fire order (dropping the target at the
first check that answers *illegal*) should be checked against S-0215's attack behaviour, for
vehicles and infantry as well as buildings." is removed, since claim 29 here and S-0215 claim 32
give that handling for every holder kind that claim 29 here covers. Level: as given just above for
the parts of claim 2. Backing: as given just above.

S-0306 claim 23: its text, its Backing and two items of its Confidence line leave to S-0309
whether another rule drops earlier a held target that its first row refuses. Claim 29 here
gives that drop. The quoted lines become:
- Its text, "For a building the drop is rated as S-0215 claim 32 rates it, and for a unit low
  (`unmeasured`). Whether another rule can drop such a target earlier is an open question of
  S-0309.", becomes "For a building the drop is rated as S-0215 claim 32 rates it. A unit that
  S-0309 claim 29 covers drops it at that claim's periodic check, no later than its first
  checking turn at the first check tick in which its turn comes after the refusal begins, as
  that claim's Phase rule gives. Other drops at the start of a turn are listed in S-0309's Open
  questions (*Other drops at the start of a turn*)." Level: medium for a vehicle holder in
  *attack* or *guard* that takes a checking turn in every tick, for the drop no later than that
  turn, and low for other units. Backing: `measurable` for that vehicle (scenario A6 part p run
  p4 here, in *attack*, and part m, in *guard*), `unmeasured` for other units (no part here
  gives one a target that lifts off).
- Its Backing's "but cannot tell the first row's refusal from an earlier drop by another rule,
  an open question of S-0309" becomes "but does not judge the tick of the drop; S-0309 scenario
  A6 part p run p4 and part m judge it for a vehicle holder", and its "`unmeasured` for a unit's
  deadline for the drop (no later than the first *illegal* answer): part i does not judge the
  tick of the drop" becomes "`measurable` for a vehicle holder's deadline for the drop, through
  S-0309 scenario A6 part p run p4 and part m, and `unmeasured` for other units, which no
  scenario of either spec gives a target that lifts off". Level and backing: as in the items of
  its Confidence line below.
- Its Confidence items "low: that the first row is what refuses it, rather than another rule
  dropping it earlier (part i cannot tell the two apart, see the backing)" and "low: a unit's
  deadline for that drop, no later than the first *illegal* answer (`unmeasured`)" become "medium:
  that the first row's refusal is what drops it, at S-0309 claim 29's periodic check, for a vehicle
  holder in *attack* (S-0309 scenario A6 part p run p4); low for other units, and for a vehicle
  holder in *guard*, whose drop S-0309 scenario A6 part m allows at a search too" and "medium: a
  vehicle holder's deadline for that drop, S-0309 claim 29's Phase rule, in *attack* or *guard*
  (S-0309 scenario A6 part p run p4 and part m); low for other units (`unmeasured`)". Level: as each
  item gives. Backing: `measurable` for the vehicle holder in the runs each item names, `unmeasured`
  for the low items.

S-0215 claim 32: no change needed. A building in *attack* still drops a refused target at its
own processing; for a building that claim 29 here covers, the periodic check can come first.
S-0215's behaviours other than *capture* and *sabotage*, and the economy specs' behaviours,
could point to claim 29 here for the drop of a held target that the firing check refuses: a
holder in any of them makes the check, whether or not it aims a weapon at the target. No rule
of theirs needs to change. Level: medium (claim 29 here). Backing: `measurable` for *area guard*
and *move* (scenario A6 part p, run p8, here), `unmeasured` for the others, as claim 29 here
gives.

S-0516 claim 14 and scenario CS9 part c: a holder that claim 29 here covers and that keeps a
cloaking target under that claim's exceptions can lose it at claim 29's check here, at the first
tick whose tick number is a multiple of 16 at which the firing check refuses it, for example by
S-0516 claim 24's hidden-target refusal. In CS9 part c the refusal cases therefore end with that
drop. Its sentence "a lost target makes that trial inconclusive" can say that a loss at such a tick
in a refusal case is expected, and that shot creation is judged in the turns before it. Level:
medium (claim 29 here, for a refusal by rank). Backing: `measurable` (that part, with tick numbers).

S-0501 (the aircraft's take-off): "An aircraft that is taking off keeps its target unchanged
until its height reaches half its cruise height" is a rule of its movement, and claim 29 here
does not change it. An aircraft that claim 29 here covers can still lose a target that the
firing check answers *illegal* or *impossible* for at claim 29's check in that time, at a tick whose
tick number is a multiple of 16. The sentence gains that exception: "An aircraft that is taking off
keeps its target unchanged until its height reaches half its cruise height, except that S-0309 claim
29's periodic check can drop it in that time." Level: medium (aircraft holders, claim 29 here).
Backing: `unmeasured` (no part here gives an aircraft that is taking off a refused target).

S-0310 claim 11 and scenario T10 part d: for a healer that claim 29 here covers (T10's healers
hold targets from orders), the reply "keeps it and does nothing in this step" lasts only to
claim 29's next check here: at the first tick whose tick number is a multiple of 16 at
which the firing check answers *illegal* (no higher-ranked answer holding),
the healer drops the target. T10 part d's expected "it holds
the target and does nothing" holds only up to that tick, and its falsifier "(d) drops it" should
except a drop at such a tick. Level: medium. Backing: `measurable` (that part, with tick numbers).

S-0310 claim 11 and scenario T10 parts g and h: its sentence "What the infantry unit does with
the answer belongs to its attack behaviour (S-0215), not to this spec." becomes "An infantry unit
whose default weapon is a healing weapon replies to *illegal* in its own firing step, in each of
its turns, by the table above with *infantry* in place of *a vehicle in the glossary sense*: it
drops a target that is not infantry or is at full health, and keeps an infantry target below
full health and does nothing in that step." So T10 parts g and h stand unchanged: claim 29 here
does not delay them, since that firing step drops the vehicle, and the infantry target once it
is at full health, in the healer's first turn in which the answer holds, and the infantry
healer's *illegal* outranks *rearming*. Its keep of an infantry target below full health that
still gives *illegal* (for instance by a versus value of exactly 0) lasts only to claim 29's
next check here, as for the vehicle healer of part d. Its Confidence item "low (that the
infantry answer shows through its reply as S-0215 gives it)" becomes "medium, `unmeasured` (the
infantry healer's reply in its firing step; not measured: no part sets up the wrong-kind or
full-health cases with the order accepted)". Level: medium. Backing: `unmeasured` (not
measured: no part sets up the wrong-kind or full-health cases with the order accepted; T10 parts
g and h are evidence only if their orders are accepted, as S-0310 claim 11's Backing gives).

S-0513 claim 39: a warp member's *illegal* answer
from the launch until its departure update makes every holder that claim 29 here covers and that
takes its checks drop
it at its first checking turn in a check tick after the launch, unless an exception of claim 29
holds there. Such a holder holds the member without a break from the launch to the departure
only if an exception held at each of its checking turns in the check ticks among the 62 ticks
that follow the launch tick, in the last of them those of its turns that come before the
member's update (S-0513 claim 39). Level:
medium (the rank of that answer, S-0301's, and claim 29's exceptions). Backing: `measurable` (S-0513
scenario SW12 parts i and j, for infantry and vehicle holders).

S-0513 claim 39, its scope sentence: "That claim gives its rules for holders of
human-controlled owners with a target from an order or a search, which in our engine are the
holders of every player house, and leaves the original's other holders and targets to its Open
questions (low); so does this claim." becomes "That claim gives its rules for holders of every
owner, in every behaviour but *capture* and *sabotage*, whatever gave their target; in our engine
those are the holders of every house. A holder of a computer-controlled owner whose target is
allied with it drops it earlier, at the check's place in its turn (S-0309 claim 29 *Place in
the turn*, claim 25 part k), unless an exception of that part holds." Level: as claim 29 here
rates each case: medium for holders of
computer-controlled owners, for targets from retaliation or a revert and for behaviours other
than *attack* and *guard*, and claim 25 part k's levels for the allied drop. Backing: as claim 29
here gives (`measurable`, scenario A6 part p here, runs p8 and p9a, and run p9b for the
allied drop; `unmeasured` for a retaliation's or a revert's target and for the other
behaviours).

S-0513, its Changes for other specs item for S-0309 claim 29: its words "a holder in *attack* or
*guard* drops a target" become "a holder in any behaviour but *capture* and *sabotage* drops a
target", and its sentence "Its rules are given for holders of human-controlled owners (in our
engine, of every player house) with a target from an order or a search; a holder in *capture* or
*sabotage* makes no check, and whether a holder in another behaviour makes it is left open
there." becomes "Its rules are given for holders of every owner, whatever gave their target, and
a holder in *capture* or *sabotage* makes no check." Level and backing: as for the claim 39
sentence above.

S-0513 claims 48, 51 and 52 and S-0203 claim 4: a
warp member's own check as a holder is claim 29's *Warp members* rule here. In the original it
is not settled here: S-0513 claims 48 and 52 own what is established about the ordinary work of
a member's warp turns. Our engine's warp turn contract (S-0513 claim 51) lists this check among
the ordinary work it suppresses from the first pending update through the arrival effects, and
S-0203 claim 4's list of that work gains it: "the periodic held-target check (S-0309 claim
29)". Level: low for the original, whose handling of this check in those turns is unsettled, high as
design for our engine's suppression. Backing: `unmeasured` for the original (no part here gives a
warp member a refused held target at a check tick), `measurable` in our engine (S-0513 scenario SW23
part f).

## Questions for the project lead

No open questions. Recorded decisions and settled points:

- S-0215 claim 33 and the attack-move search: settled from the original's behaviour (this spec's
  claims 1, 23 and 24 were right). S-0215 claim 33 points here for the attack-move search and
  for which targets are opportunity targets.
- Neutral and special houses (project lead, 2026-10-10, afternoon, item 2): decided; applied in
  Terms (*Houses in our engine*) and claim 25, and in S-0215 (Terms and claim 18).
- S-0215 claim 26 backing: S-0215 claim 26 points to claim 12 here for the search's draws.
