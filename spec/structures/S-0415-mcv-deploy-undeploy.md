---
id: S-0415
title: Deploying a vehicle into a building and packing it up again (construction yard vehicles and other deploying vehicles)
topic: structures
revision: 4
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: [Q-0132]
supersedes: []
---

# S-0415: Deploying a vehicle into a building and packing it up again (construction yard vehicles and other deploying vehicles)

## Summary

What a vehicle whose type deploys into a building does when it carries out a deploy order (conditions, anchor cell, turn to the deploy facing, timing, what the building takes over, refusal), and the reverse: a building whose type undeploys into a vehicle packing up, where the vehicle appears and what it takes over. Skirmish only; campaign and network behaviour is out of scope. S-0213 (claims 10 and 12) owns the orders and when a building packs up instead of being sold, and S-0214 owns when input offers the deploy action; this spec owns what follows.

## Terms

Shared terms used here are defined in the glossary: *house*, *vehicle*, *building*, *type*, *field*,
*cell*, *lepton*, *facing*, *foundation* (with its *top-left cell*), *tick*, *behaviour*, *on the map*,
*in play*, *removed*, *construction state*, *health*, *strength*, *health ratio*, *experience*,
*target*, *local player*, *AI house*, *computer-controlled*, *human-controlled*, *scenario random
stream*, *absent-key default*, *quirk switch*. S-0205's notation applies (trunc, RZ₅₃); S-0204's *n-step facing* (Formulas F4) is used with *n* = 256. S-0214's
*deploy readiness* (Terms), S-0213's *pack up* and *guard spot* (Terms, claim 20) and S-0406's
*cell test* and *foundation rectangle* (Terms, claim 3) are used as those specs define them. So are
S-0215's *slave-master* and its *slaves*, S-0214's *sell mode*, S-0414's *base units* and its
*insignificant* and *not scored* marks, and S-0011's *tag* (Terms), here called the object's *trigger
tag*. S-0215's *current behaviour*, *queued behaviour*, "queues" (Queueing wording), *processing timer*
and *processing delay* (Terms) are used as it defines them; a *destination* is the place that a mobile
vehicle's movement is heading to (S-0207). In addition:

- **Deploying vehicle.** A vehicle whose type names a building type it deploys into (Import
  mapping). It is not S-0215's *deployer*, a type that deploys in place.
- **Deploy facing.** A building type's integer field *D*, normally from 0 to 255 (other values: Edge
  cases and quirks), in steps of 1/256 of a turn
  (Import mapping); the facing *D* × 256 in the glossary's facing units.
- **Deploy anchor.** The cell that becomes the top-left cell of the deployed building (claim 3).
- **Packing building.** A building whose type names a vehicle type it undeploys into (Import
  mapping).
- **Construction yard.** In this spec, as in S-0213 and S-0214: a building whose type has the
  construction-yard flag (Import mapping). This may be a different set of types from S-0215's
  *construction yard*, which is defined by a list of the computer player's.
- **Abandoned turn.** A turn to the deploy facing (claim 5) that a later attempt gives up.
- **First attempt, later attempt.** The *first attempt* is the first deploy attempt a vehicle makes
  after its deploy order, which is made before any turn. A *later attempt* is one made after a turn has
  begun (claim 5), including the attempts that repeat after a failed placement during a turn (claim 6).
- **Still.** A vehicle is *still* when it is not moving along the ground. Turning in place does not
  count as moving.
- **Sale refund.** The amount a sale of the building would pay its house (S-0408, claim 1).
- **Return cell.** The cell where the packed-up vehicle is offered a place on the map (claim 10,
  F3). This is not a search for an exit cell.
- **Entry outcome.** S-0209's answer for a vehicle and a cell (Terms, claim 5), including its
  terrain checks. Claim 12 uses that answer, not the substitutions for starting a movement step
  in S-0209 claim 5f.
- **Redeploy setting.** S-0412's setup value (a lobby check box) that says whether a deployed base
  unit may pack up again. S-0213 (claim 12) and S-0214 (Terms, *movable*) call it "the game option
  that lets construction yards be packed up again". It is one of S-0213's pack-up conditions for
  a construction yard and does not matter for any other packing building.
- **Refusal announcement.** The stock spoken announcement that the deploy cannot be done there
  (Import mapping, last row). It is spoken only to the local player.

## Behaviour

### Deploying

1. **Where deploying happens.** A deploy order gives a deploying vehicle the *unload* behaviour (S-0213,
   claim 10). Each processing of *unload* by a deploying vehicle makes at most one deploy attempt (claims 2 and
   5), and the processing ends with the standard delay of S-0215 (claim 6: the *unload* rate delay plus one draw
   over 0 … 2 from the scenario random stream), whatever the attempt's outcome. The first attempt is
   made at the first processing after the order applies. A deploying vehicle in *hunt* also seeks a
   spot and deploys (S-0215, claim 27); the attempt itself then follows claims 3 to 6.
   The *unload* of a vehicle follows this spec only when no other *unload* rule takes precedence.
   The first rank whose condition holds decides what *unload* does:

   | Rank | Condition | What *unload* does |
   |---|---|---|
   | 1 | the vehicle is not bunkered and its type has a passenger capacity above 0 | unloads its passengers (the transport rules) |
   | 2 | its type has the deploy-fire flag and not the simple-deployer flag | fires its deploy weapon (S-0215) |
   | 3 | its type has the harvester or the weeder flag | follows the harvesting rules (S-0402) |
   | 4 | its type deploys into a building | a deploy attempt (this spec) |

   A type that deploys into a building and has one of the properties of ranks 1 to 3 therefore
   never deploys through *unload*. The importer warns about such types.
   - Backing: `measurable` (scenarios D1 and D9); the bunkered condition of rank 1 is `unmeasured`
     (scenario D9, Not covered). The delay draw at the processing that completes a deploy is
     `unmeasured`: the vehicle is removed in that tick, so its processing timer cannot be read, and
     the bystander's draws keep the draw log from attributing the draw to it (Measurement scenarios)
   - Confidence: high (attempts at each processing); medium (the first-processing timing); low (the
     precedence of the other *unload* rules)

2. **A deploy attempt.** Before a turn has begun, a moving deploying vehicle makes no attempt: it
   keeps *unload*, unless a queued behaviour other than *unload* is waiting, which then takes over.
   A still vehicle makes the attempt. The attempt fails without a turn when the vehicle lacks deploy
   readiness (S-0214, Terms) or has a destination, or when the cell test fails (claim 3) or the
   building cannot be put on the map (claim 6). After a failed first attempt (the one made before any
   turn) *guard* becomes the vehicle's queued behaviour; only the cell-test failure plays the
   refusal announcement (claim 4). An attempt that neither fails nor completes (claim 6) is a turn
   (claim 5). Once a turn has begun, claim 5 governs every later processing, a moving vehicle
   included.
   - Backing: `measurable` (scenarios D1 and D2; D2 part k for a first attempt without deploy
     readiness; D2 part f for what happens during a turn). The failure of a first attempt for a
     destination is `unmeasured`: it needs a vehicle that has a destination and is still at the first
     processing after the order, and no scripted input is known that leaves a vehicle in that state
     (a vehicle given a destination starts to move; Open question 1). The rule that a moving
     vehicle makes no attempt is `measurable` (D2 part m).
   - Confidence: medium (the attempt, the failures); low (the moving rule, which part m can show only
     when a deploy order leaves a moving vehicle's destination in place)

3. **The anchor and the cell test.** The deploy anchor is the deploying vehicle's cell when the building's
   foundation is at most 2 cells along map *x* and at most 2 along map *y*; otherwise it is the cell
   one step back along both map axes (*x* − 1, *y* − 1). The building must pass S-0406's cell test
   (claims 3 and 8) at that anchor, with these differences:
   - the deploying vehicle itself does not count as being in the way,
   - the wall exceptions of S-0406 claim 8, which ask that the wall's owner be the placing house,
     here apply only to wall cells that have no wall owner at all, never to the deploying house's
     own walls (`quirks.deploy_refuses_own_walls`),
   - neither adjacency nor the shroud (S-0406, claims 4 and 5) is asked, nor any tech-tree condition
     or build limit of the building type (S-0407).
   - Backing: `measurable` (scenario D2: parts a and b for the anchor, c for adjacency, d for the tech
     tree and build limit, i for the shroud, g and h for the walls)
   - Confidence: high (the anchor, the cell test, no adjacency); medium (the wall exception, no
     shroud condition)

4. **Refusal.** When the cell test fails, the deploy is refused: when the deploying vehicle's house is the
   local player's, the refusal announcement plays, unless the deploying vehicle's type gathers resources
   (Import mapping). The deploy attempt ends the vehicle's *unload*: *guard* is queued and is the
   vehicle's current behaviour by its next processing (Open question 3); it stays where it is. A new
   deploy order is needed to try again.
   For AI designers: the original's computer-controlled houses instead give a deploying vehicle whose
   first attempt failed for any reason (claim 2) the behaviour *hunt*, with which it looks for a new
   spot by its own rules. The same holds for the two non-player houses (neutral and special),
   which keep the original's rules for houses that are not human-controlled: a deploying vehicle of
   theirs, which only a map trigger can start, goes to *hunt* too. In our engine AI houses follow the
   rule above and may give their own move order.
   - Backing: `measurable` (scenario D2: parts a and b for the refusal, part e for the
     sound; part j for the *hunt* of a bystander's vehicle after a refused deploy; part k for the
     *hunt* after a first attempt without deploy readiness). The *hunt* after a failed first attempt
     for a destination is `unmeasured` for the reason given in claim 2.
   - Confidence: high (the refusal and the sound); medium (*guard*); medium (the *hunt* of the
     original's computer-controlled houses); low (the same for the neutral and special houses, which
     no scenario covers: no map of the measurement setup gives them a deploying vehicle)

5. **The turn to the deploy facing.** When the cell test passes but the deploying vehicle's body facing,
   rounded to 256 steps (F1), differs from the building type's deploy facing *D*, the deploying vehicle turns
   in place towards facing *D* × 256 at its turn rate (S-0206), and the attempt ends. The vehicle is
   then *turning*. Each of its later *unload* processings has the outcome of the one row below that
   matches the vehicle's situation at that processing:

   | Situation at the processing | Outcome |
   |---|---|
   | it has a destination, moving or not | no deploy; the turn is abandoned, with no announcement |
   | no destination, no deploy readiness | no deploy and nothing else changes: the turn is not abandoned, and the vehicle deploys at the first later processing at which it has deploy readiness again and the other rows allow it (low confidence) |
   | no destination, deploy readiness, cell test fails | refused as claim 4 says (*guard* queued); the turn ends |
   | no destination, deploy readiness, cell test passes, rounded facing ≠ *D* | the turn goes on; it is not restarted |
   | no destination, deploy readiness, cell test passes, rounded facing = *D* | the deploy completes (claim 6); when the building cannot be put on the map, nothing changes and the next processing is judged by this table again (claim 6) |

   So a destination abandons the turn even when the vehicle lacks deploy readiness or stands where
   the cell test would fail, and a vehicle without deploy readiness is never refused. Only a
   destination abandons a turn. A refusal ends it too, with the announcement where claim 4 gives
   one.
   After an abandoned turn, at its next *unload* processing the vehicle makes no further attempt: its
   queued behaviour, if it has one, becomes current. When it has none, the working hypothesis is that
   nothing else happens, so the vehicle stays in *unload* and does nothing at each later processing
   until it is given another order (Open question 2).
   - Backing: `measurable` (scenario D1; scenario D2, part f: (1) for the lost readiness, (2) for
     the destination and what follows the abandonment, which can be told apart from a mere pause
     of the attempts because the vehicle stands elsewhere afterwards)
   - Confidence: high (the turn, the target facing); medium (the abandonment by a
     destination); low (what follows an abandoned turn; the continuing turn after a lost readiness)

6. **Completion.** When the cell test passes and the rounded facing equals *D*, the deploy completes
   in that attempt:
   - a building of the deploys-into type, owned by the deploying vehicle's house, is put on the map at the
     centre of the anchor, (256*x* + 128, 256*y* + 128) leptons, standing on the ground layer at height 0
     above the terrain of that cell, and its behaviour becomes *construction*: its construction state
     begins (S-0108). A cell that holds a bridge overlay never passes the cell test (S-0406,
     claim 3 c), so no building is put on a bridge deck. A cell that a high bridge only marks, with
     no overlay in it, passes only for a water-bound type (S-0406, claim 3 d); the building is then
     still put on the ground layer at height 0 (low confidence; scenario D12, part b),
   - its health is max(1, trunc(*r* × *S*)) (F2), where *r* is the deploying vehicle's health ratio and *S*
     the building type's strength,
   - it takes over the deploying vehicle's experience (S-0311), its control group, its selection (a selected
     deploying vehicle gives a selected building), its trigger tag, and the slaves of a slave-master vehicle
     (S-0215),
   - every object whose target was the deploying vehicle gets the building as its target, except a vehicle
     thief (glossary) when the building is a construction yard: its target becomes nothing,
   - the building's own target becomes the deploying vehicle's target, if it had one,
   - the deploying vehicle type's deploy sound plays at the deploying vehicle's position (S-0107, Import mapping) and the
     deploying vehicle is removed in the same tick.
   The building is put on the map only if the anchor also passes S-0406's cell test as it applies to
   a placement by the deploying vehicle's house. That test, too, ignores the deploying vehicle itself
   and asks for no adjacency, shroud or tech-tree condition. The only input known for which it differs
   from the cell test of claim 3 is a wall with no wall owner in a foundation cell of a type that has a
   wall exception: claim 3's test lets it through, and the placement, which asks that the wall's owner
   be the deploying house, is expected to refuse it. When the building cannot be put on the map, no
   building appears and the vehicle stays as it was. When this happens in a first attempt, its
   *unload* ends with *guard* queued and no announcement (claim 2). In a later attempt no behaviour is
   queued, and the vehicle attempts again at its next processing.
   - Backing: `measurable` (scenarios D1, D3 and D4 for the completion; D12 part a for the
     position within the cell, the height and the layer, part b for the water-bound case; scenario D2,
     part g for the failure in a first attempt, part l for the failure in a later attempt and the retry)
   - Confidence: high (building, cell, construction state, health, removal); medium (the position
     within the cell and the layer, the take-overs, the retargeting, the placement height); low (the
     water-bound case; the failure after a passed cell test, including what it asks beyond the wall
     owner)

7. **What a deployed building counts as.** The new building counts for its house from the tick of
   completion as any building put on the map: as a prerequisite and a factory (S-0407, claims 1 and
   8), for adjacency (S-0406, claim 4) and for power (S-0405). The deploying vehicle stops counting for its house when it is removed, so a
   build limit on the vehicle type is free again from that tick (S-0407, claim 9). The building does
   not raise its type's ever-built count (S-0407, claim 10).
   - Backing: `measurable` (scenario D5, parts a to d); the ever-built sentence is `unmeasured`: no
     negative build limit takes effect on a building type in the original (S-0407, claim 10), so
     the count cannot be seen there, and on our engine it matters only with that claim's quirk off
   - Confidence: medium

8. **The original's computer-only marks.** For AI designers: when the deploying vehicle's house is not the
   local player's, the original marks the new building for automatic repair and rebuilding by its
   computer player, and for a construction yard of a computer-controlled house in a skirmish it
   moves that house's base centre to the yard. These marks serve only the original's computer player.
   The two non-player houses get the repair and rebuild marks as well, since they are not the local
   player's, but they have no computer player to act on them. In our engine an AI house gets none of
   them (glossary, *AI house*); its AI may repair through ordinary orders.
   - Backing: `unmeasured`: the marks act only through the original's computer player's own decisions
     (when it repairs a building, whether and where it rebuilds, where it builds next), which also
     depend on its other state and on random draws. No setup can separate the effect of a mark from
     those decisions, so the marks cannot practically be observed on their own.
   - Confidence: high (that the marks are set; not observed)

### Packing up

9. **The packing-up sequence.** A building packs up when a sell order reaches it and S-0213's
   conditions (claim 12) hold. It then goes through its sale (S-0408, claim 4) with these
   differences:
   - at the sale's first processing the building type's deploy sound plays at the building's position
     (Import mapping), whoever owns the building; when the build-down starts, and only when the
     building's house is the local player's, that player hears the type's pack-up sound when it has
     one. The sell sound never plays for a building that packs up, whether or not its type has a
     pack-up sound: a packing building whose type has no pack-up sound makes no sound at the start of
     the build-down at all,
   - its construction state runs backwards (the build-down, S-0108) as in any sale,
   - when the build-down has finished and the return cell accepts the vehicle (claim 12), the
     vehicle of claim 10 appears and the building is removed in the same tick. The house receives
     no refund. Refusal has claim 11's outcome instead.
   A building that is sold instead of packing up plays the sell sound as S-0408 gives it; when its
   type has a pack-up sound and its house is the local player's, that sound also plays at the start
   of its build-down (`quirks.packup_sound_on_sale`).
   S-0213's conditions are judged again when the build-down finishes: a construction yard that no
   longer meets them then (for example one that has become mind-controlled during the build-down)
   is sold as an ordinary building, with the sale's refund.
   No survivors, passengers or garrison occupants come out of a *packing* building that has a guard
   spot. A packing building without a guard spot (a type that is not a construction yard, sold
   through the sell mode of S-0214) releases them as in an ordinary sale before it packs up
   (`quirks.packup_release_by_guard_spot`).
   - Backing: `measurable` (scenario D6 for the sequence, refund and sounds, and for the no-release
     case of a building with a guard spot; D11 for a type with no pack-up sound; scenarios D7 and
     D10 for the release of a building without a guard spot and the sound of a sold building). Two
     parts are `unmeasured`.
     - The second judging: no scripted input is known that makes one of S-0213's conditions fail
       between the order and the end of the build-down. The one candidate is mind control, and it is
       not usable: no spec shows that a mind-control weapon can take over a building, so even a
       named bystander controller (which the void rule allows) is not known to produce that state.
       A measurement would first need a test weapon shown to capture a building (S-0215).
     - A construction yard that has a guard spot but fails another pack-up condition: the only input
       that gives a yard a guard spot needs the redeploy setting on (S-0214, Terms, *movable*), and
       no other condition can be made to fail under it, so the state cannot be set up.
   - Confidence: high (the sequence, no refund); medium (the sounds); low (the release rule; the
     second judging)

10. **The vehicle that appears.** The vehicle of the undeploys-into type, owned by the building's
    house, is put on the map at the centre of a cell: the building's top-left cell when its foundation
    is at most 2 cells along map *x* and at most 2 along map *y*, otherwise the cell one step forward along both map axes (*x* + 1, *y* + 1)
    from it, so that a vehicle that deployed and packs up again returns to its own cell. It faces the
    building type's deploy facing, *D* × 256. Its health is max(1, trunc(*r* × *S*)), where *r* is the
    building's health ratio and *S* the vehicle type's strength (F2). It takes over the building's
    experience, control group, selection, trigger tag and slaves, and every object whose target was the
    building gets the vehicle as its target. When the building had a guard spot, the vehicle is given
    that spot as its destination with the behaviour *move* (S-0213, claim 20: the move click that led
    to the pack-up). For a deploy facing outside 0 … 255 the vehicle's facing is not established
    (Open question 4; scenario D13).
    - Backing: `measurable` (scenarios D6, including its slave-master part, and D7; D13 for a
      deploy facing outside 0 … 255)
    - Confidence: high (position, facing, health, the move); medium (the take-overs); low (the
      facing for a deploy facing outside 0 … 255)

11. **A failed pack-up.** When the vehicle cannot be put on the map at its return cell (claim 12),
    the house receives the building's sale refund instead
    and the building is still removed. The house goes on owning a vehicle of the undeploys-into type
    that is never on the map, and that vehicle counts in the house's owned count for build limits
    (S-0407, claim 9) (`quirks.failed_packup_keeps_vehicle`). For victory and defeat (S-0414 claims 2
    to 4) it counts from the tick the build-down finishes, but it is never on the map. So with the
    short-game setting on it keeps its house alive when its type is among the first three base units
    (for example a construction-yard vehicle type that the rules list there), unless the type is marked
    insignificant or not scored; with the setting off it does not count (the count used with the
    setting off takes in only vehicles that have been put on the map). No game-state condition is
    known under which no vehicle results at all; this spec does not cover that case. The vehicle type that the building names always
    exists, because a building whose undeploys-into name matches no vehicle type is not a packing
    building (Import mapping).
    - Backing: `measurable` (scenario D8, including its variants for the short-game setting and
      for the two marks)
    - Confidence: medium (the refund); low (the vehicle that goes on existing)

12. **Which return cells accept the vehicle.** At the completion of the build-down, the packing
    building itself no longer occupies the map for this test. The newly created vehicle is
    accepted exactly when its entry outcome for the return cell is *clear*, using S-0209 claim 5
    with no required approach direction or source height, no bridge-deck request and no
    movement-step substitutions. The vehicle has not yet inherited the guard spot as a destination
    or the building's experience. Thus this is not a test using the building's locomotion or the
    vehicle's eventual veterancy abilities. Other outcomes, including *cloaked blocker*, *moving
    blocker*, *friendly obstacle*, *enemy obstacle* and *stationary friendly*, all refuse it.

    The following consequences assume no other blocker or special occupant override of S-0209
    claim 5, and ordinary ground-layer placement:

    | Return cell contents or condition | Result |
    |---|---|
    | own or allied vehicle standing there | refused |
    | enemy vehicle that the new vehicle cannot crush | refused, whether cloaked or visible |
    | own or allied infantry standing there, even one unit in one standing position | refused |
    | enemy infantry, all crushable by the new vehicle, with no vehicle-present flag | accepted |
    | enemy infantry that the new vehicle cannot crush | refused |
    | tree or another ordinary terrain object not crushable by the new vehicle | refused, even when its weapon can damage that object |
    | wall the vehicle cannot crush | refused, even when its weapon can damage walls |
    | wall the vehicle can crush, owned by an allied house | refused |
    | wall the vehicle can crush, not owned by an allied house | accepted when the terrain checks also pass |
    | resource overlay on land with a nonzero multiplier for the vehicle's locomotion class | accepted |
    | land with a zero multiplier for that class, such as water for a class forbidden there | refused |

    Enemy crushable vehicles and terrain objects, movement reservations and exceptional buildings
    follow the same entry-outcome tables rather than a separate blanket refusal by object kind.
    The refusal does not ask nearby cells, wait for an occupant to move, request it to scatter,
    attack it or retry later. A clear neighbour does not rescue the pack-up. The only completion
    outcome of refusal is claim 11's refund, building removal and retained off-map vehicle under
    the default quirk setting.

    Q-0132's general vehicle and infantry entry outcomes were already covered by S-0209 claim 5.
    The refund was already covered by claim 11 here. This claim supplies the missing connection
    between those outcomes and pack-up placement, and the absence of an alternative placement.
    - Backing: `measurable` (scenario D14); special entry-outcome cases retain the backings and
      open questions of S-0209 and S-0211 rather than gaining new claims here
    - Confidence: high (only *clear* accepts, the packing building does not block, no alternative
      cell or retry); medium (the listed consequences, pending D14)

## Formulas

> **F1. Rounded body facing.** For a body facing *f* (0 … 65535), the rounded facing is
> *q* = ⌊(*f* + 128) / 256⌋ mod 256: *f* / 256 rounded half up, then reduced modulo 256.
> Integers. This is S-0204's 256-step facing (Formulas F4, *n* = 256). The deploy completes when
> *q* = *D*.

> **F2. Health handed over.** *h* = max(1, trunc(RZ₅₃(*r* × *S*))), where *r* = health ÷ strength of
> the object that is replaced, computed in double precision as S-0214 gives it (Formulas F2), and *S* is the
> strength field of the new object's type (an integer). The product is formed in floating point at
> 53-bit precision and truncated toward zero (S-0205). Hit points, integer.

> **F3. Anchor and return cell.** With the deploying vehicle in cell (*x*, *y*) and the building's foundation
> *W* × *H*: anchor = (*x*, *y*) when *W* ≤ 2 and *H* ≤ 2, else (*x* − 1, *y* − 1). The packing
> vehicle's cell is the building's top-left cell when *W* ≤ 2 and *H* ≤ 2, else that cell + (1, 1).
> All values are integer cell coordinates and sizes in cells; exact integer arithmetic, so no
> rounding applies.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Deploy facing absent-key default | 128 | 1/256 turn | facing 32768 |
| Size above which the anchor moves | 2 | cells along either map axis | claim 3, F3 |
| Anchor step | (−1, −1) | cells | claim 3 |
| Return step | (+1, +1) | cells | claim 10 |
| Placement height | 0 | leptons above the terrain of the anchor cell | claim 6 |

## State and ordering

| Vehicle state | Event (at an *unload* processing) | Result |
|---|---|---|
| moving, no turn begun | processing | no attempt; *unload* goes on, unless a queued behaviour other than *unload* takes over |
| first attempt, without deploy readiness or with a destination | attempt | fails: *guard* queued, no announcement |
| first attempt, ready, still | cell test fails | refusal (claim 4): *guard* queued, announcement |
| first attempt, ready, still | cell test passes, rounded facing ≠ *D* | turn towards *D* × 256 (claim 5) |
| first attempt, ready, still | cell test passes, rounded facing = *D* | completion (claim 6) without turning |
| first attempt, ready, still | cell test passes, rounded facing = *D*, but the building cannot be put on the map | no building; vehicle as it was; *unload* ends with *guard* queued, no announcement (claims 2 and 6) |
| turning, no destination, ready | cell test passes, rounded facing ≠ *D* | the turn goes on; it is not restarted (claim 5) |
| turning, no destination, ready | cell test passes, rounded facing = *D* | completion (claim 6): building in play, deploying vehicle removed |
| turning, no destination, ready | cell test passes, rounded facing = *D*, but the building cannot be put on the map | no building; vehicle as it was; no behaviour queued; attempts again at its next processing (claim 6; low confidence) |
| turning | has a destination (moving included) | abandoned turn; at the next processing no attempt, the queued behaviour (if any) becomes current, otherwise *unload* stays and does nothing (claim 5) |
| turning, no destination | attempt, without deploy readiness | the attempt fails, nothing else changes; the turn goes on and the next processing attempts again (claim 5; low confidence) |
| turning, no destination, ready | attempt, cell test fails | refusal (claim 4): *guard* queued, the turn ends |

"First attempt" and "later attempt" are as in Terms; *turning* means a turn has begun and has not ended in a completion, an abandonment or a refusal. A vehicle that is turning in place is still, but it may have a destination.

| Packing building | Event | Result |
|---|---|---|
| in play | sell order with S-0213's pack-up conditions | sale begins; deploy sound at its first processing; at the start of the build-down the pack-up sound, if the type has one, for the local player only; no sell sound (claim 9) |
| selling | build-down finished, S-0213's conditions still hold, vehicle can be put on the map | vehicle appears (claim 10); building removed; no refund |
| selling | build-down finished, S-0213's conditions still hold, vehicle cannot be put on the map | refund (claim 11); building removed |
| selling | build-down finished, S-0213's conditions no longer hold | sold as an ordinary building, with the sale refund (claim 9) |

Observable consequences: the deployed building and the removal of the deploying vehicle happen in the same
tick; so do the vehicle's appearance and the building's removal. A deploying vehicle that already faces *D*
and stands on a valid anchor completes at its first *unload* processing, without turning.

## Edge cases and quirks

- `quirks.packup_release_by_guard_spot` (default on): claim 9. The original decides whether a packing
  building releases its survivors and occupants only by whether it has a guard spot and an
  undeploys-into type, while the pack-up itself needs all of S-0213's conditions. So a packing
  building sold through the sell mode releases its crew and still packs up (scenario D7), and a
  construction yard that has a guard spot but fails another pack-up condition is sold with no
  survivors (`unmeasured`, claim 9). Off: the release is skipped exactly when the building packs up.
- `quirks.failed_packup_keeps_vehicle` (default on): claim 11. Off: the vehicle that cannot be put on
  the map does not remain (the refund is still paid, as in claim 11), and it does not count for build limits or for a house's
  survival in the short-game test (S-0414 claim 2).
- **A deploying vehicle on an unsuitable cell is never moved** by the deploy itself (claim 4); the original's
  computer player moves its own (a note for AI designers only). No switch.
- `quirks.deploy_refuses_own_walls` (default on): claim 3. A deploying vehicle cannot deploy onto its
  own house's wall even where a placement by place order could (a wall tower deploying onto a wall),
  yet a wall cell with no wall owner passes the cell test and then fails the placement (claim 6).
  Off: the cell test of the deploy asks for the deploying house's ownership of the wall as a
  placement by that house does (S-0406, claim 8): the house's own walls pass where a placement
  could, and a wall with no owner refuses. No stock deploying vehicle is concerned; a mod with a
  wall-tower deploying vehicle may want it off.
- **Deploy facing outside 0 … 255** (claims 5, 6 and 10): the rounded facing of F1 never equals such a
  value, so the vehicle never deploys. The facing it turns towards is *D* × 256 reduced modulo 65536
  as a working hypothesis (arithmetic on facings wraps modulo 65536, glossary *facing*), and the facing of a vehicle that a
  packing building of such a type releases is not established either (Open question 4; scenario
  D13). No switch; the importer warns.
- `quirks.packup_sound_on_sale` (default on): claim 9. A building type with a pack-up sound that
  is sold instead of packing up plays both the sell sound and its pack-up sound, the second only
  when its house is the local player's (S-0408 owns the sale). Off: a building that is sold plays only the sell sound; the pack-up
  sound is for a building that packs up.

## Import mapping

| Key (file, section) | Concept | Form | Absent |
|---|---|---|---|
| `DeploysInto` (rules, vehicle type) | the building a deploying vehicle deploys into (claims 1 to 6) | building type name | none (not a deploying vehicle) |
| `UndeploysInto` (rules, building type) | the vehicle a packing building packs up into (claims 9 to 11) | vehicle type name | none (not a packing building); a name that matches no vehicle type also makes it not a packing building (claim 11) |
| `DeployFacing` (rules, building type) | deploy facing *D* (claims 5 and 10) | integer, 1/256 turn | 128 |
| `Strength` (rules, type) | strength *S* in F2 | integer, hit points | S-0302 |
| `Foundation` (art, building type; S-0108) | foundation size in F3 | word | `1x1` |
| `ResourceGatherer` (rules, vehicle type) | no refusal announcement (claim 4) | boolean | no |
| `Passengers`, `DeployFire`, `IsSimpleDeployer`, `Harvester`, `Weeder` (rules, vehicle type) | other *unload* rules that come first (claim 1) | integer, then booleans | 0, no, no, no, no |
| `ConstructionYard` (rules, building type) | the building is a construction yard (claim 6, vehicle thieves; S-0213's pack-up conditions) | boolean | no |
| `EVA_CannotDeployHere` (eva, announcement entry) | the refusal announcement (claim 4); which sound it plays is for the sound specs | entry name | none (no announcement) |
| `DeploySound` (rules, type) | deploy sound of the deploying vehicle (claim 6) and of the packing building (claim 9) | sound name | none |
| `PackupSound` (rules, building type) | pack-up sound (claim 9) | sound name | none |
| `SpeedType`, `MovementZone`, `Locomotor` (rules, vehicle type) | the new vehicle's locomotion class, movement zone and locomotor used by the return-cell test (claim 12) | names; locomotor is a class identifier | S-0211 and the movement specs own their defaults; no pack-up-specific replacement |
| `Crusher` (rules, vehicle type), `Crushable` (rules, occupant or overlay type) | crushing eligibility in the return-cell entry outcome (claim 12) | booleans | S-0210 owns the defaults; no pack-up-specific replacement |
| `Wall` (rules, overlay type) | the return cell's wall classification (claim 12) | boolean | no, as S-0209 and S-0211 |
| `Foot`, `Track`, `Wheel`, `Hover`, `Float`, `Amphibious`, `FloatBeach` (rules, land-type sections) | land multipliers used by the return-cell test (claim 12) | decimal or percent, capped at 1.0 (S-0211) | S-0211 claim 2 and Import mapping; no pack-up-specific replacement |

## Measurement scenarios

Measurement setup (glossary); a tiny purpose-made skirmish map, one human house and one bystander
computer player (glossary), named test rules files in the map's override sections, scripted input,
and a recorder (glossary) logging per tick each object's type, owner, cell, body facing, health and
behaviour, plus the local player's sounds and, where a scenario says so, a named object's processing
timer (S-0215, Terms: a read of the game's state, as the recorder of the glossary allows), an
object's position in leptons, height above the terrain and cell layer, and the random-state
recorder's draw log of the scenario random stream. The bystander computer player may draw from the
scenario stream, so no scenario here identifies a vehicle's processings from the draw log. All objects below belong to the human house unless stated.
Observed area, unless a scenario says otherwise: the 15 × 15 cells centred on the test vehicle's cell.
The bystander starts at the start position farthest from it and owns no object there; the void rule
of the glossary applies, and a void run is repeated. A run lasts until the last observable of the
scenario has been recorded. Where a scenario's outcome does not depend on the random stream, the runs
are repetitions that guard against void runs.

**D1. Deploy timing and the turn.**
- Setup: a construction-yard vehicle on open clear ground, in two variants: (a) facing the deploy
  facing, (b) facing the opposite direction; deploy order given at a known tick; a test *unload*
  rate.
- Runs: 12 per variant, each with a different random seed (the delay between processings is random,
  claim 1). The recorder logs the vehicle's processing timer per tick. A tick on which the timer
  has just been set to a new positive value is a processing tick of the vehicle (S-0215, claim 2).
- Observable: the tick the building appears; the vehicle's facing per tick; the vehicle's processing
  ticks; the local player's sounds.
- Expected: (a) the facing never changes and the building appears at a processing tick, the first one
  after the order applies; (b) the vehicle turns to *D* × 256 and the building appears at the first
  processing tick at or after the tick the turn ends; in both the deploy sound plays in the tick the
  building appears. In (b) no processing tick between the order and the end of the turn produces a
  building.
- Falsified if: (b) deploys without turning, or (a) waits for a turn, or the building appears at a
  tick that is not a processing tick, or (for the medium-confidence timing) the first attempt of (a)
  is at a later processing than the first after the order. A run in which the timer cannot be read
  is void.

**D2. Anchor, cell test, refusal and the abandoned turn.**
- Setup: a test vehicle that deploys into a 3 × 3 test building (the plain 3 × 3 foundation word of
  S-0108, all 9 cells), with its facing already equal to *D*; a second test vehicle that deploys into
  a 2 × 2 building. One obstacle (a tree) per run.
- Runs: 3 per case below (the outcome does not depend on the random stream; the repetitions guard
  against void runs).
- Part a (3 × 3): the tree at (*x* + 1, *y* + 1), at (*x* − 1, *y* − 1) and at (*x* + 2, *y* + 2),
  relative to the vehicle's cell.
- Part b (2 × 2): the tree at (*x* + 1, *y* + 1) and at (*x* − 1, *y* − 1).
- Part c (no adjacency, no ground resources): a run far from every other building of the house, and a
  run where the vehicle's anchor cell holds ore.
- Part d (no tech-tree conditions): the building type has a prerequisite the house does not meet and
  a build limit of 0; no tree.
- Part e (no announcement for a gatherer): a test vehicle with the resource-gatherer flag deploying
  into the 2 × 2 building with a tree at (*x* + 1, *y* + 1), and a second without the flag.
- Part f (the turn under disturbance): the vehicle faces away from *D* and begins its turn; the
  recorder logs its cell and behaviour per tick. (1) A named parasite of the bystander attaches to it
  during the turn and holds it for a known number of ticks, which is 8 processings or more (it loses
  deploy readiness, S-0214, Terms, *disabled*); if no hold that ends can be set up, the hold
  lasts to the end of the run and only the first half of the expectation is recorded. (2) A
  scatter request is made during the turn (it gains a destination and moves to another cell); this
  is run with a queued behaviour present and with none, 5 processings of the vehicle's behaviour
  being recorded after it has stopped.
- Part g (a wall with no owner): the test building is a wall-tower test type (S-0406, claim 8) with
  a 3 × 3 foundation, and the cell (*x* + 1, *y* + 1) holds a map-placed wall overlay that has no wall
  owner (S-0210, claim 14); `quirks.deploy_refuses_own_walls` at its default (on).
- Part h (an own wall): the same with the wall owned by the vehicle's house.
- Part i (no shroud condition): the shroud is on for the house; a test deploying vehicle type with
  a sight radius of 0 deploys into the 3 × 3 building with its facing equal to *D*, no tree, and the
  foundation cells other than the vehicle's own cell unexplored for the house at the attempt (if the
  original still reveals the vehicle's own cell, those other cells remain unexplored).
- Part j (the original's *hunt*): the part b setup (2 × 2 building, tree at (*x* + 1, *y* + 1)) with
  the deploying vehicle a named object of the bystander computer player, which a test trigger gives
  the *unload* behaviour at a known tick. A run in which the computer player gives that vehicle another
  behaviour before the attempt is void.
- Part k (no deploy readiness at the first attempt): a parasite is attached by a test trigger in the
  tick of the deploy order, before the first processing, to a vehicle that faces *D* on a valid anchor:
  (1) a vehicle of the human house, (2) the bystander's named vehicle of part j.
- Part m (a moving vehicle at the first attempt): the part a setup (3 × 3 building, no tree, facing
  equal to *D*), but a move order to a cell 8 cells away is given one tick before the deploy order.
  If the deploy order stops the vehicle at once, the run says nothing about the rule and is
  recorded as such.
- Part l (a failed placement during a turn): the part g setup (the unowned wall at
  (*x* + 1, *y* + 1)) with the vehicle facing away from *D*, so that the turn begins first; a test trigger
  removes the wall 8 processings after the turn has ended.
- Observable: whether and when the building appears and its top-left cell, and the refusal
  announcement. Also the vehicle's behaviour after a refusal. In part f, its behaviour at each of the
  next 5 processings, where it stands, and whether and where a building appears. In part m, the tick
  the vehicle stops and the tick and cell of the building. In parts g and h, whether the building appears, the announcement, and the
  vehicle's behaviour afterwards. In part i, whether the building appears. In part j, the
  bystander vehicle's behaviour per tick after the attempt. In part k, whether the building appears,
  the announcement and the behaviour per tick. In part l, the behaviour at each processing after the
  turn has ended, and whether and when the building appears.
- Expected: part a: the trees at (*x* + 1, *y* + 1) and (*x* − 1, *y* − 1) refuse, the tree at
  (*x* + 2, *y* + 2) succeeds with top-left (*x* − 1, *y* − 1); part b: (*x* + 1, *y* + 1)
  refuses, (*x* − 1, *y* − 1) succeeds with top-left (*x*, *y*); part c: distance to the base never
  matters; ore refuses; part d: the vehicle deploys; part e: the gatherer is refused silently and the
  other vehicle's refusal plays the announcement; in every refusal the vehicle rests in *guard*; part
  f: (1) no building and no announcement while the hold lasts, the behaviour stays *unload*, and the
  building appears at the first processing after the hold ends; (2) the vehicle moves away, no
  building appears (neither during the move nor after it has stopped, at its new cell), and no
  announcement plays; with a queued behaviour it becomes current at the next processing after
  the abandonment; without one the vehicle's behaviour stays *unload* with no change of anything
  else (the hypothesis of claim 5); part m: no building while the vehicle moves, and the building
  appears at the first processing after it has stopped (at its new cell, which is the anchor); part g: no building and no announcement, the vehicle in *guard*
  (the hypothesis of claim 6); part h: refused with the announcement; in part i the vehicle
  deploys; in part j: the vehicle is in *hunt* after the attempt, not in *guard*; part k: (1) no building, no
  announcement, *guard* current by the next processing, (2) *hunt*; part l: no building, no
  announcement, the behaviour is *unload* at each of the processings with nothing queued, and the building
  appears at the first processing after the wall is removed.
- Falsified if: the anchor differs or distance matters; a refusal moves the vehicle; in part f (1) the building never appears after the hold ends, or the behaviour leaves
  *unload* during the hold (the turn is then abandoned by a lost readiness, against claim 5); in
  part f (2) a building appears after the vehicle has stopped (the destination then only pauses the
  attempts, against claim 5) or, with no queued behaviour, the behaviour at the next processings is
  not *unload* (this settles Open question 2); in part m a building appears while the vehicle is
  moving; in part g the building appears or the announcement plays; in part i the deploy is refused; in part j
  the vehicle rests in *guard*; in part k an announcement plays or the human vehicle's behaviour never
  becomes *guard*, or the bystander's does not become *hunt*; in part l the behaviour becomes *guard*
  or another behaviour while the wall stands, or the building does not appear after the wall is
  removed. The behaviour recorded per tick after a refusal also settles Open question 3.

**D3. Health handed over.**
- Setup: a test deploying vehicle with strength 300 at 101 hit points deploying into a test building
  with strength 1000; a second vehicle, also with strength 300, at 1 hit point deploying into a
  building with strength 50.
- Runs: 1 per vehicle (deterministic), observed area as above.
- Observable: the building's hit points in the completion tick (recorder).
- Expected: 336 (trunc of 336.66…) and 1 (trunc of 0.166…, raised to 1).
- Falsified if: 337 or 0.

**D4. Take-overs and retargeting.**
- Setup: a selected, veteran deploying vehicle in a control group, carrying a test trigger attached
  to it that fires on the destruction of its object, targeted by an enemy tank and an enemy vehicle
  thief (both named objects of the bystander, which the void rule allows) and holding a target of its
  own; it deploys into a construction yard. A second run with a slave-master test vehicle that has two
  slaves (S-0215) deploying into a non-yard building.
- Runs: 3 per setup (the outcome does not depend on the random stream; the repetitions guard against
  void runs); observed area: the 15 × 15 cells around the vehicle, with the tank and the thief placed
  within 6 cells.
- Observable: the building's selection, experience, group and target in the completion tick; the
  tank's and the thief's targets in the next tick; after the building is destroyed later whether the
  trigger fires; the slaves' owner and master.
- Expected: the yard is selected, veteran, in the group and holds the vehicle's former target; the
  tank attacks the yard; the thief has no target; the trigger fires on the building's destruction and
  not at the vehicle's removal; the slaves are the building's.
- Falsified if: any of these differs.

**D5. Counting.**
- Setup: (a) a test type whose prerequisite is the construction yard; (b) a deploying vehicle whose
  building is a factory-kind building, with a unit type it makes (S-0407, claim 1); (c) a vehicle
  type with a build limit of 1 that is owned by the house and is the deploying vehicle; (d) a test
  building with a power output of 100 as the deployed building, and a house that otherwise has a
  power balance of 0; a test building placed by a place order adjacent to the new building in the tick
  after completion, with the house having no other building within the adjacency distance.
- Runs: 1 per part (deterministic).
- Observable: (a) the tick the type's icon appears; (b) the tick the unit's icon appears; (c) the
  tick the vehicle type is offered again; (d) the power balance per tick and whether the place order
  succeeds; all against the completion tick.
- Expected: all of these change at the completion tick itself (S-0407, claim 12); the adjacent
  placement succeeds.
- Falsified if: any of these does not change, or changes at a later tick.

**D6. Packing up.**
- Setup: the redeploy setting on. A construction-yard test type of strength 1000 with a pack-up
  sound, deploying from a vehicle type of strength 600. Yard *Y*₁ is deployed from a full-health
  vehicle at a known cell, then selected, put in a control group, made veteran, given a test
  trigger that fires on its object's destruction, and targeted by an enemy tank (a named object of
  the bystander). Yard *Y*₂ is deployed from a second full-health vehicle at another known cell and
  damaged to 333 hit points by a test trigger. Each yard is packed up by a move click to a far cell
  (S-0213, claim 20), which gives it its guard spot and issues the sell order; the two clicks are on
  different cells. Part (c): the non-yard test building of D4's second run (deployed from the
  slave-master vehicle with two slaves), packed up by a move click in the same way. Credits are
  recorded.
- Runs: 1 per yard and 3 for part (c), with a different random seed per run; observed area: the
  15 × 15 cells around the building.
- Observable: the tick the build-down ends and the vehicle appears; its cell, facing, health,
  behaviour, selection, experience, group and target of the tank; credits; survivors; the local
  player's sounds with their ticks; in part (c) the slaves' owner and master.
- Expected: each vehicle appears in the build-down's last tick at the cell its deploying vehicle
  stood on, facing *D* × 256, moving to the clicked cell; *Y*₁'s vehicle has 600 hit points (ratio 1),
  is selected, veteran, in the group and targeted by the tank; *Y*₂'s has 199 (trunc of 199.8, F2); no
  credits; no survivors from any of them (each has a guard spot); the deploy sound at the first
  processing of the sale, the pack-up sound at the start of the build-down and no sell sound; in
  part (c) the slaves belong to the vehicle and have it as their master.
- Falsified if: a refund appears, survivors come out, or the cell, facing, health (200 for *Y*₂) or a
  take-over differs, or a sound is missing or extra.

**D7. Release rule.**
- Setup: a test building type that undeploys into a vehicle, is not a construction yard and has a
  3 × 3 foundation (so the sell mode offers *sell* on it, S-0214), sold through the sell mode
  without a prior move click, so that it has no guard spot; one run with the redeploy setting on and
  one with it off. A garrison or passengers are not needed: the survivors of the sale are observed.
  (The state of a construction yard that has a guard spot and fails another condition cannot be set
  up, claim 9; D6 covers the case of a packing building that has a guard spot, and S-0213's
  scenario O15 the sale of a yard that has none.)
- Runs: 3 per setting, observed area: the 15 × 15 cells around the building.
- Observable: infantry created at the sale; the vehicle at the end of the build-down; credits.
- Expected: survivors come out before the build-down, the vehicle appears and no refund is paid, in
  both settings (the quirk's default).
- Falsified if: no survivors, no vehicle, or a refund is paid.

**D8. Failed pack-up.**
- Setup: a packing construction yard (packed up by a move click, the redeploy setting on) whose
  return cell is made unenterable just before the build-down ends (a test trigger places a building
  there); a second run where the build limit of the vehicle type is 1.
- Runs: 3 per setup; observed area: the 15 × 15 cells around the yard.
- Observable: credits; whether the vehicle type stays at its limit afterwards (S-0407).
- Expected: the sale refund is paid; the vehicle type stays at its limit (the unplaced vehicle
  counts).
- Falsified if: no refund, or the limit is free again.
- Variant (S-0414 claims 2 and 4): the house has nothing else left, its construction-yard vehicle
  type is at the first base-unit position; 1 run per setting. Expected: defeat is not declared with
  the short-game setting on and is declared with it off. Falsified if: the reverse.
- Variants for the marks (claim 11): the same, short-game setting on, with the vehicle type marked
  (a) insignificant, (b) not scored; 1 run each. Expected: defeat is declared in both. Falsified
  if the house survives.

**D9. Other unload rules first.**
- Setup: test vehicle types that deploy into a test building, each with one of these: (a) a
  passenger capacity of 2, empty; (b) the deploy-fire flag without the simple-deployer flag; (c) the
  harvester flag; (d) the weeder flag; (e) the deploy-fire flag with the simple-deployer flag.
- Runs: 3 per type, observed area as above.
- Observable: whether the building appears after a deploy order.
- Expected: (a) to (d) do not deploy; (e) deploys.
- Falsified if: any of (a) to (d) deploys, or (e) does not.
- Not covered: the bunkered condition of rank 1. No scripted input is known that gives a deploy order
  to a bunkered vehicle, so that part of claim 1's rank 1 is `unmeasured`.

**D10. Pack-up sound on a sale.**
- Setup: (a) a construction-yard test type with a pack-up sound, the redeploy setting off; a sell
  order to it through the sell mode (S-0214: the yard is sold, not packed up). (b) The same type
  without a pack-up sound.
- Runs: 3 per part, observed area: the 15 × 15 cells around the yard.
- Observable: the local player's sounds with their ticks.
- Expected: (a) the sell sound plays at the sale's start and the pack-up sound at the start of the
  build-down (the default of `quirks.packup_sound_on_sale`); (b) only the sell sound.
- Falsified if: in (a) only the sell sound plays; in (b) any other sound plays.

**D11. A packing building whose type has no pack-up sound.**
- Setup: the D6 yard type without a pack-up sound but with a deploy sound, deployed from its vehicle,
  with the redeploy setting on; it is packed up by a move click (S-0213, claim 20), which gives it
  its guard spot and issues the sell order.
- Runs: 3, observed area: the 15 × 15 cells around the yard.
- Observable: the local player's sounds with their ticks.
- Expected: the deploy sound at the first processing of the sale and no other sound from the sale's
  start to the vehicle's appearance (no sell sound, no pack-up sound).
- Falsified if: the sell sound plays.

**D12. Position, height and layer of the deployed building.**
- Setup: the recorder also logs each object's position in leptons, its height above the terrain of
  its cell and its cell layer. Part a: the D1 setup (a), on open clear ground, and a 2 × 2 variant.
  Part b: a water-bound test building type (S-0406, claim 3 d) that a test vehicle deploys into,
  the vehicle standing on the cell that a high bridge marks without any overlay (the cell of
  S-0406's scenario PL4, part e); the vehicle type is one whose locomotion class may stand there
  (S-0211). If no vehicle type can stand there, part b is not run and the water-bound sentence of
  claim 6 stays untested.
- Runs: 3 per part (deterministic apart from void runs).
- Observable: the building's position, height and cell layer in the tick it appears.
- Expected: part a: position (256*x* + 128, 256*y* + 128) for the anchor cell (*x*, *y*), height 0, ground
  layer. Part b (low confidence): the building appears on the ground layer, at height 0 above the
  terrain of its cell.
- Falsified if: the position is not the cell centre or the height is not 0 or the layer is not the
  ground layer in part a; in part b, if the layer is the bridge-deck layer or the height is that of
  the bridge.

**D13. A deploy facing outside 0 … 255.**
- Setup: a test building type with deploy facing 300 (and a second with deploy facing −5) that a
  test vehicle deploys into and that undeploys into the same vehicle type; the vehicle starts on open
  ground facing 0. Since that vehicle never deploys, a building of the same type, owned by the human
  house, is pre-placed by the map; it is packed up by a move click (the redeploy setting on; the
  building is a non-yard type).
- Runs: 3 per facing.
- Observable: whether a building ever appears; the vehicle's body facing per tick for 30 processings
  after the order; the facing of the vehicle that appears at the pack-up.
- Expected (low confidence): no building appears, the vehicle turns at every attempt and its facing
  settles at 300 × 256 mod 65536 = 11264 (and 65536 − 1280 = 64256 for −5); the vehicle that appears
  at the pack-up has that facing.
- Falsified if: a building appears, or the settled facing differs. The facings that are found settle
  Open question 4.

**D14. Return-cell refusal and no alternative cell (Q-0132).**
- Setup: the standard measurement setup above, game speed index 2, 800 × 600 pixels, director
  and mods off. A tiny map has a packing yard with the redeploy setting on and a guard spot,
  and all eight neighbours of its return cell are clear. Use named map overrides for a second
  packing building with a zero-size foundation, so that the return cell can contain the test
  occupant or overlay without that building's own foundation displacing it. The yard run covers
  ordinary packing, and the zero-size test type isolates the entry test. Use a stock ground
  vehicle type as the return type, then named map overrides that change only its crusher flag or
  locomotion class where a pair below needs it. The bystander owns only the named enemy test
  occupants in the observed area and is otherwise kept away as in the common setup.
- Procedure: pack up by a move click to a distant clear cell. In separate runs arrange that the
  return cell at completion contains (a) an own standing vehicle, (b) an enemy noncrushable vehicle,
  (c) one own standing infantry unit, (d) three own standing infantry units, (e) one or three
  enemy crushable infantry units, with a crusher and a noncrusher return type, (f) a tree,
  (g) an uncrushable wall, (h) a crushable wall under an allied owner and under an enemy owner,
  with a crusher return type, (i) ore, and (j) water, with a wheeled return type whose water
  multiplier is zero. Include an empty clear-ground control. Put the contents there by the test
  map or a test trigger before completion; record their actual state rather than assuming the
  trigger succeeded. For water and ore, use the zero-size test building. In the water case,
  repeat with an amphibious class whose water multiplier is positive. Keep the vehicle's weapons,
  the map land multipliers and all other settings unchanged within each paired test.
- Runs: 3 valid runs per case. The observed area is the 15 × 15 cells around the packing building.
  Void a run if the required contents are absent at completion, an occupant is moving rather than
  standing, or a named enemy acts on the building before completion.
- Observable: the return cell's contents and occupancy reservations in the completion tick, every
  newly created vehicle's type, cell and on-map status, the building's removal tick and the credits
  delta. Continue recording for 30 ticks, with no additional orders, to detect a late appearance.
  Record health and owner as needed to calculate the sale refund by S-0408.
- Expected: clear-ground control succeeds with no refund. Cases a to d, the noncrusher part of e,
  f, g, the allied-wall part of h and the wheeled-water part of j refuse: the sale refund is paid,
  the building disappears and no vehicle appears anywhere, despite clear neighbours. The crusher
  part of e succeeds. The enemy-wall part of h succeeds when its land multiplier is nonzero.
  Cases i and the amphibious-water part of j succeed. A successful vehicle appears at the return cell
  in the building's removal tick, before moving towards the guard spot, not at a neighbour.
  Failed cases have the off-map owned count of D8 and never retry in the following 30 ticks.
- Falsified if: any refusal case produces an on-map vehicle, any acceptance case instead pays
  the refund, a refused vehicle appears in a neighbouring cell or later, or the packing building
  itself prevents the clear-ground control. Record unexpected occupant damage separately: this
  scenario tests admission, not the later crushing effects owned by S-0210.

## Open questions

1. The deploy readiness that S-0214 defines is asked at every attempt; scenario D2, part k, records
   the case of a parasite attaching before the first attempt. Whether a vehicle can be still and have
   a destination at the first processing after the order (the destination arm of claim 2) is not
   known.
2. What follows an abandoned turn (claim 5): whether the vehicle stays in *unload* doing nothing, as
   hypothesised, or takes another behaviour at its next processing is not known. Scenario D2, part f
   records it.
3. Whether the *guard* that a failed attempt queues replaces *unload* already in the tick of the
   attempt or only at the vehicle's next processing is not established (claims 2 and 4). Scenario D2's
   per-tick behaviour record settles it.
4. The facing that a deploy facing outside 0 … 255 gives, both for the turn of a deploying vehicle
   (claims 5 and 6) and for the vehicle that a packing building of such a type releases
   (claim 10): the original may reduce *D* × 256 modulo 65536 or use another reduction. Scenario D13
   records both.
