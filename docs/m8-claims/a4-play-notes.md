# M8 step A4: the lead's play notes

Step A4 of [m8-plan.md](../m8-plan.md) starts from the lead's play notes. Each note becomes a code
fix when the code departs from an approved spec, a spec request when the specs are silent or the
note disputes them, or an importer check when the data may be at fault. A note is never a source of
behaviour by itself: a rule it states reaches the code only through a spec.

Requests for A4 are numbered Q-1750 to Q-1769, and from Q-1771 on after them.

## Round 1 (2026-10-21, a build of main from that day)

Triage was read-only, against the specs and the code at main 80f8a85.

| # | Note | Class | Specs | Request | Status |
|---|---|---|---|---|---|
| 1 | "Weird pathing issues" | The specs are silent in places. No departure from the specs found. | S-0213 claim 7 (interim group spread); S-0212 Terms (threat, coarse threat), claims 12a, 13, 17 to 23 | Q-1750 (group destinations), Q-1751 (cell threat); Q-1752 held until the note is narrowed down | Waits for details from the lead: units, terrain, symptom, and best a save with the tick. Q-1750 answered by S-0213 revision 46 and built at M8 step A3 intake 7: the group spread's interim rules are now the claim's rules (look of 6 points for each member, the first one cell out, the fallback with no further search, `quirks.group_spread_no_detour`; the leader's cell shares the common cell; the entry check's reference is the common cell's height level). On open ground only a look with all 6 points taken makes members of different cells share |
| 2 | A group deploy should deploy all members while any is undeployed, never mix | Code bug | S-0213 *The deploy rule*; S-0214 Terms | Q-1753 (members that are still deploying or undeploying) | Fixed (A4-1, `targeting::key_command`): a garrisoned building first in the selection gets the only order; a selection mixing deployed and undeployed in-place deployers skips the deployed ones; every other member needs the deploy check and deploy readiness. Interims Q-1753 and Q-1754 retired at M8 step A3 intake 7 (S-0213 revision 46, S-0214 revision 49): a press sorts a member part-way through a transition by the side it is heading to (infantry in *deploy* counts as deployed, in *undeploy* as not; a deploying vehicle as deployed, an undeploying one as not), and an infantry type with an undeploy delay of 0 or more lacks deploy readiness in every deploy-kind sequence, *deploy* included, in *undeploy*, and while its rearm countdown runs |
| 3 | IFVs have no turret | Code bug, in the renderer: the importer keeps a multi-turret type's numbered turret parts, but the renderer reads only the single turret part | S-0105 claims 12, 16, 26; S-0007 claims 16 to 18 | Q-1760 (which turret model is shown) | Plumbing done (A4-1): the renderer draws `turret{k}` and `barrel{k}` of the selected index and keys the image by it. The index function gives no index until Q-1760 is answered, so IFVs still show no turret. The local lane confirms the premise: both multi-turret vehicle types import `turret0`, none a plain `turret`. Seen again 2026-10-24 (Round 3): the plumbing is in place on main of that day, nothing else is missing; still waits for Q-1760. Fixed at M8 step A3 intake 19 (S-0105 r35 claim 43, Q-1760 answered; ledger [intake19-S-0105.md](intake19-S-0105.md)). The importer reads the turret table of the fixed type name, and a gunner multi-turret vehicle now shows the table entry for its weapon number, from creation and at every passenger hand-over. Lane `ifv-turrets`: the gunner type shows a turret with no passenger, 17 passenger modes show 4 distinct turret indices and none shows no turret |
| 4 | Patriot missiles can target ground units | Code bug for a person's attack order: table B row B7's anti-ground and being-sold conditions are missing. Retaliation against a ground attacker follows the specs | S-0214 table B row B7; S-0309 claims 5, 18; S-0306 claim 23 and Import mapping | Q-1761 (retaliation, and B7 over aircraft); Q-1763 (the lead's follow-up of 2026-10-22: in the original an anti-air-only building never fires back at ground attackers, and no object gets a fire-back exception for a target it cannot otherwise attack) | Fixed (A4-1): row B7 gives nothing for a first-slot projectile that is not anti-ground, a pulse cannon or a building being sold, over ground units and aircraft alike (Q-1761 open for aircraft). Importer check: the 2 buildings whose primary projectile is anti-air and not anti-ground import anti-ground = no; no importer bug. Retaliation: fixed (A3 intake 5) by S-0306 revision 33 and S-0309 revision 38 (Q-1761 questions 1 and 2, Q-1763 answered): 2 of 2 such building types have land-targeting mode 1, so the firing check's land row refuses every ground object on land to them and they no longer fire back (S-0309 claim 18, first case); lane scene `aa-only` (`m7b_lane.rs`): before, each held the infantry attacker as its target until it died, for a human and a computer owner; after, neither holds or fires at a ground attacker. Q-1761 question 3 (row B7 over aircraft) stays open |
| 5 | Harriers that lose their target before firing circle at random | Not confirmed. By the specs the aircraft flies home when a pad grants contact, and it circles the old firing spot only with no dock types (T2's guard row). Also a small bug: the held flight point's cruise height is recomputed on every read instead of once when the point is accepted | S-0501 claims 5, 6, 15, 17, 18, 32, 40; T1, T2; AC19 | Q-1755, Q-1756 | Partly fixed (A4-1). The held point's height is now fixed at acceptance (claim 6), so a landable aircraft no longer dips at the old spot before going home; a target removed without a kill is now dropped (claim 32) instead of being attacked forever. With one dock type the aircraft flies home and docks. The local lane shows 10 of 12 aircraft types have no dock types, 2 of them armed and landable: by T2 such an aircraft moves to its last firing spot and holds at cruise height, now after every attack (before it landed by the recomputed height). Q-1756 asks whether it should land; Q-1755 asks about a target taken off the map. Q-1755, Q-1756, Q-1775 and Q-1776 answered by S-0501 revision 40 and built at M8 step A3 intake 9: an aircraft loses its target at the moment the target is killed, taken off the map or removed, with claim 37's timer draw (claim 32); an armed aircraft with no dock types heads for its house's edge after every run, a human player's lands at a usable cell near that edge cell and rests in guard, a computer player's keeps the edge cell in area guard (claim 45); a pad-bound aircraft with no pad to go to crashes at the run end, credited to nobody (claim 46). Q-1837 (area guard above the flight height) and Q-1838 (a visiting type placed on the ground retreats at once) were answered by S-0501 revision 41 and built at intake 13: area guard falls back only at exactly the cruise height, and a type that is not landable, placed on the ground, rests in guard and takes orders |
| 6 | Destroyed units leave spinning particles | The specs are silent. A sprite debris piece (a bouncing animation) plays in place with no motion and no landing, so a looping one never ends. This is the seam "bouncing motion and landing" | S-0109 Terms (bouncing type), claims 7, 22; S-0312 claim 4, F1; S-0202 claim 10 | Q-1762 (answered in part by S-0319 r9); Q-1846, Q-1847 | Fixed at M8 step A3 intake 15 part 1 (S-0319 r9, `bounce`; ledger [intake15a-S-0319.md](intake15a-S-0319.md)). S-0317 (built at M8 step C5 part 2) covered the voxel-model debris only. Since S-0319 a sprite debris piece flies from its creation, lands and ends there with its landing effects (claims 2 to 8), so a looping one no longer stays. Lane `debris-left` (below, note 13): bouncing animations left at +1000 ticks went from 10 to 0 |
| 7 | Barracks units with no rally point stay at the door and block later units | Follows the specs: the unit rests in guard and keeps the factory's one contact place, so later releases are busy and refunded | S-0417 claims 10, 14 and Open question 3; S-0404 claims 7, 13 | Q-1757; Q-1850 | Waits for Q-1757. Seen again 2026-10-24 (Round 3): the lead says the blocking "is 100% wrong": in the original, later infantry units still come out. Recorded as Q-1850 (the lead's report against S-0417 claims 10 and 14, a follow-up to Q-1757). The code keeps to the claims until Q-1850 is answered. Fixed at M8 step A3 intake 35 (S-0417 r64 claim 17, Q-1757 and Q-1850 answered; ledger [intake35-S-0417.md](intake35-S-0417.md)): a unit resting in guard on a cell of its barracks is sent a scatter request in its next turn, walks to a cell next to the barracks and frees the contact place at its first arrival, so later units come out. Lane `barracks-no-rally`: five units queued at a barracks with no rally point all come out, on five different cells |
| 8 | A chrono miner should teleport only when it returns to a refinery, but it teleports to any click | Disputes an approved claim: every destination teleports while teleporting locomotion is active, a person's move order included. The claim has no measurement record | S-0207 claims 14, 20 (a); S-0214 claim 11; S-0402 Terms, claims 9, 17 | Q-1759 (answered by S-0207 r82 claim 28) | Fixed in M8 A3 intake 39 ([intake39-S-0207.md](intake39-S-0207.md)): a chrono miner drives to every destination and jumps only to a refinery's docking cell in *enter* (and at a sale of that refinery); a click, attack-move, waypoint or harvest order is driven. Tests `claims_s0207_r82.rs` |

## The lead's answers (2026-10-22)

The lead played the native build. Answers to the triage questions:

- Note 4: the Patriot fired at ground units **on its own**, not after a person's order and not only
  after being hit. By S-0309 claim 5 an anti-air-only building's automatic search looks for aircraft
  only (classes come from the projectiles of its weapons in slots 0 and 1), so this route is not the
  retaliation route of Q-1761 and Q-1763. Traced on real data in a follow-up triage.
- Note 5: the harriers were the computer player's, and they circled over their target, which was
  the lead's base. The lead has not built harriers. Traced on real data in the same triage (dock
  types as imported, whether the computer player had pads).
  The lead adds, unsure: in the original an aircraft with nowhere to land crashes. Filed as
  Q-1775 (no spec gives a crash; no change until it is answered). Answered by S-0501 revision
  40 claim 46 and built at M8 step A3 intake 9: a pad-bound aircraft in flight that ends its run
  or loses its pad with no pad granting contact crashes then (health 0, a dead fall, one loss of
  its own house, no kill credited).
- Note 6: the spinning particles looked like debris. This matches the premise of Q-1762 (a sprite
  debris piece played in place with no motion and no landing).
- Note 1: details will come as a pathing report (M8 A4 capture: a key saves a snapshot from about
  20 seconds earlier and every order since; `glasnost-headless path-report` replays it).

## Triage 2 (2026-10-22, real data, read-only; lane `local-lane/m8/a4-triage-2/`)

- Note 4 (Patriot on its own). No code or importer bug. Both anti-air-only buildings have a single
  weapon slot whose projectile imports anti-air yes, anti-ground no (S-0306 Import mapping). In
  600-tick scenarios they never acquired a grounded enemy that did not hit them (S-0309 claim 5),
  for a human and a computer owner. Two spec routes make them fire at what looks like a ground unit:
  retaliation after any damage from a known attacker, splash included (S-0309 claim 18 with S-0306
  claim 23; Q-1761, Q-1763), and the scan of airborne objects, which accepts infantry and vehicles
  in flight, such as jumpjet-locomotor units (S-0309 claim 9). The lead's answer (2026-10-22): ground
  units only, none flying. That leaves the retaliation route, which needs damage from the unit fired
  on (splash included, so a hit the lead did not notice counts). By the lead's report in Q-1763 an
  anti-air-only building never fires back at ground units, so this is Q-1763's case; no change
  until it is answered. Answered by S-0306 revision 33 and S-0309 revision 38 and built at M8
  step A3 intake 5: 2 of 2 such building types have land-targeting mode 1 (the first case of S-0309
  claim 18), so they no longer fire back at ground attackers.
- Note 5 (computer aircraft circling). No code or importer bug. Only the two pad-bound aircraft
  types have dock types; the computer player had a pad for every one it built, and with no pad a
  pad-bound aircraft in flight crashes, by S-0501 claim 18 and T2 (seen in the lane). The armed,
  landable types with no dock types end their run in *guard* (T2), queue *move* to their
  destination (claim 33), which after a strafing run is the target object (T1), keep the holding
  mark (claim 6 with Q-1756's workaround) and stay above the target at cruise height. That is the
  lead's "circled over their target". Q-1776 corrects Q-1756's premise (destination: the target,
  not the last firing spot) and asks about armed non-landable types with no dock types. Q-1775's
  question 1 is answered by claim 18 for pad-bound types; the data has no type with dock types that
  is not pad-bound. Since M8 step A3 intake 9 (S-0501 revision 40, claim 45) these types no longer
  hold over the target: the run end gives them a cell on their house's edge outside the usable
  area, so a computer player's aircraft keeps that cell in area guard and flies to it, where its
  landings are refused, and a human player's lands at a usable cell near it.

## Round 2 (2026-10-23, native build)

| # | Note | Class | Specs | Request | Status |
|---|---|---|---|---|---|
| 9 | "The MCV cannot be deployed by clicking it again after selecting." (the deploy key works) | Code bug: `targeting::vehicle_deploy` had no row VD2, so a vehicle that deploys into a building fell to row VD4 and the click gave nothing | S-0214 table VD rows VD1, VD2; S-0415 claim 3 and F3; S-0601 claim 15 | none needed | Fixed. Row VD2: *deploy* if the building fits where the vehicle stands (`econ::deploy::fits`, the same cell test the deploy attempt runs, the vehicle itself ignored, a foundation wider or deeper than 2 anchored by `deploy::anchor`, the step the spec calls fixed) and `deploy_readiness` holds, else *no-deploy*. The spec's wording and the deploy order agree, so nothing departs from it. Row VD1 (route mode held) stays a seam: route mode does not exist (no key binds it), so it is never held. The click's order is the table M unload behaviour order; the key's is `Order::Deploy`; a test runs both on copies and gets the same building at the same cell. Tests `claims_s0601_deploy::vd_click::*` |
| 10 | "Random debris after a tank got destroyed, staying in place" (a screenshot shows a dark piece left on the ground beside the wreck spot) | Recorded only, no code change. Matches a sprite debris piece with no motion and no landing (Q-1762), or a voxel debris piece (Q-1702); which one cannot be told from the screenshot | S-0109 Terms (bouncing type), claims 7, 22; S-0312 claim 4, F1; S-0317 | Q-1762 (open); Q-1702 answered by S-0317 | Half settled at M8 step C5 part 2 (S-0317). It was not a voxel piece: until then a voxel debris type was handed to the animation path, where a name that is not also an animation type made no animation (at most the debug view's short-lived stand-in marker), so no voxel piece was ever drawn; since S-0317 every voxel piece flies, bounces and ends (its lifetime counter runs down each turn and a stop sets it to 0, claims 6 and 9). A piece left in place is a sprite debris piece. Fixed at M8 step A3 intake 15 part 1 (S-0319 r9): sprite debris pieces now fly, land and end; lane `debris-left` (note 13) went from 10 left at +1000 ticks to 0 |
| 11 | "Captured oil derricks do not animate, nor they get the player color" | Code bug, in the renderer, for the colour and for the animation after the capture; the specs are ambiguous for the animation before the first ownership change. The simulation's capture follows the specs: the engineer's capture changes the owner in place (same object). The map's buildings are drawn in the cell pass built at load, with their owner and health at load and their slot animations at their first image frame; the live drawing played their slots but never drew them, and nothing redrew them on an owner or health change. The colour is read from the owner at draw time for every building drawn live (and on the radar), so only the map's buildings were stuck | S-0509 claims 1, 7, 8; S-0214 table M; S-0103 claim 8; S-0108 claims 2, 11, 13; S-0109 claims 1, 5; S-0405 claims 8, 9; S-0515 claim 2 | Q-1772 (slots of a tech building before and at its first ownership change: S-0108 claim 11 starts them at set-up, S-0405 claim 9 stops the working animations of a building not in operation, and S-0108 claim 13, the only rendering rule for that, covers needs-power buildings only) | Fixed (`Live::redraw`): a map building whose owner or health differs from the start, or that has a running slot animation and is not an unmanned tech building, leaves the cell pass and is drawn live, in its owner's house colour, with its damage frame and its playing slots. A captured tech building is manned, so it now shows its new owner's colour and animates. Left alone for Q-1772: a tech building that has never changed owner keeps its load-time drawing (slots at their first frame), as before. Also fixed on the way: the map's buildings did not show damage in play (S-0108 claim 2), and the map's other animated buildings stood at their slots' first frame. Tests `mapview::capture_tests::*` (synthetic tech building captured by an engineer through a click; damaged; a building that needs no engineer animating from the start) |
| 12 | A screenshot with three small pieces left on the ground, two dark (one with a blue streak) and one tan, no wreck in view | Recorded only, no code change. As note 10: sprite debris pieces with no motion and no landing (Q-1762), or voxel debris pieces (Q-1702); the screenshot cannot tell which | S-0109 Terms (bouncing type), claims 7, 22; S-0312 claim 4, F1; S-0317 | Q-1762 (open); Q-1702 answered by S-0317 | As note 10 since M8 step C5 part 2: not voxel pieces (none was drawn before S-0317, and since it each one ends), so sprite debris pieces. Fixed at M8 step A3 intake 15 part 1 (S-0319 r9), as note 10 |

## Pathing report 1 (2026-10-23, note 1; lane `local-lane/m8/a4-path1/`)

The lead's first pathing report, played on 2a334bd and replayed there to the capture's state hash.
Two infantry units of an engineer type leave a factory and stand at its exit cell (142,71), just
south of its foundation (141..143, 69..70). Ordered to capture, or move to, cells about 15 cells
north, (136..139, 54..57), across a base whose buildings stand in short rows (x 135 to 142, y 60 to
64) with open ground east of them, they never move: 19 path computations from (142,71), ticks 8897
to 10465, each with 5 cell searches inside a corridor of 9 to 26 expansions, all failed; each time
the unit gives up, its destination is cleared and it rests in *guard*. A 5-cell order east, to
(147,69), succeeds; from (145,67) the capture order succeeds with a 19-step path round the east
ends of the rows.

**Cause.** The cap of 5 cell searches of S-0212 claim 13. The coarse searches see through buildings
(claim 1), so the first level-0 corridor runs north through the factory's own cells. Hand check of
the first retry (claim 13a): the progress cell is the exit (142,71) (no candidate in the next
corridor sector is recorded: the factory's cells are *blocked*); its level-0 sector is the block
(142..143, 70..71); the free part is (142,71) and (143,71); the probe cell (142 + 0, 71 + 1) =
(142,72) lies in another sector, so the sector is *whole*, and its connections to the three
sectors north of it, which the free part does not border, are closed. The level-1 and level-2
corridors still hold the factory's block, so the next corridors run round the factory's west side
into the next row (progress cells (140,65), then (139,65), then (137,61)), where the retries
read sectors as split and close their connections onward. The level-2 corridor takes in the blocks east of the factory only at the 5th
search, whose level-0 corridor crosses the factory's corner diagonally between two building cells
(claim 13b's connection rule ignores buildings); it fails, and claim 13's table fails the
computation. Claim 23 row 2 then gives up. A diagnostic build (not committed) showed that a 6th
search still fails and the 7th finds a 17- to 19-step path, and that flipping
`quirks.corridor_retry_no_fallback` or `quirks.corridor_split_probe` changes none of these
computations: the fallback search of the first switch follows only a failed repeated coarse search,
and here no coarse search fails.

**Class.** No departure from the specs: the code follows S-0212 claims 1, 13, 13a, 13b and 23 and
S-0209's *blocked* outcome for building cells, and S-0212 says so for buildings (Edge cases
"Buildings do not separate regions" and "A short line of buildings across the route"; P12 expects a
give-up at long walls). The note disputes the result, and it has the gameplay impact for which
Q-1280's case got a switch, but no switch covers the cap. Filed as **Q-1771** (it takes up the
question held as Q-1752, which was never written): does the original's unit walk out, and if not,
should a switch lift the cap's failure. Interim: S-0212 as written, pinned by
`crates/deterministic/sim/tests/claims_s0212_cap.rs` on a layout of our own (a 3 × 2 block, its
exit, a row of ten 1 × 1 buildings 4 cells north, the destination behind it): 5 corridor searches,
the computation fails and the destination is cleared at once, the same with
`quirks.corridor_retry_no_fallback` off, and a search without a corridor finds the way round.

**Tool.** The report printed every failed computation as "to no cell (the destination X was
substituted)", although nothing was substituted (the trace has the searched-for cell only when a
path is stored). A failed computation now reads "to X" with no substitution remark, and the totals
count a give-up after a failed computation as well as a given-up move (the report said "0
give-ups" for a unit that gave up 16 times).

## Round 3 (2026-10-24, the lead's build of main of that day; lane `local-lane/m8/a4-r3/`)

The lead's words: "still particle issues [screenshot]. the IFVs didn't have their turrets still.
also the prism tanks. the barracks unit spawn issue was still there. the patriot turret and ifv and
other AA capable units/defenses should engage with the enemy air units automatically, currently it
seems that they don't or are too delayed." Notes 3 and 7 are seen again (their Round 1 rows say so);
13 to 15 are new.

| # | Note | Class | Specs | Request | Status |
|---|---|---|---|---|---|
| 13 | "still particle issues", with a screenshot: a small blue-and-white piece on paving beside an infantry unit, a dark piece on the grass above it, two small tan pieces on a road beside vehicles | Recorded only, no code change. The lane scene `debris-left` kills enemy infantry by each death kind and enemy vehicles, and samples what stays on the map every 50 ticks up to 1000 ticks later: only bouncing animations (sprite debris pieces) stay, 10 of them at the end; every corpse animation ends inside the window, every voxel debris piece ends earlier, no other animation is endless and no particle system with live particles stays (the census counts empty systems too; they draw nothing). The tan pieces and the dark piece match sprite debris pieces (Q-1762). The blue-and-white piece beside an infantry unit matches a corpse animation, which ends (S-0107 claim 17); it is a leftover only if it was still there long after the death, and then it too is a sprite debris piece. Same before and after the note 15 fix | S-0107 claim 17; S-0109 Terms (bouncing type), claims 7, 22; S-0312 claim 4, F1; S-0317; S-0616 | Q-1762 (answered in part by S-0319 r9); Q-1846, Q-1847 | Fixed at M8 step A3 intake 15 part 1 (S-0319 r9, `bounce`; ledger [intake15a-S-0319.md](intake15a-S-0319.md)), with notes 6, 10 and 12. The same scene on main 707b61f and after the step (rebased onto b236238) (`local-lane/m8/a3-15a/debris-left-before` and `-after`; 14 kills, 6 of them vehicles): before, 10 bouncing animations at +50 ticks and still 10 at +1000; after, 4 at +50 and 0 from +100 on. Corpses, voxel pieces and other animations end as before. The lane's screenshots still do not frame the kill spot, so there is no side-by-side crop |
| 14 | Prism tanks have no turret | Same case as note 3. The type the note names is one of the 2 multi-turret vehicle types: it imports four numbered turret parts and no plain turret part, so the renderer draws no turret until Q-1760 gives the selected index. The local lane's class counts over every voxel vehicle type: 2 multi-turret (none drawn until Q-1760), 15 with the turret flag and a single turret part (drawn), 1 with the turret flag and no turret part, 57 with neither | S-0105 claims 4, 8, 12, 16, 26; S-0007 claims 16 to 18 | Q-1760 | Fixed at M8 step A3 intake 19 (S-0105 r35 claim 43; ledger [intake19-S-0105.md](intake19-S-0105.md)). This type is the charge-turret multi-turret type. Its turret now follows its rearm countdown (S-0301 claim 16), with turret 0 from creation and between shots. Lane `ifv-turrets`: index 0 at creation, and after each of 3 shots in 300 ticks it steps 3, 2, 1, 0 |
| 15 | Anti-air defences and units do not engage enemy aircraft on their own, or too late | Code bug. Aircraft in flight were never registered as tracked flying objects. An aircraft in flight is listed in no cell (S-0209 claim 13), and a range search that looks for aircraft finds them only through the tracked flying objects it scans first (S-0309 claim 9; an aircraft-only search scans no cell ring), so no automatic search ever found an aircraft in flight: anti-air-only defences fired only when ordered or after being hit. S-0303 Terms make an aircraft-kind object in ordinary flight a tracked flying object; S-0501 gives no events for when that starts and ends | S-0303 Terms, claim 11; S-0309 claims 1, 4, 5, 9, 20; S-0209 claim 13; S-0501 Terms; S-0215 claim 33; S-0308 claim 6 | Q-1845 (when an aircraft is a tracked flying object), answered by S-0501 r45 claim 47 | Fixed. Round 3 kept the registration by Q-1845's interim rule (tracked exactly while on the map and in flight). Since M8 step A3 intake 16 ([intake16-S-0501.md](intake16-S-0501.md)) it follows claim 47: tracked from the take-off start, or from a placement in the air with the counts-as-aircraft flag, to an accepted touchdown, a crash impact or leaving the map (`objects::take_inside`, `superweapons::warp`, removal); its sector is brought up to date at the start of each movement update with a nonzero carried speed fraction. Tests in `claims_s0309_airborne.rs` (synthetic types, real order and tick path, human and computer owner): an anti-air-only building, an anti-air and anti-ground unit and an anti-air-only unit each acquire and fire at an enemy aircraft flying past on a move order; an aircraft is tracked exactly while in flight. The tracking has a second effect: a detonation above the terrain now reaches an aircraft in flight, at half distance (S-0303 claims 6 and 10; `area_damage_reaches_an_aircraft_in_flight_at_half_distance`), and under `quirks.flying_candidates_unfiltered` (S-0303 claim 9) an aircraft's own burst at its own height can hit it. Lane timings below |

**Note 15 in the lane.** Scene `aa-auto` (`m7b_lane.rs`) puts each of the 33 building, vehicle and
infantry types with an anti-air weapon on open land, owned by the local house and then by a
computer house, and flies an enemy aircraft across it on a move order, so that no retaliation
decides. Before the fix the aircraft was tracked on 0 of its flight ticks for every type and owner,
and no type took it as its target. After the fix it is tracked on every flight tick, and the types
it passed within weapon range engage it. Ticks are counted from the tick it first comes into range
in flight:

| Searcher | Owner | Target taken | First shot |
|---|---|---|---|
| Buildings 0, 1, 3 | human and computer | +17 to +19 | +30 to +32 (it turns to face first) |
| Vehicle 4 | human | +7 | +10 |
| Vehicle 6 | human | +1 | +11 |
| Vehicle 7 (distributed fire) | human | never held | +20; 5 shots, the target dropped onto its fired-at list after each (S-0309 claim 20) |
| Vehicle 7 (distributed fire) | computer | +7 | +7 |
| Vehicle 10 | human / computer | +20 / +24 | +26 / +30 |
| Vehicle 11 (gunner type, empty) | human | +1 | +7 |
| Vehicle 12 | human | +7 | +11 |
| Vehicle 15 | human | +17 | +17 |
| Infantry 22 | human | +2 | +5 |
| Infantry 29 | human | +12 | +14 |

The target is taken within one targeting-timer interval of entering range (S-0309 claim 1, rows
"targeting timer has run out"; the timer's draw is S-0215 F6), as the specs give. Building 2 and
many unit types never had the aircraft within range on the lane's flight line. The other
computer-owned units held *attack* against another target, ordered by the live skirmish AI, and
under S-0215 claim 33 only *move*, *harvest* and *guard* run the per-tick search, so they did not
switch: this follows the specs. Infantry type 20 has the aircraft in range throughout but its
search classes leave out aircraft (its primary warhead is infantry-only; S-0309 claim 5), which
also follows the specs. An empty unit of the gunner type that note 15 names has weapon number 0
(S-0308 claim 6), so it uses slot 0, which is anti-air and anti-ground.

**Shots that do no damage.** Vehicles 4, 7 and 15 fire at the aircraft but never damage it (the
other types' shots do). A diagnostic pass on the rebased tree (not committed) traced both cases
to approved claims, and the code follows them. The shot at an off-map target, the third case
looked at, was a code bug.

- Types 4 and 7: the slot that fires at the aircraft has no muzzle position in the type's art
  image section (no muzzle key of any form is there). By S-0313's Import mapping an absent muzzle
  position is 0, 0, 0, so by claim 1 the fire point is the firer's own position, on the ground (on
  flat ground the body transform is a rotation only, S-0105 claim 7). S-0306 claim 8 places the
  steering projectile there, so *h* is 0 at the start of its first turn, which falls in the tick of
  the shot, and T1's row "steering, *h* ≤ 0: detonate" detonates it beside the firer. The aircraft
  is at cruise height, far outside the blast. Any steering shot from a firer on the ground whose
  slot has no muzzle height does the same.
- Type 15: its weapon is a disc laser, so the shot makes no projectile (S-0615 claim 12). Claim
  14's range re-check measures from the firer's position to the aircraft's position in three
  dimensions, with the height difference, since the firer is on the ground. As the aircraft flies
  on, that distance passes the weapon's range one step before the final step, and the disc laser
  is cancelled without damage, as claim 14 states.
- The shot at an off-map target that detonated at the firer (an aircraft hitting itself before
  intake 9): this one was a code bug. The target, a vehicle on the ground, had been taken inside,
  and the projectile turn replaced any target off the map with no target. S-0306 claim 17 gives
  a target that leaves the game or becomes untargetable the cell it stood in when it was not
  airborne, and nothing only when it was airborne. With no target, the steering projectile was at
  or above the missile safety altitude, so T1's safety row detonated it at *P*′, beside the
  firer, and the unfiltered flying candidates of S-0303 claim 9 let the blast reach the firer.
  Fixed: `combat::projectile::target_leaves` applies claim 17 when the target goes inside
  (`objects::take_inside`) or is removed (`objects::remove`), as the kill and the teleport
  already did. Test `claim17_target_taken_inside_or_removed` (`claims_s0306.rs`). Since A3
  intake 9 an aircraft also loses a target that is taken inside (S-0501 r40 claim 32), so it
  makes no such shot.

No request is filed: the two remaining cases follow from approved claims, and the third was a
code bug, now fixed. Whether the slots of types 4 and 7 are meant to have no muzzle height is a
question for the lead, not for the code.

Note 7 (seen again): the lead's statement and Q-1850 are in note 7's Round 1 row; nothing more is
filed and the code keeps to S-0417 claims 10 and 14 until Q-1850 is answered. Fixed at M8 step A3
intake 35 (S-0417 r64 claim 17).

## Pathing report 2 (2026-10-25)

The lead's second batch, "especially the ore miners": two reports played on their native build of
2026-10-25, lane `local-lane/m8/a4-path2/`. Both name the build d31775d (the merge of C4 part 4),
built for the replay in a detached worktree with its own target folder: both replays reproduce
their capture's state hash. Neither has a mark. The two windows overlap and show one chain at the
person's refinery on raised ground: foundation (52..55, 98..100), docking cell (55,99) (S-0402
Terms: the foundation's top-left cell + (3,1)), the ore field west of the base at (36..38, 91..96),
more buildings between them, among them (50..53, 94..96) and (46..49, 96..99).

- **Report `path-20040-1`** (ticks 18750 to 20040, 29 orders, no selection). The clicks are on
  harvester #4984 on the docking cell. It has rested there in *guard* since tick 16657, after
  unloading. At tick 19693 it is ordered to harvest at (36,95): the computation at 19694 makes 5
  cell searches inside a corridor, all fail, and it gives up (destination cleared, *guard*). Moved
  to (56,102) at 19782 and again at 19795, it is refused 49 step checks into (56,100), ticks 19789
  to 19837, all *moving blocker* with nothing listed in the cell. Moved to (54,103) at 19837 it
  leaves by (55,100) at 19869, and harvesting from (56,102) at 19971 it finds a 20-step path in
  one cell search. The other units ordered in the window (a group of six vehicles of one tank
  type, #1119, #1120, #1121, #5497, #5832 and #5897, sent to attack and move, and harvester #5187,
  crowded by them on a bridge deck at (69,89), then sent to attack, move and harvest) are refused
  steps only by moving or standing units in the way, and all arrive.
- **Report `path-20828-2`** (ticks 19375 to 20828, 14 orders, selection #4553). Harvester #4553
  holds the refinery as its contact in *enter* from tick 16979, loaded, and drives round the
  docking cell, (56,98) → (57,99) → (56,100) → (57,99) → … from the snapshot to tick 19985, while
  #4984 stands on it. Once #4984 has left, #4553 docks at 19985 and unloads; its harvest at 20089
  to (38,91) fails with 5 corridor searches and it rests on the docking cell in *guard*. The
  lead's click on it at 20528 and the 7 harvest orders to (36,93), (37,93) and (36,94), ticks 20586
  to 20777, all fail the same way.

**Cause and class.** No departure from the specs; the case is pathing report 1's, for a vehicle
starting on a refinery's docking cell, and the rest follows from approved claims:

- The give-ups: S-0212 claim 13's cap (Q-1771, open). The coarse searches see through buildings
  (claim 1), so the first level-0 corridor runs west through the refinery's own cells. The
  progress cell of the first retry is the start (55,99); claim 13a's probe cell (56,100) lies in
  another sector, so the sector is *whole*. The later progress cells are (51,97), (53,97), (55,97)
  and (54,95); at the second search the level-2 retry reads its sector as *split* (probe (54,98))
  and closes its connections west, so the later corridors run north into the other buildings, and
  the 5th fails. Claim 23 row 2 then gives up: the destination is cleared and, with a
  human-controlled owner, the harvester queues *guard* (claim 27a), on the docking cell, where
  nothing moves it (S-0402 claim 3a). A diagnostic build (not committed) showed that a 6th search
  still fails and the 7th finds a path of 21 to 23 steps; that `quirks.corridor_retry_no_fallback`
  or `quirks.corridor_split_probe` flipped alone changes nothing; and that both flipped together
  find the way (the fallback search without a corridor, once a repeated coarse search fails).
- The circling: #4553's destination is the docking cell, which #4984 holds standing. S-0402
  claim 10 row 2 offers the dock again at each processing and claim 10a has no eviction, timeout or
  scatter, so #4553 keeps driving to it; S-0212 claim 24 substitutes a cell for the occupied
  destination, and claim 25's nearby search rejects its own cell (its own occupancy) and the
  building's cells, so it takes (56,98) and (56,100) in turn. Record only: claim 24's distance
  Dᵤ counts the unit's full height under `quirks.substitution_distance_flat_destination`'s
  default; measured from the surface, it would not reach the substitution here.
- The refused steps: #4553's two-cell curves (S-0207 claim 7) set the vehicle-present flag on both
  cells at once (S-0209 claim 1), (56,100) among them; claim 5 row VR5 gives *moving blocker*, and
  S-0207 claim 15 T4 sends that to S-0212 claim 22: wait and re-path, no scatter request.

No request is filed: Q-1771 covers the cap, and the main session or the lead may want to add this
case (a harvester on a docking cell, its refinery between it and the field) to it. Interim, pinned
by three tests in `crates/deterministic/sim/tests/claims_s0212_cap.rs` on a layout of our own
(a 3 × 3 block, a vehicle on the cell east of its middle row, two more blocks west, the
destination far west): from that cell 5 corridor searches fail, the destination is cleared at once
and the vehicle rests in *guard*; a search without a corridor finds the way; from a cell south of
the blocks the first corridor search finds it.

**Tool.** A *moving blocker* refusal whose cell lists nothing named no blocker (a seam of the
capture step). The report now names the vehicles whose vehicle-present flag is set on the cell at
the end of the tick, from their claimed cells (`path_report::flag_holders`), in the timeline and in
the summary ("nothing listed in the cell; its vehicle-present flag held by #4553 … at (56,98),
moving"). A failed computation whose 5th cell search, inside a corridor, failed now says "the 5th
cell search inside a corridor failed, which ends the computation (the cap of 5)". Replayed with
the change (`--any-build`), both reports reproduce their hashes and give the blockages above. The
holders are read at the end of the tick, so a vehicle that cleared the flag later in that tick is
missed and one that set it after the check is named; a refusal with nothing listed and another
outcome still names nobody.

**By eye.** Nothing is fixed, so this shows the case, on the lead's map and base or any base where
the refinery stands between its docking cell and the ore field with other buildings beyond it. Let
a harvester unload there: its way back to the field fails at once and it stays on the docking cell
in *guard*, and a harvester ordered to harvest from there does the same. A second harvester that
comes to unload drives round the docking cell until the first is moved off. Move the first a few
cells south and give the same harvest order: it goes.

## Round 4 (2026-10-25, the lead's native build of main; lane `local-lane/m8/a4-r4/`)

The lead's words: "The planes summoned by an aircraft carrier do not despawn / get killed when the
ship dies." "To attack an enemy mirage tank, I had to ctrl+click; I should be able to click
normally." "I saw an oil derrick animated with no house color (meaning it's animated without being
captured and/or it doesn't get the capturer's color)." "Also some problems with debris still", with
a screenshot. Lane scenes `carrier-death`, `mirage-click` and `derrick-look` (`m7b_lane.rs`).

| # | Note | Class | Specs | Request | Status |
|---|---|---|---|---|---|
| 16 | The aircraft of a carrier do not die when the ship dies | Code bug: a departure from S-0501 claim 36. Lane: the data's one ship spawner with three slots of a non-missile aircraft type, ordered onto an enemy vehicle, killed by a hit once all three slots were attacking (69 ticks after the order). All three aircraft were at health 0 one tick later, and none left play in 1500 ticks: they lay at height 0 on the water by the sinking ship, their fall counter still growing. The release crash (`spawn::start_crash`) set health 0 but not the killed mark, and our dead state is killed with no health left (`damage::dead`). So each fell as a living aircraft (counter +3 a tick), and at impact took the crash-without-kill hit of S-0312 claim 16 for its remaining health, 0, which killed nothing, again every tick. S-0501 claim 36: the release crash sets health 0, "not a new living disabled-fall state"; by claim 9 it falls as a dead aircraft and its impact follows S-0312 claim 16 for one shot down (its death weapon at the impact point, and it leaves play) | S-0506 claims 12, 13; S-0501 claims 9, 36; S-0312 claim 16 | none | Fixed: `spawn::start_crash` marks the released aircraft killed, with none of a killing hit's other effects (claim 36 gives the release none). Test `claim_36_released_aircraft_crash_and_leave_play` (`claims_s0501.rs`): a carrier killed by a damage hit; each flying slot aircraft is dead at health 0, its fall counter grows by 1 a tick, and it leaves play. Lane after: the three aircraft leave play 2, 15 and 22 ticks after the hit. The data's two other ship spawners of a non-missile type (one slot each) did not launch against the lane's ground target in 1500 ticks; their docked slot is removed silently at the death (claim 13), so nothing is left behind there |
| 17 | A plain click does not attack an enemy mirage tank; ctrl+click does | Code bug in the client: a departure from S-0214 Terms (*clicked object*). Lane: the data's one disguise-when-still vehicle type takes a terrain-object disguise with no house one tick after it stops; it appears disguised to the local house and cannot be selected by it (S-0516 claims 35, 36); the general attack-cursor-on-disguise rule reads yes in the data. Before: for each of 89 armed own vehicle and infantry types the plain click went to its cell (no-move 88, move 1) and force-fire attacked the cell (89), the same with its fake blink running. `Play::target` kept only an object the local house can select, and claim 36 makes a unit disguised as scenery unselectable, so the click became a click on its cell. By S-0214 Terms only a fully cloaked object the viewer neither owns nor senses cannot be clicked; the scenery disguise excludes selection only. Clicked as the object, the disguise rule of claim 6 passes with the general rule (and with the fake blink, S-0516 claim 39) | S-0214 Terms (clicked object, can be selected, scenery check, fallback move), claim 6, table G rows G2, G8 to G11, table AC row AC4; S-0516 claims 26, 35, 36, 39 | none | Fixed: `Play::target` asks the new `targeting::clickable_by` (`cloak::view::clickable`, claim 26's cloak exclusions; `pickable` is it plus claim 36); the select rows (G2, G10, claims 3 and 12, the cursor with nothing selected) ask `selectable_by` for the clicking house, so the mirage is still never selected by a click. The scenery check is built (`targeting::scenery_check`: move for a target disguised as an overlay; `fallback_move` takes the target; row AC4). Tests `s0214_claim_6_cursor_over_an_enemy_disguised_as_scenery` (`claims_s0516_disguise.rs`: with the general rule on, attack; off, no-move for a terrain disguise, move for an overlay disguise, attack while the fake blink runs; nothing selected, no select; the owner selects it) and `a_click_finds_an_enemy_disguised_as_scenery_but_not_a_cloaked_one` (`play_tests.rs`). Lane after: plain click attack 74, bomb 2, no-move 13, all on the unit; force-fire attack 84, bomb 2, no-move 3; the same counts disguised, blinking and undisguised (the control), so the disguise no longer changes the click. The pass for a terrain-fire weapon (claim 6, low) stays a seam: the importer does not read the key |
| 18 | An oil derrick animated with no house colour | Recorded, no code change: the colour follows the specs and the animation is Q-1772's. Lane, map 0: 5 map buildings need an engineer and have running slot animations; none has the terrain-palette art flag (S-0103 claim 8's exception). Fresh: neutral, unmanned, drawn in the cell pass (standing still), light grey. Damaged: drawn live, its 2 slots play, still light grey. Captured by the local house or by the enemy (277 ticks): manned, drawn live, slots play, the capturer's colour. So what the lead saw matches a derrick never captured but drawn live, most likely because it had been hit (or it was protected, flashing or being erased), animating in the neutral house's light grey (S-0103 claim 10). A captured derrick drawn without its capturer's colour was not seen in any case. Q-1772 question 1 asks whether an unmanned tech building's slots play at all; until it is answered the interim draws one live (and so animates it) once its owner or health changes, which is why a damaged one animates and a fresh one does not. If it is seen again, look at its health bar (damaged?), its owner in the hover name, its radar dot's colour, and whether income arrives | S-0103 claims 8, 10; S-0108 claims 2, 11; S-0405 claims 8, 9; S-0509 claim 8; S-0515 claim 2 | Q-1772 (open; covers the animation half). No new request | Recorded. Test `a_damaged_unmanned_tech_building_animates_in_the_neutral_colour` (`capture_tests.rs`) pins today's interim: drawn live after the damage, its slot plays, in the neutral house's palette and not a player's. Reopened below (*Note 18, reopened*): one code fix (an unmanned one drawn live for its look no longer animates), and the art explains the "no colour" half |
| 19 | "Some problems with debris still" (screenshot: small pieces near infantry and on open ground) | See intake 15 part 1 (S-0319, bouncing animations), which was not on the lead's build. S-0319 is not on main at this round's rebase (0d08efc). From the picture: small flat pieces lying still, one light one beside the infantry, one on the open ground to the north-east and two darker ones near the track, the same look as notes 12 and 13, which round 3's lane matched to sprite debris (bouncing animations); voxel debris ends since C5 part 2 (S-0317). So they are of the kind S-0319 ends | S-0109 Terms (bouncing type); S-0312 claim 4; S-0317; S-0319 | Q-1762 | Waits for intake 15 part 1; to be confirmed on a build with it |
| 20 | "The ships do not sink with an angle" | No spec: S-0504 claim 4 lowers a sinking vessel and S-0105 draws its body from its facing alone, so it went straight down level. Our own design under the 2026-10-25 decision, drawing only: a vehicle the simulation marks sinking pitches bow or stern down by up to 24° with a roll to one side of up to 8°, in 12 steps over its first 240 leptons below the surface (48 ticks), then holds until it leaves play at 400; the end and the side come from a bit mix of its object id (no random draw, the same on every client and replay). Lane `sinking-list` (`m7b_lane.rs`, output `local-lane/m8/a4-n20/`): 16 naval vehicle types, 14 voxel and 2 sprite; 7 sink at a killing hit (all voxel), 4 are underwater (2 voxel). A vessel sunk by a hit (id 363, facing 8192: stern down, starboard side down) and an underwater one set sinking as a cull does (id 367, facing 40960: bow down, starboard side down): step 0 of 12 two ticks after the hit, 3 at 12 ticks, 10 at 40, 12 at 78; the first left play 81 ticks after the hit. No vessel type a hit sinks has the turret flag, so the turret and barrel riding the body is tested on synthetic data only | S-0504 claims 4, 6, 10; S-0105 claims 7, 8, 10; S-0106 claims 9, 11; S-0614 claim 11 | Q-1871 (answered: S-0504 r26) | Answered. S-0504 r26 claims 15 and 16 (F4, F5) replace the interim; taken in by M8 A3 intake 26 ([intake26-S-0504.md](intake26-S-0504.md)). The interim (`mapview::sinking`, its id-hashed side roll, shadow tilt and depth steps) is deleted. Now the simulation steps each sinking vehicle's pitch by 0.01 at each descent up to the full lean 0.785398185253143310546875 (about 45°) at the 79th, bow down for 8-directions 1 to 5 and stern down for 0, 6 and 7, fixed at the first step (hashed and saved, save format 79, AI game format 41). The view draws it by F5 for the driving, ship, hovering and jumpjet locomotors, the turret and barrel on the leaned body, and no shadow while sinking. The roll stays level (rocking is not modelled). The Class text is the interim's history |

**By eye (note 16).** Build a carrier, let it send its planes at a target, and sink it while they
are out: each plane falls and disappears when it reaches the ground or the water, within a second
or two; none stays lying on the water or the ground.

**By eye (note 17).** Select any armed unit and hover an enemy mirage standing still (it looks like
a tree): the attack cursor shows, and a plain left click attacks it. With nothing selected, a click
on it selects nothing. Your own mirage still selects with a click.

**By eye (note 20, answered by S-0504 r26).** Sink a ship that a hit sinks rather than blows up: from the hit on it draws no shadow, and with each step down it tips further, one end under first, until it stands at about 45° just before it goes (it leaves play a little over 80 ticks after the hit; 82 in the lane). Whether the bow or the stern goes down depends only on which way it faced when it began to sink (its body facing's 8-direction: 1 to 5 bow down, 0, 6 and 7 stern down), not on the ship; it keeps that end even if it was still turning. There is no sideways lean. Frames: `local-lane/m8/a3-26/sinking-list-*.png` (see [intake26-S-0504.md](intake26-S-0504.md)).

**Note 18, reopened.** The lead disputed round 4's reading ("Once damaged, it starts animating ->
this doesn't match my observation"). Lane scene `derrick-paths` (`m7b_lane.rs`, output
`local-lane/m8/a4-n18/before` and `after`) and the code give every path we found by which a
needs-engineer tech building is drawn live or animates, or a captured one shows no owner's colour:

- *Reasons a map building leaves the cell pass and is drawn live* (`Live::redraw`): removed; a gate
  or occupiable type (none of the needs-engineer map buildings on maps 0 to 7 is one); protected,
  charge-flashing, air-strike-marked or being erased (S-0102 claim 34, S-0519 claims 25 and 26,
  S-0514 claim 27); its owner or health differs from the start; running slots on a building that
  is not an unmanned tech building. Nothing else: selection and hover change only the status extras
  (`Live::set_marks`), the shroud only the protection masks, and the radar has its own colours.
- *Code bug, fixed.* Q-1772's *Current workaround* lets an unmanned tech building's slots play only
  once its owner or health changes. Drawn live for its look (protected, flashing, marked or being
  erased) with its owner and health as at the start, it played its slots too. Lane before, map 0:
  protection and a charge flash each drew it live, 12 and 13 different image sets over 24 ticks
  (animating); after: 1 image set each (standing still), still drawn live with the look. Now such a
  building is drawn live with the set-up slots at their first image frame, as the cell pass draws
  them (`Live::redraw`'s `still_map`, `Live::unmanned_tech`; `LiveBuilding::slots` is `None` for
  it, the still branch of the painter's `slot_animations`).
- *Any health change, however small.* A 1-point hit draws it live and it animates (lane: 13 image
  sets over 24 ticks), with its normal body frame (no damage shows above the damaged ratio, S-0108
  F3) and a health bar that looks full. Any stray splash, a debris piece or a bomb's blast does
  this. This follows the workaround's own words, so it stays; whether an unmanned tech building's
  slots play at all is Q-1772 question 1. A repair back to its start health puts it back in the
  cell pass.
- *Ownership changes that leave it grey.* Every ownership change mans it for good (S-0515 claim 2),
  so a tech building that goes back to a passive house (a map trigger, or the release of a
  mind-controlled building, S-0601 claim 9; neither arises in a stock skirmish with stock weapons)
  animates in the neutral light grey. That follows S-0515 claim 2 and S-0103 claim 10; Q-1772
  question 4 asks about the slots. A house's defeat destroys its buildings (S-0414 claim 5), so a
  captured derrick never falls back to the neutral house that way.
- *The capturer's colour.* Captured by the local house (test
  `a_captured_map_building_takes_its_new_owners_colour`) and (lane) by the last of four houses
  (house index 3, lobby colour slot 6, 263 ticks): manned, drawn live, slots play, never the
  non-player grey. A computer or allied capturer goes through the same owner lookup, which does not
  depend on who plays the house (from reading the code; not run). One latent key mismatch was
  fixed on the way: the lobby's colour slots were keyed by the importer's form of the house name
  (`live::name_bytes`) while the drawing passes the name's own bytes, so a house name with a
  character in U+F780 to U+F7FF would have drawn every object of that house in the fallback grey
  (`load::load_with`). Our lobby names its houses in plain ASCII ("Player", "Computer 2" and so
  on), so this never showed, and no test covers it. The map's own imported owner colours are used
  only by the map viewer, outside a game. A network rejoin keeps the manned state (saved and hashed) and the start
  record; the client has no saved-game load. A tech building a map gives a player house at start
  is unmanned (S-0515 claim 2), stays in the cell pass, and shows that house's colour.
- *Engineer failure.* An engineer entering a neutral tech building in a skirmish captures it (S-0509
  claim 3 row 7; the multi-engineer rule's damaging hit of row 4 is unreachable in a skirmish,
  claim 6); one is refused only while the building is being erased (row 6), when it is drawn live
  for the erasure anyway. A refusal or a cancelled order changes neither health nor owner. A
  neutral one is never offered for sale (S-0408 claim 3, the sell mode acts on own buildings only),
  and a repair needs it to be one's own.
- *Lookalikes.* Over maps 0 to 7 (98 map buildings that need an engineer, all with running slots),
  no non-player building that needs no engineer shares a body image or a slot animation with a
  needs-engineer type (62 such buildings animate from the start, S-0108 claim 11, in the grey).
- *The art.* None of the slot animations of the two needs-engineer types on map 0 is drawn off its
  building (all have the drawing-with-object flag on, alternative palette and tiled off), so all of
  them use the owner's house-colour palette (S-0103 claim 8 row 3). But their pixels hardly use the
  remap range (indices 16 to 31, S-0103 Terms): type 0's two slot animations have 0 remap-range
  pixels in 8,325 and 7,374 opaque pixels over their frames; type 1's have 32 in 187,102 (one
  pixel in each of 32 of 128 frames) and 0 in 8,463. Only the body carries the colour: type 0's
  first frame 1,228 of 6,182 opaque pixels, its other two frames 420 of 5,675; type 1's frames
  449 of 2,933 down to 163 of 3,286. So the moving parts of a captured derrick look the same for
  every owner, and only a part of its body shows the capturer's colour, less on its later frames
  (the lane did not read which body frame is the damaged one). This follows S-0103 claim 8; no request.

So what the lead saw is one of: a never-captured derrick hit by as little as 1 point (it animates
under Q-1772's interim and still looks undamaged, in grey), one protected, flashing or marked
(fixed: it now stands still), or a captured one whose moving parts carry no house colour by their
art. No new request: Q-1772 questions 1 and 4 cover the open slots, and S-0103 claim 8 covers the
colour. Tests (`capture_tests.rs`): `an_unmanned_tech_building_drawn_live_for_its_look_keeps_its_slots_still`
(protected and flashing: drawn live, the slot at the cell pass's frame; fails before the fix; a
protected building that needs no engineer still animates) and
`a_chipped_unmanned_tech_building_looks_undamaged_but_animates`. Round 4's
`a_damaged_unmanned_tech_building_animates_in_the_neutral_colour` stays: it matches the workaround.

**By eye (note 18).** When a derrick seems to animate without colour: hover it and look at its
health bar (a hit as small as one point counts, and the bar may still look full; a damaged frame
shows only below the damaged ratio); select an engineer and hover the derrick (the capture cursor
means it is not yours or an ally's); look at its radar dot (grey: neutral; a house colour: owned);
and, if you think you captured it, watch whether your credits rise at each income cycle. A
captured derrick's colour is on its body's coloured panels only, never on its moving parts. A
neutral derrick under protection, a planted bomb's flash or an air-strike mark now stands
still while it shows that look.

**Note 18 after S-0108 r75 (M8 A3 intake 20).** r75 answers Q-1772 and replaces its interim
(`docs/m8-claims/intake20-S-0108.md`). A needs-engineer map building that is not manned is now
disabled at its placement (S-0108 claim 26): its slots that count *freeze-unpowered* hold the
frame they were created with, while a slot with no power flag on keeps playing (claim 13), and so
do its light and effect slots (claim 27). It is drawn live from the start (the still-map special
case is gone). A capture, or any other ownership change, re-enables it and the held slots resume
from the frame they held (claim 26; `quirks.enable_unfreezes_unpowered` on). A damage swap starts
the slots afresh (claim 12), so a damaged neutral derrick animates fully, still in the neutral
grey. Lane `local-lane/m8/a3-20/before` and `after`, map 0: a fresh derrick was in the cell pass,
1 image set over 24 ticks; now it is drawn live, 1 of its 2 slots frozen, 9 image sets over 24
ticks (its type's second slot has no power flag on, so it plays). Protection and a charge flash:
1 image set each before, 8 and 9 now (the same slots as a fresh one). A 1-point hit: 13 before,
9 now (no damage swap above the damaged ratio, so its freeze slot stays held). Damaged by half and
each capture (local house, enemy, the last of four houses; 277 and 263 ticks): unchanged, 0
frozen, both slots play, the capturer's colour after a capture.

**By eye (note 18, r75).** A neutral derrick never captured: one of its moving parts stands still
and the other moves, in grey. Capture it with an engineer: the part that stood still starts
moving from where it stood, and the body takes your colour. Hit a neutral one hard enough to show
its damaged frame: both parts move, still grey. Placing or replacing a building in play shows no
slot animation until its build-up ends.

**Checks.** The simulation changed (notes 16 and 17; the computer player clicks through
`targeting::click` and `cursor` too). Before on main 0b935c1; after, the arena on this branch on
0b935c1 and the sweeps and goldens on it rebased on main 0d08efc (main's changes in between touch
no simulation code). Oracle arena: the CSVs are byte-identical before and after (M-0001 31 of 96 pairings failing, 152 of 172 unanimous outcomes
reproduced; M-0002 13 of 24, 43 of 47; M-0003 2 of 35, 61 of 64). The M6c, M7a and M7b seed
sweeps 1 to 16 print the same lines before and after (smallest minimum M6c 4, M7b drain 3). The
sim-gate goldens hold unchanged (`cargo test --release -p glasnost-sim-gate -- --include-ignored`
without the update flag), so in the gate games the changed paths do not change play. No state
was added (the killed mark already exists), so the save format stays 76 and the AI game format 38.

## Round 5 (2026-10-26, the lead's build of main of that day; lane `local-lane/m8/a4-r5/ui/`)

The lead's words, in order (notes 21 to 34): "still the released infantry does not leave barracks
door" · "minimap doesnt have a box to show where we're looking at" · "slave miners do not work" ·
"Other miners from AI also seem not to work" · "no superweapon countdowns are shown" ·
"chronosphere building is missing a part" · "no indicator that I'm going to use a superpower, and
it's impossible to use when a unit is selected" · "no building repair animation" · "no
possibility to set waypoints (at least not from the GUI)" · "ore drills do not work, no ore
appears" · "T doesnt work to select all units of the type in screen. also other shortcuts arent
there" · "when I press D to deploy paratrooping units, they should deploy after landing" · "no
unit prices can be seen on hover or anywhere at all" · "no range indicator when placing defense
buildings". This part covers the interface notes (22, 25, 27, 29, 31, 33, 34). The others are
with the steps named in the table. Lane `r5_lane.rs` (`round_5_notes_on_the_real_data`) plays
them all in one skirmish on map 0 through the windowless client. Part 2 covers notes 26, 28 and
32; its lane `r5b_lane.rs` (`note_26_superweapon_buildings`, `note_28_repair_wrench`) writes to
`local-lane/m8/a4-r5/part2/`.

| # | Note | Class | Specs | Request | Status |
|---|---|---|---|---|---|
| 21 | Released infantry does not leave the barracks door | With M8 A3 intake 35 (S-0417 r64) | S-0417 | — | With intake 35 |
| 22 | The radar has no box for the battlefield view | Departure from S-0410 claim 11 (the outline that marks the battlefield view's part of the map is drawn over the radar map). Ledger [S-0410](../m5-claims/S-0410.md) row 11 said "not implemented" | S-0410 claim 11, F1, F2; S-0101 F1 | none | Fixed: `Radar::image_rect` and `view_outline` (`radar.rs`) give the view's four corners on the radar image through the inverse of F1 at height 0 (design: the outline reads height level 0). `hud::view_box_lines` draws a 1-pixel white frame clipped to the image (design). It is drawn only while the radar shows. Tests `view_box_lines_keep_the_edges_on_the_image_and_clip_them` (`hud.rs`), `the_radar_outline_follows_the_view` (client). Lane: outline edges (42, −5, 69, 34) on the image, (64, 16, 92, 56) after the view moved by (900, 450) |
| 23 | Slave miners do not work | Spec gap: S-0402 claim 14 wakes a slave-miner vehicle with no destination only on tiberium land, with a field in its short scan, or for a computer-controlled house; a player house is never the latter | S-0402 claims 14, 22 | [Q-1990](../../spec/requests/Q-1990-slave-miner-far-from-a-field-in-a-player-house.md) | Checked in M8 A3 intake 36 ([intake36-S-0402.md](intake36-S-0402.md)), lane `harvest_lane.rs` on map 0: a slave miner released far from a field with no order stays at rest for good; ordered to a field cell it drives there, deploys and its slaves work; near a field it waits about the kick delay and works. The placed slave-miner building, the harvester and the chrono miner work. Waits on Q-1990 |
| 24 | The computer's miners do not work either | Not reproduced; where it is a slave miner far from a field, as note 23 | S-0402 claims 14, 22, 25 | Q-1990 (as note 23) | Checked in M8 A3 intake 36 ([intake36-S-0402.md](intake36-S-0402.md)): on map 0 the computer's harvester, chrono miner, slave miner and slave-miner building cycle for 15000 ticks; on map 8 (headless, 22000 ticks) its two slave masters pack up at tick 20400 and redeploy at real sites, not at (0, 0). Its brain deploys and orders its own slave miners; one released far from a field with no order would sit, as in note 23. Fixed in round 5: our opponent sent slaves resting by their master into its bio reactor as spare infantry, where they never mined again (M7a seed 1; in seeds 1 to 16 it lost 2 to 11 slaves per seed (median 6), of 4 to 13 infantry absorbed, and none after the fix, with 0 to 7 absorbed). Class: our AI (design, not a spec departure); action: `free_infantry` in `absorber.rs` excludes slaves (test `slaves_are_kept`), as the grinder already did; the M7a gate seed moved 92 to 17 (seed 92 absorbed nothing without slaves; every counter at least 3 in seeds 17 and 26 of 1 to 40). By eye: a Yuri computer's slave miners keep mining through a long game; its reactors fill with other infantry |
| 25 | No superweapon countdowns | Departure from S-0512 claim 18 (no countdown row was drawn) | S-0512 claim 18, F6; S-0412 Terms | none | Fixed: `Session::countdown_rows` takes the granted records with the countdown flag of non-passive houses in grant order (`charge::countdown_rows`). `hud::countdown_view` draws each at the battlefield's top right: name in the house colour, then *s* = ⌊remaining / 15⌋ as h:mm:ss from an hour, else m:ss; at 0 the time blinks once a real second. Position, font and blink look are ours, as the claim allows. Seam: the client has no string table, so the row shows the type's name in place of its display name. Tests `countdown_time_switches_to_hours_at_an_hour`, `countdown_at_zero_blinks_once_a_real_second` (`hud.rs`), `granted_countdowns_show_on_the_battlefield` (client). Lane: one row for the placed superweapon building, counting down (4:40, later 4:39), its time blinking once it is ready (the blink's dark half, name without time, in `r5-34-range.png`) |
| 26 | The chronosphere building is missing a part | Departure from S-0108 claims 11, 13 and 18 (the superweapon slots *super-1* to *super-4* and *super-low-power* were not played: `slot_play`'s doc listed them as not covered), plus a request for a record not yet granted | S-0108 claims 11, 12, 13, 18, F6, Constants, Import mapping; S-0512 F2 | Q-2001 | Fixed for every type that names the slots, not only the chronosphere (the lane's six: Iron Curtain, chronosphere, weather control, missile silo, genetic mutator and psychic dominator kinds). `slot_play::Slot` gains *super-1* … *super-4* and *super-low-power* (art keys `SuperAnim` … `SuperAnimFour`, `SuperLowPower`), in claim 12's restart order. The grand opening starts *super-3* below 15 ticks of charge, else *super-1*, occupied variants allowed (claim 11). The charge cycle of claim 18 runs when `ChargedAnimTime` is at most 990: *super-1* → *super-2* when *t_c* ≥ *t_r* (F6, κ = 2 386 093 / 2³¹ in binary64), *super-2* ending → *super-3*, *super-3* → *super-4* when the charge restarts, *super-4* ending → *super-1*; not while sold. An *effect-unpowered* *super-3* hands over to *super-low-power* at a power loss and back at the return (claim 13). `ChargedAnimTime` has no typed field, so `live::charged_anim_time` reads it from the import's stored INI text of the type (no rules schema or hash change). The remaining charge is the owner's granted record (S-0512 F2); a record not yet granted counts as not charged and the cycle waits (`// design:`, Q-2001: S-0512 claim 2 grants it at the next recheck, after a building built in play opens). Intake 53: Q-2001 answered by S-0108 r102, which reads a record that is not granted by its clock (`quirks.ungranted_super_reads_clock`, on; the play-fixes layer turns it off), [intake53-S-0107-S-0108-S-0113.md](intake53-S-0107-S-0108-S-0113.md). Tests `grand_opening_starts_a_superweapon_slot`, `the_superweapon_charge_cycle`, `remaining_charge_in_game_minutes_f6`, `super_low_power_at_an_outage` (`slot_play.rs`), `charged_anim_time_is_read_from_the_building_section` (`live_tests.rs`). Lane `r5b_lane.rs`: all six types that name the slots run *super-1* (the dominator kind also its *active-1*); each sets `ChargedAnimTime` at most 990; the chronosphere charged to 100% shows *super-3* |
| 27 | No sign that a superweapon is armed; it cannot be fired with a unit selected | Departure from S-0214 claim 31 / table CS (no aim images), plus our design for the selection (Q-1985) | S-0214 claim 31, table CS; S-0513 claim 1; S-0512 claims 16, 23 | Q-1985 | Fixed: `cursor::Aim` gives table CS's frames for each aim action (nuke, storm, iron curtain, chronosphere both stages, force shield and its refusal, paradrop, spy plane, dominator, genetic converter, psychic reveal), offered while a superweapon is armed (`Session::offer`); a selection still takes precedence (S-0513 claim 1). Arming from the icon clears the unit selection, so the next battlefield click aims: our design for Q-1985, now S-0512 claim 25 (intake 50: arming also ends sell, repair and power mode; an icon with no aim action fires at once and keeps the selection; the select-target announcement has no identifier, seam Q-2140). Seam: table CS's radar column (the aim image over the radar) is not drawn yet. Tests `claim_31_table_cs` (`cursor.rs`), `arming_from_the_icon_clears_the_selection_and_the_next_click_fires` (`play_tests.rs`), `an_armed_superweapon_offers_its_aim_image` (client). Lane: armed with one tank selected, the selection is 0 after arming, and the battlefield shows the aim image |
| 28 | No building repair animation | Departure from S-0111 claim 19 (the wrench was not drawn: `combat.rs`'s doc said "buildings are not repaired yet") | S-0111 claims 1, 19, 19a, F4, Constants, Import mapping; S-0110 claim 8; S-0107 F3 | none | Fixed: `live::draw_status` draws the wrench (`WRENCH.SHP` through the cursor palette `MOUSEPAL.PAL`, `Cache::wrench_item`) centred at the status anchor of every building whose repair is switched on, unless its top-left foundation cell is shrouded for the local player by S-0110 claim 8's point test at the cell centre's surface height (`slot_cell_shrouded`). Image frame ⌊6 (*t* mod *p*) / (*p* − 1)⌋, *p* = max(2, ⌊N(14, *s*) / 4⌋) (28 … 4 by speed; 3 with `quirks.speed_normalised_delays` off). A missing sprite draws nothing and gives the importer's warning at load; a missing cursor palette gives its own. Its place among the extras (after the bomb icon, under the insignia and bar) is ours: S-0111 gives none. Tests `wrench_frame_f4`, `repair_wrench_claim19` (`live_tests.rs`). Lane: a damaged power plant switched to repair shows the wrench |
| 29 | No way to set waypoints from the interface | Built at intake 50 from S-0213 r54 claims 16 to 16g and 17 (Q-1986 answered) | S-0213 claims 16 to 17, 22, table T2; S-0215 claim 13 | Q-1986; Q-2136 to Q-2139 | Fixed (intake 50, [intake50-S-0213-S-0214-S-0512-S-0522.md](intake50-S-0213-S-0214-S-0512-S-0522.md)): Z held (`Action::RouteMode`, rebindable) is route mode; clicks add route points, join points or close loops; releasing commits; Delete removes a selected point; routes are drawn while Z is held. Simulation state in `routes.rs`, saved and hashed. Tests `claims_s0213_routes.rs`, `route_mode_adds_points_and_commits_on_release` (client). Intake 56 (S-0213 r74, Q-2138, Q-2139 and Q-2166 answered, [intake56-S-0213.md](intake56-S-0213.md)): 16-pixel click areas, the press that takes, refuses or passes a point, joins for the holders of a selected route's last point, the kind test at the press, holder selection, the short right click and the end at the options menu and defeat; tests the `route_mode_*` tests in `client_play_notes_tests.rs`. The command bar's route-mode button is not built (open in S-0213). Intake 63 (S-0213 r84 and r87, Q-2191, Q-2192 and Q-2193 answered, Q-2139 in part, [intake 63](intake63-S-0213-S-0301.md)): points issue their recorded orders at their recorded places, a loop close turns moves into attack moves for eligible holders, a loop that ends early runs to the route's end, and aircraft and buildings follow routes at their own moments |
| 30 | Ore drills give no ore | Import bug: a terrain object's frame count was never filled, so a mine that started its animation never stopped it and never grew ore again | S-0401 claim 15 (the mine's animation over its N frames, resource at its end) | none | Fixed in M8 A3 intake 36: `convert::sim` takes `image_frames` from the sprite's frame count (test `terrain_image_frames_from_the_sprite`). Lane (map 0, ore within 2 cells of all 14 mines taken away): before, 35 of 286 cells had ore again at tick 9000, from field spread alone; after, 123 (62 by tick 1500). A far 5 × 5 control patch stays empty. Seam: the client does not draw a mine's animation frames |
| 31 | T does not select every unit of the type on the screen; other shortcuts are missing | Type select: S-0214 Terms and claims 12, 19, 29 were not built at first; intake 50 takes S-0214 r62's tap (claim 32) and control groups (claim 33) | S-0214 Terms, claims 12, 19, 29, 32, 33, F9, Constants; S-0213 claim 22; S-0111 claim 17; S-0606 claim 7; S-0415 claim 6; S-0701 claim 17 | Q-1987, Q-1988 (answered by S-0214 r62); Q-2140 | Fixed. The type-select role's key is a binding (T by default, `Action::TypeSelect`). A tap (release at most 500 ms after the press) adds the selection's types (with DeploysInto) from the screen candidates, or across the map when the screen has none left or the reach is already map; the reach resets to screen after an addition or clearing; messages `MSG:SelAcrossScreen`, `MSG:SelAcrossMap`, `MSG:NothingSelected`; one response. Held clicks take own objects of the exact type within the reach; held drags read the reach. Control groups (S-0214 claim 33, S-0213 claim 22): Ctrl+digit makes, Shift+digit adds the group to the selection, Alt+digit centres on it, the digit selects it; a repeat less than 800 ms after the last select centres the view (F9). Quirks `group_center_keeps_extra_selection`, `group_kept_on_owner_change`. Tests `claim_32_*`, `claim_33_*` (`play_tests.rs`), `claims_s0214_tap.rs`, `the_type_select_key_taps_on_a_release_within_500_ms`, `control_group_keys_make_add_select_and_centre` (client), `defaults_are_the_specified_ones` (settings). The lane rows of round 5 predate intake 50 |
| 32 | Paratroopers told to deploy should deploy after landing | S-0522 r12 claim 17 (Q-2000 answered) and S-0214 r62 Terms: a falling unit takes orders and carries them out after landing | S-0213 table T2 (deploy rule); S-0214 Terms (*movable*, *able to fire*), claim 6 (e); S-0522 claim 17 | Q-2000; Q-2140 | Fixed (intake 50): falling is no longer part of *movable* or *able to fire*; the click's attack rule asks the firing check as for a landed unit (`check_ignoring_fall`, Q-2140). Tests `claim17_*` (`claims_s0522.rs`): a deploy press, a move click and an attack click during the fall are orders at once and are carried out after landing |
| 33 | No prices on hover | Our design: no spec gives a tooltip | S-0404 (price to the house); S-0512 claim 16, F6 | none | Fixed: hovering a sidebar icon (or focusing it with the keyboard; the pointer wins) shows a tooltip left of it with the type's name and its price to the house, or a superweapon's name with on hold, ready or its time left (`hud::tooltip`). The name has the same string-table seam as note 25. Test `an_icon_tooltip_shows_the_price_or_the_time_left` (`hud.rs`). Lane: a building icon's name and price; a superweapon icon's name and "Ready" |
| 34 | No range indicator when placing a defence | Our design: no spec gives a placement range indicator | S-0307 claim 2 (weapon range; negative is unlimited); S-0101 F1 | none | Fixed: while placing a building with a weapon of positive range, an ellipse of the longest range, projected by F1 (half-axes 30√2·R/256 and 15√2·R/256 pixels, integer square roots), is drawn around the foundation's centre at the pointer cell or the keyboard aim cell, in yellow, clipped to the battlefield (`Play::placement_range`, `range_axes`, `ellipse`). Tests `the_range_ellipse_follows_the_projection` (`session.rs`), `placing_an_armed_building_offers_its_weapon_range` (`play_tests.rs`). Lane: the first armed defence type, range positive, the ellipse around the placement preview |

**By eye (note 22).** Build a radar. A white frame shows on the radar map around the part of the
map you see, its edges cut at the map image's edge. Scroll the battlefield or click the radar:
the frame follows. Frames: `r5-22-25-radar-and-countdown.png`, `r5-22-radar-zoom.png`,
`r5-22-radar-moved-zoom.png`.

**By eye (note 25).** Build a superweapon whose type shows a timer (the lane used the first such
building on map 0). A line appears at the battlefield's top right: the type's name in your
colour and the time left as minutes and seconds (4:40 at first). When it is ready the time
blinks once a second. The name is the rules name until the string table is
read. Frames: `r5-22-25-radar-and-countdown.png`; the blink's dark half in `r5-34-range.png`.

**By eye (note 26).** Build a chronosphere (or an Iron Curtain, weather control, missile
silo, genetic mutator or psychic dominator). Once it is built its top part shows: the
chronosphere's dome sits closed on its base while it charges. When the charge is nearly full
the dome opens and, once open, shows the charged sphere with its energy until you fire. Two
things the lane did not see and only the unit tests cover: after firing the top closes again
and the charge starts over, and with low power a type whose charged picture is marked
effect-unpowered shows its low-power picture instead (of the stock types only the genetic
mutator names a low-power picture; the lane did not check its charged picture's mark). Frames: `r5-26-chrono-zoom.png` (charging), `r5-26-chrono-charged-zoom.png`
(charged), `r5-26-sw4-zoom.png` … `r5-26-sw13-zoom.png` (the other types), in
`local-lane/m8/a4-r5/part2/`.

**By eye (note 27).** Select a tank, then click a ready superweapon's icon: the tank is
deselected, and over the battlefield the pointer shows that superweapon's aim image (the lane's
Iron Curtain shows the red target rings). A click on the ground fires it. A right click cancels
the aim. Frames: `r5-27-aim.png`, `r5-27-aim-zoom.png`.

**By eye (note 28).** Damage one of your buildings, then switch on its repair (the repair button,
then a click on it). A white wrench appears over the building's middle and turns through its
frames, about the same pace at every game speed. It goes when the repair stops or the building is
whole. An enemy building under repair shows the wrench only where you have explored. Frames:
`r5-28-wrench-zoom.png`, `r5-28-wrench-0-zoom.png`, `r5-28-wrench-1-zoom.png`.

**By eye (note 31).** Select one tank with others of its type around it and press T: every tank
of that type on the screen is selected, none off it. Press T twice quickly: every one on the map
is selected. Hold T and click a tank: the same as the single press, but from the clicked tank's
type. Hold T and shift-click a selected tank: that type leaves the selection. Ctrl+1 makes group
1 of the selection, Shift+1 adds to it, 1 selects it, and 1 twice quickly moves the view to it.
There is no group number over the units yet. On the web, check that the browser does not take
Ctrl+digit for its tabs. Hold T and drag a box round one tank: every tank of its type on the
screen is selected; with Shift the earlier selection stays. The settings screen lists the
key and the three group rows, and they fit at 100% and 300% interface scale. Frames:
`r5-31-type-screen.png`, `r5-31-type-box.png`, `r5-31-group-went-to.png`, `settings.png`,
`settings-300.png`.

**By eye (note 32).** Unchanged until Q-2000 is answered: select paratroopers of a type that
deploys in place while they fall and press D. Nothing happens, and they stay undeployed after
landing. Press D after they land: they deploy.

**By eye (note 33).** Hover a building icon: a small box left of it shows the type's name and
"$" and its price. Hover a superweapon icon: its name and "Ready", "On hold" or the time left.
Frames: `r5-33-tooltip-building-zoom.png`, `r5-33-tooltip-superweapon-zoom.png`.

**By eye (note 34).** Start placing a defence: a thin yellow ellipse follows the placement
preview, as wide as the defence's reach on the ground (twice as wide as high). A building with no
weapon shows none. Frame: `r5-34-range.png`.

**By eye (note 30).** Clear the ore round an ore mine (or wait until the miners have). Within a
minute or two of game time the cells next to the mine hold ore again, a few at a time, and keep
filling while the field near it is mined. Before the fix they stayed bare.

**By eye (note 24).** Play the Yuri side's opponent on a map with ore near its base and watch its
slave miners: they deploy, the slaves walk out and back, and when the field nearby is mined out
the miner packs up and drives to another field.

**By eye (note 23).** Build a slave miner and send it onto an ore field (right-click the ore): it
drives there, deploys and its slaves walk out and back. One left standing with no order far from
any ore does nothing until it is ordered (Q-1990).

### Round 5 seams (2026-10-28, M8 A4 seams)

Note 31 left two seams open, and the web build's Ctrl+digit was to be checked. Before this part,
none of the pip scale (S-0111 claims 10 to 17) was drawn: `Live::draw_status` drew the health
bars, unit brackets, insignia, bomb icon and wrench. Control groups live in the client only
(`Play::groups`, presentation state, never hashed or saved). The brief's "claim 12" for the
pack-up is claim 10 in S-0415: claim 10 gives the vehicle the building's control group and
selection; claim 12 is about which return cells accept the vehicle. The code cites claim 10. The
"Not yet done" text of [Q-1988](../../spec/requests/Q-1988-control-group-input.md) is now stale
for these two seams. Implementers do not edit requests, so the lead may update it.

| Seam | Specs | Request | Status |
|---|---|---|---|
| Group digit over units and buildings | S-0111 claims 9, 10, 11, 17, Edge cases, OV12; S-0103 claim 15 | [Q-2040](../../spec/requests/Q-2040-control-group-number-look.md) | Drawn. A member of one of the local player's groups shows (*γ* + 1) mod 10 (`combat::group_digit`). The image's top-left corner is at (−4, −39) from the pip-scale origin *o* for infantry and (−4, −36) for others. For units *o* = (−10, +10) from the status anchor, so (−14, −29) and (−14, −26) (OV12's points). For buildings *o* is claim 10's building origin (`combat::pip_origin`, `group_number_at`). It is drawn with the pip scale: only when the health bar shows (selected or hovered) and claim 9 lets the viewer see the pip scale (own or allied owner; a building whose type has `PipsDrawForAll` or `CanBeOccupied`). A building whose `PipScale` is the resource kind shows none while `quirks.building_resource_pips_hidden` is on. That switch is now implemented, default on, and its row has left the not-implemented table in development.md. Design (Q-2040): our own 3 × 5 pixel digits in a 1-pixel border with a 1-pixel gap (7 × 9 pixels), top-left at the point. Colour: the owner's display RGB (S-0103 claim 15), white with no owner. Unshaded. Pushed last of the object's status extras, so it covers its bar and any later pip rows. The client hands the digits over in `Session::update_marks` (`Play::group_digits`, own objects only). Tests `group_number_digits_and_places`, `group_number_images` (`combat.rs`), `the_group_number_follows_the_pip_scale`, `a_building_group_number_and_the_resource_switch` (`live_tests.rs`). Seams: the pip rows (claims 11 to 15) and the self-heal marker (claim 16) are still not drawn |
| Control group and selection carry over on deploy and pack-up | S-0415 claims 6, 10, 11 | none | Done. The client cannot see the identity hand-over by itself: the building and the vehicle get new object ids. So the sim now reports it as a presentation event, `Effect::Converted { old, new }`. `deploy::complete` emits it after the building is placed (claim 6), and `deploy::pack_up`'s accepted branch after the vehicle is placed (claim 10). A refused deploy or a failed pack-up (claim 11) emits none. Effects are presentation only: `World.effects` is `#[serde(skip)]`, `Effect` has no `Serialize`, and the effects are not saved, hashed, replayed or sent over the network (sim-gate only reads and clears them). No format bump. Goldens are byte-identical. In the client, `play::converted` replaces the old id with the new one in the selection and in every group, in place (design: the member keeps its place in the group's order). This happens before the selection's `selectable_by` filter runs. Audio and the headless tool ignore the event. Tests `claim_6_deploy_reports_the_hand_over`, `claim_6_refused_deploy_reports_nothing`, `claim_10_pack_up_reports_the_hand_over`, `claim_11_failed_pack_up_reports_nothing` (`claims_s0415.rs`); `a_deploy_hands_the_control_group_and_the_selection_to_the_building`, `a_pack_up_hands_the_control_group_and_the_selection_to_the_vehicle`, `a_hand_over_keeps_the_order` (`play_tests.rs`). The pack-up test injects the event, because the fixture's yard does not pack up |
| Ctrl+digit in browsers | none (our design) | none | Our web build already asks for every key: winit's web backend calls `preventDefault` on the canvas's key events by default, and our window attributes keep that default. Keys the browser keeps for itself, such as Ctrl+1 to Ctrl+9 for tabs in common browsers, may never reach the page. Design: a rebindable make-group key (`Action::MakeGroup`, stored as `key.group`, N by default, no command key of S-0213 claim 22, Z included). Held with a digit, it makes that group as Ctrl+digit does. It acts only in the web build (`Client::group_fallback`, `cfg!(target_arch = "wasm32")`). Natively Ctrl+digit is unchanged and the key does nothing. The settings screen lists it as "Make a group, web (+digit)". An older settings file keeps N for it, or the first free key if it gives N to another action. Tests `the_make_group_key_stands_in_for_ctrl_on_the_web` (client), `a_file_older_than_the_make_group_key_keeps_its_own` (settings). Seam: not checked in real browsers |
| Settings screen on a 640 x 480 logical screen | none (our design) | none | Done. With 9 key bindings the panel was 554 px high and Reset and Back fell off a 480-px screen. The fixed-key list now sits in two columns under both the value rows and the key list (`glasnost_ui::settings::view`), so the panel is 456 px with 9 bindings and 14 fixed keys. Same stops, same keyboard order, no change in count. Test `every_target_is_reachable_at_a_small_window` checks every control at 640 x 480 and 800 x 600 |

**By eye (round 5 seams).** Make group 1 of a tank and select it: a small "1" in the owner's
colour sits over the left end of its health bar (our alignment, Q-2040). Infantry show it 3 pixels higher. Group 10
shows "0". A grouped building of yours shows its digit at its pip-scale origin (S-0111 claim 10) when
selected or hovered; one whose pips show its stored resource shows none. Put a construction vehicle in group 2, select it
and deploy it: the yard is selected and 2 selects it. Sell the yard (pack it up): the vehicle that
drives out is selected and in group 2. In the browser build, try Ctrl+1. If the browser switches
tabs instead, hold N and press 1. In the native build N does nothing and Ctrl+1 works as before.

## Round 6 (2026-10-31, the lead's build 48a9516; lane `local-lane/m8/a4-r6/`)

The lead's playtest of 2026-10-31, native, build 48a9516 (main after intake 51), *Pathing and
other fixes* on. The lead's words for this part (notes 35 to 37): "I can buy new black eagles and
harriers when others are in the air ... So the existing ones die when trying to return base. The
option is successfully blocked when the existing ones arent flying" (expected: buying blocked
while the house has as many planes as plane slots) · "IFVs and other untis with AA sometimes get
stuck attacking even after the threat has been eliminated ... mostly against missiles" · "units
ignore attack commands and disengage, sometimes just stop", with the pathing report
`path-46317-1.gpath`. Each was checked on main at 6267666 (after intakes 52a, 53 and 54). Lane
outputs are in `local-lane/m8/a4-r6/sim/`.

| # | Note | Class | Specs | Request | Status |
|---|---|---|---|---|---|
| 35 | New pad aircraft can be bought while others fly; the old ones crash when they come back | The specs gave it until S-0407 r29; r29 (Q-2172) makes claim 16's test at issue, with an aircraft in flight counted in *D*, so it became a departure, fixed by intake 57. The reading before r29: S-0404 claim 3 lets a build order start an item on an idle line when a switched-on aircraft factory has a free dock: a docking place empty, or no aircraft in contact with it. A pad-bound aircraft sent out on an attack holds no contact (S-0213 table T3 part b: a behaviour order breaks off an untethered actor's contact; a pad-bound type is not tethered, S-0417 claim 11, as S-0501 claim 48 notes), so its pad is a free dock. The house-wide pad capacity (S-0501 claim 30, S-0407 claim 16 and F3: refused when D ≥ T) is tested only for a copy queued behind an item, and S-0404 claim 3 says it "does not replace the free-dock factory-eligibility test". While every docking place is in contact, the order is ignored even as a request to queue; that is the "blocked" the lead saw. The icon stays lit either way (S-0411 claim 3). A pad-bound aircraft that comes back with no pad granting contact crashes (S-0501 claim 46) | S-0404 claim 3 and Terms (*free dock*); S-0407 claim 16, F3; S-0501 claims 30, 46, 48; S-0213 table T3 (b); S-0417 claim 11; S-0411 claim 3 | [Q-2172](../../spec/requests/Q-2172-pad-bound-aircraft-built-while-others-fly.md) (filed by the main session: is it a quirk, and its off position, so the play-fixes layer can admit what the lead expected); answered by S-0407 r29 claim 16 *At issue*: the original makes the test at issue, so no switch is needed | **Fixed by M8 A3 intake 57** (ledger [intake57-S-0407-S-0607-S-0307-S-0215-S-0408.md](intake57-S-0407-S-0607-S-0307-S-0215-S-0408.md)): every build order, the sidebar click included, makes claim 16's test when it is issued, with the line's copies then (`econ::tech::refused_at_issue`). A dock-bound aircraft type is refused while *D* ≥ *T*, an aircraft in flight counted like one on its pad, and the refused click gives no order and plays no sound. Tests `claim_16_pad_capacity_is_tested_at_issue_with_the_aircraft_in_flight` (T12 run 8 a: both planes out, every pad empty, no order), `claim_16_at_issue_counts_the_line_copy` (run 8 b), `claim_16_idle_line_counts_present_aircraft_in_flight_or_docked` (replaces the old test) and `claim_16_refusal_sound_only_after_a_changed_count`. The icon's darkening while *D* ≥ *T* (S-0411 claim 3, run 8 d) waits on S-0411's revision: the icon stays lit, and the click does nothing. Before intake 57: `claim_16_pad_capacity_is_tested_only_at_queue_time` (`claims_s0407.rs`) pins it. With T = 1 and one pad aircraft in flight, the idle line starts a copy; a second copy is refused at queue time; with the aircraft in contact with the pad, the order is ignored. Not a play-fixes candidate under C7: no switch exists and no spec states an off position. The lead's "expected" would make the pad-capacity test also apply to an idle start. That would be our own rule, and it limits what a house can build (D14), so it needs the lead's ruling. If the lead wants it, it would be a new `// design:` switch in the layer, as the pathing exception of 2026-10-29 allows for pathing only |
| 36 | Anti-air units stay on a missile that is gone | Departure from an approved claim: S-0309 claim 27 (r51, Q-2006) drops a held target from every holder (infantry, vehicle, building, aircraft) the moment the object is destroyed, goes off the map or is removed. Main drops a removed target only from aircraft holders (`objects::remove` → `air::targeters_lose`). A rocket missile that ends at its impact is removed, not destroyed, so an anti-air vehicle holding it kept it for ever, in *guard*, facing its last place | S-0309 claim 27 (r51), Q-2006 | none (Q-2006 answered) | Fixed by M8 A3 intake 52b (merge d0b04cb; ledger [intake52b-S-0309-S-0405-M-0005-M-0006-M-0016.md](intake52b-S-0309-S-0405-M-0005-M-0006-M-0016.md)): S-0309 claim 27, every holder of any kind drops a target that is destroyed, goes off the map or is removed, with the timer draw. `claim_27_a_removed_missile_is_dropped_by_its_anti_air_holder` (`claims_s0309_removed_missile.rs`) is un-ignored and passes; intake 52b adds `a_detonated_missile_is_dropped_by_a_ground_anti_air_holder` (`claims_s0506.rs`). By eye: an IFV that fired at a V3 rocket turns to other targets once the rocket lands |
| 37 | Units ignore attack orders and disengage, or just stop | Departure from an approved claim: S-0307 claim 19 makes the approach search's clear-cell test use "the actor's ... current reach region, determined from its actual cell and its current ground/deck layer". For a deck that is S-0212 claim 4's read through the bridge link. The code read the actor's ground region whatever its layer (`regions::same_region` ignores both layers), although ledger [S-0307](../m4-claims/S-0307.md) row 19 said it was built. An attacker on a high-bridge deck over water got the water's "no region", so every candidate failed, and claim 17 dropped the target: the unit fell to *guard* | S-0307 claims 15, 17, 19 (RG18 run c); S-0212 claim 4; S-0215 claim 37 (a, d) | [Q-2171](../../spec/requests/Q-2171-approach-candidates-on-a-deck-over-water.md) (two residuals: the candidate side, and claim 37 a's gate); answered by S-0307 r20 claim 19 (questions 1 to 3) and S-0215 r91 claim 37 a1 (question 4) | Fixed, and the residuals built by **M8 A3 intake 57** (ledger [intake57-S-0407-S-0607-S-0307-S-0215-S-0408.md](intake57-S-0407-S-0607-S-0307-S-0215-S-0408.md)): claim 19 now reads a high-bridge candidate for its deck too (`combat::range::approach_search`), so the deck cells over water qualify; claim 37's per-tick gate is S-0212 claim 6's comparison with both decks read (`approach::reaches`, `regions::compare`). Tests `deck_candidates_are_read_for_their_deck` (RG18 e), `a_deck_over_other_ground_is_read_for_its_deck` (RG18 c), `the_per_tick_reach_test_reads_decks` (B32 h), `the_per_tick_comparison_shares_no_region` and `the_per_tick_comparison_exceptions` (`claims_s0212.rs`). The round 6 fix: `combat::range::approach_search`'s clear-cell test asks `regions::same_region_read` (the actor's cell read for its deck when it stands on one, the candidate's ground), and `same_region`'s doc no longer claims that read. Test `an_attacker_on_a_deck_takes_its_reach_region_from_the_deck` (`claims_s0212.rs`, on the river fixture's enabled high bridge): the attacker on the deck keeps its target and gets a firing cell on the far bank. RG18's control is a link that is not enabled (banks in two regions): there the target is dropped. The test fails before the fix (target dropped, no destination). Replay of `path-46317-1.gpath` at 48a9516 (lane `n37`, `n37-dbg*`; window 45000–46317, 27 orders, selection #6867 #6442 #7070 #5074 #5192 #7259 #5729 #5137 #4885). At tick 45461 the group was ordered to attack #1077, an enemy outpost at about (81, 80). Five of them (#7259, #5729, #7070, #5074, #6442) stood on the deck of a high bridge over water at (96, 71)/(96, 72). They dropped #1077 alive between ticks 45529 and 45867 by claim 17, every candidate failing only the region test. The other four kept it until it was destroyed. With the fix applied to that build (lane `n37-fix`, `n37-fix2`; the replay diverges from tick 45021, once the fix first acts), #5074, #6442, #7070 and #7259 get firing cells, drive to (83, 83)–(84, 85) and destroy #1077. #5729 stood on another deck over water at (95, 88) when the order came. Its behaviour was still *move* on the next tick, and S-0215 claim 37 (d)'s table dropped the target there (its per-tick gate, claim 37 a, reads the ground's region under the deck: Q-2171 question 4). Residual at round 6, the specs gave it (Q-2171 asked; built by intake 57): a target standing on a bridge over water, whose in-range candidates are all deck cells, was still dropped. Claim 19 tests a candidate's ground region, and the water under a deck has none. In the fixed replay, #5074 drops #7913, which stands on the deck at (91, 90) |
| 38 | Some ore is not rendered / invisible | Departure from S-0112 claim 4 (an ore or gem cell draws the body of its tiberium's run at P + ((x·y) mod F) on flat cells, or the sloped position, not its stored overlay type). The drawing was right, but the map's asset list (`build::wanted_files`) read only the sprites of the overlay types the map stores. A map that stores one type for most of its ore (map 0: 466 of 478 cells of the first tiberium store its first body) then had no sprite for most bodies, and claim 9 draws nothing for them. Ore that grows or spreads in play (S-0401 claims 9, 11, 15) hit the same gap | S-0112 claims 4, 9, 10, Terms; S-0401 claims 9, 11, 15; S-0009; S-0520 claim 3 | none | Fixed: `wanted_files` now also reads every body of every tiberium's run (F flat and S sloped positions from P, `tiberium_run_files`), and `game_files` reads the crate overlay types' sprites (a crate placed in play, S-0520 claim 3; a skirmish map's own crates never bring them). Lane, map 0: tiberium overlay sprites read 4 of 72 before, 72 of 72 after; crate sprites 0 of 2 before, 2 of 2 after (walls were 8 of 8). Of 23 cells regrown round an ore mine, 19 drew nothing before and 8 after; a strip of 12 cells at stages 0 to 11 drew 2 before and 11 after; the live overlay items changed 429 pixels before and 3138 after. The 8 regrown cells that still show nothing lie east of the mine, probably behind its pictures (not checked). Low stages are small by the data (the frame is the stage, claim 4), and new ore near a mine grows at stages 3 and 4 in the lane: that is the game's rule, not drawing. Tests `a_maps_ore_reads_every_body_of_its_run`, `game_files_hold_crate_sprites` (`mapview/tests.rs`). Lane `r6_render_lane.rs` (`note_38_ore_near_mines`), frames `render/n38-*.png`, before in `render/before/` |
| 39 | Terror drone legs do not move when terror drones are in motion | Departure from S-0102 claim 22 (a sprite vehicle's frame comes from its walk counter, which grows on ticks that are multiples of the walk rate while it moves, or of the idle rate while it stands; a moving vehicle shows its walking row). The client kept no walk counter, so every sprite vehicle stood in its standing frame (development.md said so). The lane's terror drone (chosen by a parasite warhead) is a sprite vehicle | S-0102 claim 22 and its Import defaults; S-0107 *is moving now*; S-0105 claim 16; S-0402 F8 | [Q-2176](../../spec/requests/Q-2176-the-walk-animation-counter-of-a-voxel-vehicle.md) | Fixed: the view keeps a walk counter per vehicle (`mapview::walk`), advanced once per tick in `Live::take_effects` by claim 22's rule from the type's `WalkRate` (absent 1) and `IdleRate` (absent 0), with *moving* read as S-0107's *is moving now* (`occupancy::moving_on_tick`). The sprite vehicle frame table (`build::sprite_vehicle_frame`) reads it: walking row while moving, standing row or the facing's standing frame otherwise. Q-2176 answered by S-0102 r32 claim 22a (built at intake 58, [intake58-S-0102.md](intake58-S-0102.md)): the one counter serves sprite and voxel bodies and the harvesting image, advances on the locomotor's *moving now* (not S-0107's is-moving-now read above), and the walking row follows *movement in progress*; the interim is retired. Counter outside the game state, restarted at 0 on a save, load or join (the spec's rule). No rendering quirk switch. Lane, map 0: the drone moved on 38 of 40 ticks and showed 7 body images (39 changes) after; in the before frames it holds one standing pose throughout (by eye). Tests `moving_steps_on_multiples_of_the_walk_rate`, `standing_steps_only_with_an_idle_rate`, `a_zero_walk_rate_steps_at_tick_zero_alone`, `a_sprite_vehicle_walks_through_its_row_while_it_moves` (`walk.rs`), `a_vehicles_walk_counter_runs_while_it_moves` (`live_tests.rs`). Lane `note_39_walking_vehicles`, frames `render/n39-16-*.png` (before in `render/before/`). The lane's second parasite type (a sea drone on land, which cannot move) now changes image while it stands; by claim 22 that needs a non-zero idle rate, which the lane did not check |

**By eye (round 6).**
- Note 37: put a group of tanks on a high bridge over water and order them to attack a building or a unit off the bridge, out of range. They drive off the deck towards it and fire. Before, they fell to guard on the bridge. Since intake 57 a target on a long bridge over water is kept too: the attackers on the deck drive to a deck cell in range (Q-2171, S-0307 r20 and S-0215 r91).
- Note 35 (since intake 57, S-0407 r29): with as many pad planes as pads, out attacking or on the pads, clicking the plane icon does nothing and makes no sound; once one is lost, or a pad is added, a new one can be bought. The icon still looks lit while the click does nothing (S-0411 claim 3, waiting on its revision).
- Note 36: once intake 52b is merged, an IFV that shot at a V3 rocket turns to new targets after the rocket lands, instead of staying on its last position.
- Note 38: start a skirmish on a map with wide ore fields and look over them: every ore cell shows a crystal heap, in mixed shapes, where before most cells of a field were bare ground. Let the miners clear the ore next to an ore mine: new ore shows on those cells as it grows back (small at first). Gems look the same way.
- Note 39: build a terror drone and send it a few cells away: its legs step while it walks and stop when it arrives. Other walking sprite vehicles do the same. A miner's harvesting picture still plays while it harvests.

## Round 7 (2026-11-01, the lead's builds 48a9516 and 6267666; lane `local-lane/m8/a4-r7/`)

The lead's playtest notes of 2026-11-01, native, *Pathing and other fixes* on: "planes use the
landing animation when they die ... in the orig. game, they have a dying animation" (build
48a9516) · "these buttons should use graphic symbols" · "established waypoints from buildings do
not show" · "it's impossible to tell how much power is being used vs available" (build 6267666).
Each was checked on main at c70fc49 with the lane `crates/app/tests/r7_lane.rs` (map 0, the
windowless client, types chosen by their fields); frames before in `local-lane/m8/a4-r7/before/`,
after in `local-lane/m8/a4-r7/after/`.

| # | Note | Class | Specs | Request | Status |
|---|---|---|---|---|---|
| 40 | Planes use the landing animation when they die | Two parts. Simulation: the specs give what the lead saw. A dead aircraft whose ordinary movement is still active keeps its horizontal motion toward its destination while it falls (S-0501 claim 9, AC5), so one shot down on its way home keeps heading for its pad. Lane: a jet shot down at tick 275 at height 1329 kept moving 35 leptons a tick toward its destination cell (38, 71), the fall counter grew by 1 a tick, and it left play 28 ticks later. Drawing: a departure from S-0113 claims 16 and 18. A crashing aircraft tumbles: its pitch and roll rates come from S-0312 claim 16's three draws by F5, and the flying locomotor draws the angles while crashing. The simulation took the draws and dropped them, so the view drew the falling jet level, as it flies, nose along its course: the "landing" look. No separate death animation or debris for an aircraft is in the specs; damage smoke (S-0501 claim 37) names no animation | S-0501 claim 9, AC5, claim 37; S-0113 claims 5, 16, 18, F5; S-0312 claim 16; S-0105 claim 42; S-0504 F5 | [Q-2236](../../spec/requests/Q-2236-how-a-crashing-aircraft-body-is-drawn-with-its-tumble.md) (how the tumble composes with the flight heading); [Q-2237](../../spec/requests/Q-2237-which-animation-an-aircrafts-damage-smoke-shows.md) (which animation the smoke shows) | Fixed (drawing). The simulation reports the three draws (`Effect::CrashTumble`, presentation only, not saved or hashed; from `air::start_crash_hit` and `jump::fly::start_crash`, the draws unchanged). The view sets both rates by F5 (`rocking::crash_rates`) and steps the body by table S's crashing row, for aircraft and jumpjets; `build.rs` draws a crashing aircraft with S-0504 F5's transform on the flight heading (`voxel::crashing_body_pose`), its shadow on the heading alone, our design under Q-2236. A flying aircraft is still drawn without rocking (claim 18). Smoke: none drawn until Q-2237 is answered. **Answered (M8 A3 intake 67, [intake67-S-0501-S-0113.md](intake67-S-0501-S-0113.md)).** Q-2236 by S-0113 r17 claim 20, which replaces our design in five places: the body is Rz(θ_flight) · Rx(φ) · Ry(ψ′) with no lift and no forward or sideways shift (F5's are gone; the order, roll outermost, was already F5's); ψ′ adds the type's drawing-pitch angle α when the speed fraction is above the drawing-pitch threshold (summed in double, stored in single, α not narrowed first), and the bank never enters; at zero or negative height the body is turned to its heading only; the tumble is drawn at every angle size (no 0.005-radian settled test) and with the exact angles, not held to the view's 0.005-radian drawing step; its images are kept apart under the F1 table steps of ψ′ and φ, which fix the drawn transform (drawn fresh). Kept as round 7 had it: the heading is the aiming facing, the shadow the heading alone with level flight's key. Q-2237 by S-0501 r65 claim 37: a shown smoke is one animation of the fixed damage-smoke type (the same for living and dead aircraft and every type), created by the simulation in the aircraft's turn right after the chance draw, at the aircraft's position after its movement, not attached, no loop delay, repeat factor 1; its creation draws come from the scenario stream, so this is a simulation change. Lane frames `local-lane/m8/intake67/after/` (27 distinct body images over a 27-tick fall, 17 smoke animations). The smoke's creation draws change AI games: 21 of the quick lane's 36 games end at another tick, 4 with the other winner (intake 67 ledger, *Lanes*). Round 7's lane, before intake 67: while falling, 28 distinct body images (27 changes) after, 2 (1 change) before. Simulation unchanged: the M6c, M7a and M7b sweeps, the arena and the AI quick lane match before and after. Tests `the_crash_rates_follow_f5`, `a_crashing_aircraft_tumbles_by_its_draws`, `a_flying_aircraft_is_drawn_without_rocking` (`rocking_tests.rs`), `aircraft_attitude_is_drawn` (`live_tests.rs`), `crashing_pose` (`voxel.rs`); claim ledger rows in [intake53-S-0107-S-0108-S-0113.md](intake53-S-0107-S-0108-S-0113.md) |
| 41 | The command and tab buttons should use graphic symbols | Round 7: specs silent; S-0411 then made the interface's look the implementer's design and named no images for these buttons. Intake 69: S-0411 r28 claim 19 (answering Q-2238) names the install's image file of the sell, repair, four tab and two scroll buttons, their sidebar palette and the frame for each state; the power button stays ours | S-0411 claim 19, Import mapping; S-0005 claims 13, 13a, 22 | [Q-2238](../../spec/requests/Q-2238-the-sidebar-command-and-tab-button-images.md) (answered by S-0411 r28); [Q-2266](../../spec/requests/Q-2266-the-sidebar-button-images-transparent-index-and-display-colour.md) (index 0 and the colour depth of these images) | Fixed (intake 69). The sell, repair, tab and scroll buttons draw the install's images, resolved with the local player's side group, with the sidebar palette and claim 19's frame per state (`game::sidebar_art`, `hud::tab_frame`, `mode_frame`, `scroll_frame`); each cell is the image's canvas (our layout, `Layout::with_buttons`). Round 7's symbols (`hud::symbol`, `symbol_button`) stay for the power button and for any file or frame that does not resolve, and with no install art. Tooltips, hit areas and focus unchanged. Tests: round 7's `command_and_tab_buttons_show_symbols_and_name_on_hover` (no art), and the claim 19 rows of [intake69-S-0411.md](intake69-S-0411.md). Lane shots `local-lane/m8/intake69/before` and `after`, per side |
| 42 | Rally points set from buildings do not show | Specs silent on drawing. S-0213 claim 20 makes a factory's guard spot its rally point, and S-0214 Terms names the rally-capable buildings; no spec draws it. The lane set it (none before, cell (49, 71) after the click), and nothing showed | S-0213 claim 20; S-0214 Terms | none | Design: while the local player has a building selected whose guard spot is a rally point (`targeting::holds_rally_point`: rally-capable or a unit factory), the view draws a translucent yellow diamond on the rally cell and a dashed yellow line from the building to it (to the object, when the guard spot is one). Nothing for other players' buildings or when not selected. Test `a_selected_factory_shows_its_rally_point` (`client_play_notes_tests.rs`). Lane frame `n42-rally-zoom.png` |
| 43 | Power used against power available cannot be told apart | The specs give the bar. S-0405 claim 19 and F4: the meter's extent is max(output, drain) on a base-dependent scale; the supplied part is the output, then up to 100 units of unmet drain, then the rest. With full power only the supplied part remains, and the meter must not become a drain/output fraction. So at full power the drain is, by the spec, not shown on the meter: what the lead saw. Layout and colours are our design (S-0411) | S-0405 claims 4, 19, F4; S-0411 | none | Design on top of claim 19, the extents unchanged: at full power a 1-pixel light mark inside the green part at the drain's place (`hud::drain_mark`, drain × supplied ÷ output when 0 < drain < output), and on hover a tooltip "Power / Output *O* / Drain *D*" (`hud::power_lines`). Lane: full power (output 200, drain 25) parts [50, 0, 0] with the mark; short of power (output 200, drain 375) parts [43, 21, 16]. Test `the_power_bar_marks_the_drain_and_names_the_totals` (`hud.rs`). Lane frames `n43-power-*-zoom.png`, `n43-power-*-hover-zoom.png` |

**By eye (round 7).**
- Note 40: shoot down a jet in flight (for example with anti-air under its path home). It rolls and pitches more and more as it falls, a different way each time, and keeps sliding along its course until it hits the ground. Before, it fell level, nose along its course. Its shadow keeps the plane's heading. No smoke yet (Q-2237). *After intake 67:* the falling jet tumbles about its own axes, and its body no longer shifts off its place as it rolls and pitches (F5's lift and shifts are gone); its type's flight pitch is added to the tumble while it still has speed, and it does not bank in a turn; a damaged jet (below half strength) leaves puffs of smoke behind it that stay where they appeared, and a falling one leaves many more.
- Note 41: the sell, repair and power buttons and the four tabs show small symbols instead of words; hover one to read its name. A disabled one is dimmed.
- Note 42: select a war factory or barracks, then set a rally point: a yellow diamond marks the cell and a dashed yellow line runs from the building to it. Deselect it and they go. Selecting it again shows them again.
- Note 43: at full power a thin light tick in the green bar shows how much of the output is used; hover the bar for the output and drain numbers. Short of power, the bar shows the green supplied part, the orange first 100 units short and the red rest, as before.

## Round 8 (2026-11-05, the lead's build 6267666; lane `local-lane/m8/a4-r8/`)

The lead's bug reports of 2026-11-05 from the bug-report page, build 6267666, *Pathing and other
fixes* on for all; notes 44 to 54 (`local-lane/m8/a4-r8/notes.txt`). The attached screenshots
reached us only as small thumbnails.

### Round 8, rendering and UI

Notes 45, 46, 47, 49 and 54, each checked on main at e782d3b (main's tree after the lead's
rewrite of intake 102 differs only in spec and docs files) with the lane
`crates/app/tests/r8_render_lane.rs` (map 0, the windowless client, types chosen by their fields);
frames before in `local-lane/m8/a4-r8/render/before/`, after in `local-lane/m8/a4-r8/render/after/`.

| # | Note | Class | Specs | Request | Status |
|---|---|---|---|---|---|
| 45 | "ship attacked by the giant squid, squid cannot be seen" | The specs give what the lead saw. An attached parasite stays off the map and travels with its host (S-0614 claim 3), so nothing of the squid is drawn while it holds the ship. The held ship shows its roll during each push (S-0113 claim 14, F4), the wakes at each push start and the splashes at each strike (S-0614 claim 10), and these are drawn. The Import mapping names a grab animation "identified by a fixed engine constant" but does not identify it, so we cannot show it; that gap is the only picture of the squid while it holds a ship. Lane, for a squid of ours and of the enemy's on a ship: from the latch on, the squid is off the map, attached, and has no drawn item; the ship keeps its items and is hit every strike | S-0614 claims 3, 10, Import mapping; S-0113 claim 14, F4 | [Q-2671](../../spec/requests/Q-2671-the-grab-animation-of-a-grappling-parasite.md) (which animation, when, where, which layer, how long, which random stream) | Recorded; the grab animation waits on Q-2671. Lane frames `n45-ours-t*-zoom.png`, `n45-theirs-t*-zoom.png` |
| 46 | "weird graphical glitch with the octopus" (thumbnail: dark water) | Not reproduced as a departure. Every phase of the squid scene was shot every 6 ticks for both owners (approach, latch, grab, pushes, strikes, the ship's sinking, the release) and no wrong frame, palette, facing or layer showed. Two things in those frames match the thumbnail and follow the specs: (a) during each push the held ship rolls to its side, up to a quarter turn (S-0113 claim 14, F4), and the freshly drawn rolled hull shows one-pixel gaps between its voxels (S-0105 claim 22, `quirks.voxel_single_pixel` on, its off state not implemented); (b) the owner sees a cloaked, submerged squid half translucent over the dark water (S-0516 claim 28); the enemy does not see it | S-0113 claim 14, F4; S-0105 claim 22; S-0516 claim 28; S-0614 claims 4, 13 | none | Recorded, not reproduced. If the lead's glitch is something else, a full-size screenshot or a save would let us find it. Lane frames as note 45 |
| 47 | "hovercraft doesnt have a shadow" | The specs are silent. S-0105 claim 40 gives the shadow point of aircraft only, and vehicle shadow points are open (S-0105 Open questions, S-0102 claim 18). We drew a vehicle's shadow around the body's own lifted point. A hovering vehicle stands above its surface, so its shadow lay under its own hull, and claim 39's own-image exclusion (which applies to hovering vehicles) took out every shadow pixel the hull covers: a sliver showed. Lane: on land and on water the hovering vehicle stands above its surface and has one shadow item, before and after; after, its shadow item sits lower under its body than a driving vehicle's | S-0105 claims 39, 40, Open questions; S-0102 claim 18; S-0101 claims 1, 3 | [Q-2672](../../spec/requests/Q-2672-the-shadow-point-of-a-vehicle-above-its-surface.md) (the shadow point of a vehicle above its surface) | Design under Q-2672 (`build::vehicle_shadow_point`): a vehicle above its surface casts its shadow at the projection of its horizontal position at that surface's height, as claim 40 places an aircraft's; claim 39's exclusion compares each shadow pixel with the body, turret and barrel images at their own screen position (`vehicle_shadow` takes the parts' shift from the shadow's point; the cutout memo is keyed by it). A vehicle at or below its surface is unchanged. Tests `a_lowered_shadow_excludes_the_parts_where_they_land`, `a_vehicle_above_its_surface_casts_its_shadow_on_it` (`build.rs`). Lane frames `n47-land-zoom.png`, `n47-water-zoom.png`, `n47-*-moving-zoom.png` |
| 49 | "unit promotion badges show up even when they are garrisoned" | Not reproduced for a garrison; one neighbouring departure found and fixed. Paths checked: (a) an elite occupant inside a civilian building is off the map (S-0508 claim 5) and draws no item at all, insignia included, outside, hovered, selected and after a fight; the building shows only its occupant row (S-0111 claims 11 and 12, the occupant's type's pip frame, not its veterancy) and has no experience of its own; (b) a vehicle in a vehicle bunker stays on the map and is drawn (S-0606 Terms), so its insignia shows, as S-0111 claim 18 gives "every unit"; (c) a hidden cloaked enemy shows no insignia, its owner sees it (claim 18, S-0516 claim 28): as specified. (d) Departure: claim 18 also says no insignia while the unit appears disguised to the local player's house (S-0516 claims 35 and 36); `draw_status` did not check it, so a veteran spy in disguise showed its badge to the players it fools. Seen on the side: of the two elite occupiers sent in, one dropped its order within 50 ticks and stayed outside, badge showing (a simulation matter, passed to the simulation round with note 53) | S-0111 claims 11, 12, 18; S-0508 claim 5; S-0606 Terms; S-0516 claims 28, 35, 36 | none | Fixed (d): no insignia while the unit appears disguised to the viewer (`live.rs` `draw_status`). Test `claim_18_a_disguised_unit_shows_no_insignia_to_whom_it_fools` (`live_tests.rs`). (a) to (c) recorded. Lane frames `n49-*-zoom.png`, `n49-cloaked-*-zoom.png` |
| 54 | "units do not get a little name tag when hovered long" | The specs are silent. No spec gives hover tips over the battlefield (S-0105 Open questions: "This spec does not specify hover tips"); S-0411 gives only the sidebar's. No string table is read for display names yet | S-0105 Open questions, claim 43; S-0111 claim 3; S-0516 claim 35; S-0110 claim 8; S-0004 (display name `UIName`) | [Q-2673](../../spec/requests/Q-2673-the-name-tip-of-an-object-under-a-resting-pointer.md) (wait, text, objects, place, look, end); [Q-2674](../../spec/requests/Q-2674-how-a-types-display-name-is-resolved-from-the-string-table.md) (display names from the string table) | Design under Q-2673 (`Session::hover_tag`, `hud::name_tag`): once the pointer has rested 700 ms of interface time on the same unit or building (the object the hover health bar marks), a one-line tag with its name shows below and right of the pointer in the sidebar tooltip's look, moved left or up to stay inside the battlefield. None for terrain objects, for an object in a cell shrouded for the local player, for a unit that appears disguised to the local player (S-0516 claim 35), while the options menu is open, the keyboard aims, a box is dragged or the right button is held. The name is the one the sidebar shows, the type's name, until Q-2674 gives display names. Lane: no tag at once and at 400 ms, a tag from 800 ms on, none once the pointer leaves. Tests `the_name_tag_waits_on_one_object` (`session.rs`), `a_name_tag_sits_by_the_pointer_inside_the_battlefield` (`hud.rs`). Lane frames `n54-0-at-once-zoom.png`, `n54-1-two-seconds-zoom.png` |

**By eye (round 8, rendering and UI).**
- Note 45: send a giant squid at an enemy ship. Once it grabs the ship the squid is gone from view; the ship rocks from side to side, with wakes, and splashes at each hit, until it sinks or the squid lets go. That is the spec's picture; the squid's own grab animation waits on Q-2671.
- Note 46: watch the same scene closely. The ship rolls far over at each push and its hull shows small gaps between its pixels while it is rolled; your own squid shows half see-through under the water. Both are as specified. If what you saw was different, please send a full-size screenshot or a save.
- Note 47: build a hovercraft and park it on land, then on water. Its shadow lies on the ground or the water under and a little in front of it, clearly visible, and follows it as it moves. Before, only a thin dark edge showed at its hull.
- Note 49: put veteran or elite infantry into a civilian building: the badges go with them, and the building shows only its row of occupant figures. A veteran tank in a tank bunker keeps its badge (it is still on the map). An enemy veteran spy in disguise shows you no badge; your own disguised veteran spy still shows you its badge. If a badge showed on a garrisoned building for you, a screenshot or a save would let us find which case it was.
- Note 54: rest the pointer on any unit or building, your own or the enemy's, without moving it. After well under a second a small dark box with the unit's name appears beside the pointer; near the screen's right or bottom edge it moves to the other side. Move off the unit and it goes. The name is the internal type name (a display name needs Q-2674). An enemy spy in disguise gets no tag, and nothing under the black shroud does.

### Round 8, simulation and AI

The lead's words: "the plane from the ship which was being attacked by the squid flew off the
map" (note 44) · "ai takes too long to capture oil derricks" (48) · "planes can be given an
attack command even when they havent reload their ammo ... they fly, then land back, even further
delaying ammo reload I assume. AI keeps doing this too" (50) · "ai engineers keep entering own
buildings even when they are not critical buildings nor on low health. wasteful" (51) · "ai
attacks with all its units directly at the beginning of the game. this is not a sane strategy
*especially* when there are more than 2 players" (52) · "buildings garrissoned by infantry do not
fire ... it started working at some point" (53). Each was checked on main at 40c2092 with the
round 8 scenes of `crates/app/tests/m7b_lane.rs` (`r8-carrier-squid`, `r8-pad-no-ammo`,
`r8-garrison-fire`) and, for the AI notes, the quick lane (`ai-sides`, seeds 7 to 10, maps 0, 2
and 8: 36 games), whose report now prints the round 8 counts (`R8Watch` in
`crates/tools/src/skirmish_lane.rs`), and a four-house free-for-all (`four_houses` with
`GLASNOST_FH_TEAMS=1,2,3,4`, map 30, seeds 1 to 3, 30 000 ticks). Lane outputs are in
`local-lane/m8/a4-r8/sim/`.

| # | Note | Class | Specs | Request | Status |
|---|---|---|---|---|---|
| 44 | A carrier's plane flew off the map while a giant squid held the ship | Not reproduced on main. The lane puts a carrier ship on open water (map 3), sends its slot aircraft at an enemy vehicle and has an enemy grappling parasite take the carrier: held (runs 1 and 4), held and killed (runs 2 and 5), the grappler 10 cells or 3 cells away. In none of the six runs did a slot aircraft leave play alive outside the usable area. Each stayed within 14 cells of its carrier and, with the carrier gone, came down or left play as S-0506 gives (a slot aircraft whose carrier is held or gone returns or is lost). The lead's build is 698 commits behind main; the slot aircraft's and the edge rules changed in that time (S-0501 r93 and r132: `aircraft_edge_hold`, `aircraft_offmap_stop_start`, both in the layer the lead played with) | S-0506 claims 9 to 13; S-0501 claims 10, 17, 34, 35, 45, T4; S-0614 claims 7 and 8 | none | Recorded: not reproduced on main. If the lead sees it again on a build of today, a report with the game file would let us replay it |
| 48 | The AI is slow to capture oil derricks | Our AI (D14). The engineer step ordered an engineer only when the house had none and could pay twice its price, and only for a building already in sight. With no early wave (note 52) the house also explored late: on map 0 a derrick was first seen at ticks 10 000 to 25 000 and held from 30 000, or never | none (our design) | none | Fixed (`special::engineers`, `scripted::send_scout`): every paying tech building in sight with no engineer bound for it wants one, up to `TECH_ENGINEERS` at once, queued at once whatever the credits (`tech_buildings::paying_in_sight`); an idle army unit within `ESCORT_REACH` cells attack-moves to the building with the engineer; while no wave goes, the fastest waiting army unit (not of the guard) scouts the unexplored cell of the house's own side (nearer home than any enemy start) nearest home, one at a time. Quick lane, seeds 7 and 8 (the counts were added for this round): houses holding a cash tech building at tick 9000, 16 of 36 before, 25 of 36 after; seeds 9 and 10 after, 24 of 36. On map 0 the derricks are now first held at about ticks 5 000 to 7 500 in most games (before: about 30 000 or never); those first seen as held by the enemy still wait for a capture. Tests `queues_an_engineer_for_a_paying_tech_building_at_once`, `keeps_up_to_two_engineers_for_paying_tech_buildings`, `an_engineer_for_a_tech_building_takes_an_escort` (`special.rs`), `a_scout_explores_the_houses_own_side` (`ai.rs`) |
| 50 | Planes take an attack order before they have reloaded, take off and land again; the AI does it too | Two parts. Player: the specs give it. S-0214 claim 6 (e) offers the attack, and a click gives the order, while the aircraft is out of ammunition (rank 7 is not *illegal*). On its pad, claim 28 (`quirks.aircraft_half_reload_with_target`, on in the reference and in the layer) serves an aircraft with a target only up to ⌊*A* / 2⌋ and sends it out once it has that many. For a type whose half rounds down to nothing that is at once, with no round. Claim 17 then ends its run at once (no ammunition: an aircraft of a human-controlled house forgets its target), and it falls back to its pad (claim 18). Lane, the pad scene, both with and without the play fixes: a plane that spent its rounds and slept on its pad took off again on the order, climbed, turned back and landed, and was full 912 ticks after it had spent its rounds, against 642 for the control with no order; an order given while it flew home changed nothing (642). AI: our own (D14), which gave the order | S-0214 claim 6 (e); S-0501 claims 17, 18, 26, 28, T1, T3, Edge cases (*Half reload with a target*) | none | Player: recorded as the specs give it. The switch's off position (reload to full, then take off) is stated and built but fails the layer's condition 3: it holds back every aircraft with a target and more rounds too. Judged in [c7-play-fixes.md](c7-play-fixes.md), *A4 round 8*. An off position for a type with no round to take off with would be our own rule and needs the lead's ruling. AI fixed: `brain::think_with` drops an attack or attack-move order to an aircraft with no ammunition (`air::ammo_left`), and `defend` skips such aircraft. Test `no_attack_order_to_an_aircraft_with_no_ammunition` (`ai.rs`) |
| 51 | AI engineers enter their own buildings that are neither important nor badly damaged | Our AI (D14). The engineer step sent an engineer to repair any own building below full health | none (our design) | none | Fixed (`special::worth_repair`): an engineer repairs an own building only when it is a construction yard, refinery, factory or cash-producing tech building at half strength or less. Quick lane, seeds 7 and 8: engineer entries into an own building above half strength or of no economy or production role, 5 before, 0 after (and 0 in seeds 9 and 10). Test `an_engineer_repairs_only_an_important_badly_damaged_building` (`special.rs`) |
| 52 | The AI attacks with everything from the start; not sane with more than two players | Our AI (D14). Every idle army unit went with a wave as soon as six waited, from the first minute, at the enemy start next in a fixed cycle | none (our design) | none | Fixed (`scripted.rs`, item 5): no wave goes before `OPENING`; a guard of a third of the army (from `GUARD_MIN` to `GUARD_MAX`), its units nearest home, stays home; each wave asks for `WAVE_GROWTH` more units than the last (up to `WAVE_MAX`); a wave takes on the enemy house whose building stands nearest home, and while it sees no enemy goes to the nearest start that may still hold an enemy base. Quick lane, 36 games: shortest game 14 082 ticks before, 26 383 after; median about 42 600 before, 46 900 after; every game ends (an uncapped guard left one without a winner after 120 000 ticks, hence `GUARD_MAX`); the first push (4 army units beyond half way to the enemy yard) at a median tick of about 7 300 before and 15 300 after, with guards near home. Four houses, free for all, 3 seeds: 7 of 12 yards lost before (5 of them by tick 3 300, houses whose yard came down late), 2 after (ticks 8 485 and 14 729); first building lost at tick 1 023 at the earliest before, 3 988 after; 11 of 12 houses still stand at tick 30 000 after, 7 before. Tests `waves_wait_for_the_opening_and_leave_a_guard` and the wave tests, which now start past the opening (`ai.rs`), `a_wave_sends_its_jumpjets_across` (`jumpjets.rs`) |
| 53 | Garrisoned buildings do not fire, then start at some point | Departure from S-0309 claim 2: only an occupiable building that holds no occupants is kept from the passive search. Main kept every occupiable building from it, so a garrison fired only back at an attacker (retaliation, which is the "at some point"). Lane: with an armed enemy in range before the entry, the target came at +34 ticks, the tick the enemy's first shot hit the building, before; at +1 after, with the building not hit first. An unarmed enemy in range is not taken in either build (some row of table C rejects the lane's type; which one was not checked) | S-0309 claims 1, 2, table C; S-0508 claim 5; S-0308 claim 13 | none | Fixed (commit *sim: a garrisoned building searches passively*, `combat::acquire::may_search_passively`). Test `s0309_claim_2_a_garrisoned_building_searches` (`claims_s0508_fire.rs`). The synthetic M6c kit's neutral country is now of the civilian side (S-0414 claim 12, S-0508 claims 7, 8 and 24): before, a hut a player garrisoned stayed the neutral house's, and once a garrison searched it fired at both players. The side observation of the rendering round's note 49 (one of two elite occupiers dropped its order within 50 ticks and stayed outside) reached this round after its lanes and was not checked here |

**Lanes (round 8, simulation and AI).** Before (main at 40c2092) and after, seeds 1 to 16
each. M6c sweep: the smallest counter over the seeds 0 before, with 9 seeds below 3 (in four of them
the game ended early, at tick 3 326 at the earliest: the rush), 3 after, no seed below 3 and every
game played to tick 12 000; buildings captured median 4 before, 9 after. M7a sweep: smallest 0
before, 2 after (psychic sensor orders, 2 in most seeds as before; every seed before had a
counter below 3, several at 1). M7b sweep: the drain's smallest counter 3 before and after;
dominator launches median 7 before, 9 after; mutator launches median 5 before, 10 after; credits
drained median 7 440 before, 1 960 after (the drained building stands for a shorter time). The
three gate exercise tests (M6c, M7a with seed 17, M7b with seed 11) pass. The arena reads the
same as before, row for row (M-0021: 3 of 41 rows failing, as on main). The B-track exits in the
quick lane (36 games, 72 houses): houses that held two construction yards at once 23 before, 43
after, the first such tick at a median of 30 661 before and 28 420 after; the first tick a main
superweapon (chronosphere, dominator, genetic mutator, iron curtain, missile, storm) stood ready,
pooled over kinds and houses, 83 times at a median of 24 741 before, 107 times at a median of
22 209 after (the earliest 12 823 before, 14 023 after).

**By eye (round 8, simulation and AI).**
- Note 53: put three riflemen in a civilian building that lets them fire, with an enemy tank in range: the building opens fire at once, before the tank shoots it. Before, it fired only once the tank had hit it.
- Note 50: with your own planes nothing changes: an attack order given while a plane sits on its pad without rounds still sends it up and back down (the specified half-reload rule). The computer no longer does it: its planes that have spent their rounds wait on their pads until they have them again.
- Note 51: watch the computer's engineers: they go into a damaged construction yard, refinery, factory or oil derrick only when it is at half strength or less, never into a power plant or a defence.
- Note 48: in the first minutes the computer sends one fast unit round its own side of the map, and its engineers go for the oil derricks it finds as soon as they are seen, with a tank alongside when one is near.
- Note 52: the computer builds up for the first minutes, keeps a few units at home when it attacks, and sends larger waves as the game goes on. With three or four players it goes for the one whose base is nearest, not for everyone at once.

### Round 8b, rendering and UI (answers to Q-2671 to Q-2674)

The lead's answers replace the round 8 interims of notes 45, 46, 47 and 54: S-0614 r32 (Q-2671),
S-0105 r44 (Q-2672), S-0111 r34 (Q-2673) and S-0004 r29 (Q-2674). Each was checked with the same
lane, `crates/app/tests/r8_render_lane.rs`; frames before (main at a143ecc) in
`local-lane/m8/a4-r8b/render/before/`, after in `local-lane/m8/a4-r8b/render/after/`. The
departures these answers name all come without a switch: judged in
[c7-play-fixes.md](c7-play-fixes.md), *A4 round 8b*.

| Request | Notes | Answer | Built | Status |
|---|---|---|---|---|
| [Q-2671](../../spec/requests/Q-2671-the-grab-animation-of-a-grappling-parasite.md) | 45, 46 | S-0614 r32 claim 25, table G and Import mapping: while a grappling parasite holds a host it is not drawn; in its place it has at most one animation of a fixed engine type, created once per hold in the grab's first turn, attached to the host and drawn upright in the ground layer, its stage set by the cycle's phase frames | `parasite.rs` (`create_grab_anim`, `tend_grab_anim`, `grab_stage`, `remove_grab_anim`), with `anims.rs`, `anim_clock.rs` and `combat/effects.rs`. The animation type `SQDG` (an engine constant, found without regard to letter case; absent: none and no draw, the cycle runs as before) is created at the host's position with no loop delay, a repeat factor of 1 and no reverse request, attached with no displacement and attached again in a later turn that finds it detached. Each turn of the hold that reaches the attached-parasite step gives it the parasite's house-colour palette and the host cell's ground level. At each step of the phase frame *j* (1 to 9) it gets the stage *k* = *b* + 10 × *d* + *j* (*b* 0 in the grab, 80 or 160 in a push by its way, *d* the host's 8-direction) and a 128-tick stage clock restart. It is removed when the parasite leaves the host by any way, when the host is lost and when the parasite is removed, also when a strike's own hit destroys the host. Save format 107, AI game format 71 (the parasite's animation is game state). Tests `claim_25_the_grab_animation_is_created_once_in_the_grabs_first_turn`, `claim_25_an_absent_type_shows_none_and_the_cycle_runs`, `claim_25_the_stage_follows_the_cycle`, `claim_25_removal`, `claim_25_save_load_mid_hold` (`claims_s0614.rs`). Goldens: unchanged (regenerated with this round's simulation; the sim gate's scenes have no grapple) | Answered and built. Lane frames `n45-*-zoom.png`, `n46-*`: while the squid holds the ship, the after frames show the grab animation on the ship, upright and in the squid owner's house colour, where the before frames show the bare ship; one more animation is in play through the hold (38 against 37 at +396 ticks) |
| [Q-2672](../../spec/requests/Q-2672-the-shadow-point-of-a-vehicle-above-its-surface.md) | 47 | S-0105 r44 claim 44: a voxel vehicle's shadow draw point is its body's draw point *B* moved down by the lift of its signed height above its surface *a* (less 416 leptons on a bridge deck), with the same lift that lifts *B*; claim 39's own-image exclusion at the same screen position | `build::vehicle_shadow_point(point, above)` replaces the round 8 interim: *B* moved down by S-0101 claim 3's signed lift of *a*, so on a surface at a whole level the shadow is exactly the surface projection and on a slope within one row of it, *B* itself at *a* = 0. Our engine also uses it for the flying locomotor and claim 40's *P* for a hunter-seeker type (the spec's corrections, no switch). The lifted state of a vehicle carried by an aircraft (raised 14 rows, 60 more over a qualifying bridge) has no simulation state yet, so no vehicle shadow is raised (`notes::VOXEL_SHADOW_LIFTED`). Tests `a_vehicle_shadow_anchor_is_the_body_point_lowered_by_the_lift`, `a_lowered_shadow_excludes_the_parts_where_they_land` (`build.rs`), `a_vehicle_shadow_lies_on_its_surface` (`live_tests.rs`) | Answered and built. Lane frames `n47-*-zoom.png`: byte for byte the same before and after, on land and on water, parked and moving; on a surface at a whole level claim 44's anchor is the surface projection the interim drew |
| [Q-2673](../../spec/requests/Q-2673-the-name-tip-of-an-object-under-a-resting-pointer.md) | 54 | S-0111 r34 claims 20 to 26: the name tip shows while the tooltip option is on, once the pointer has rested on the battlefield view for 1000 ms of real time, for 10 000 ms; its text is chosen once per rest (the shroud label, the display name by kind and disguise, or a turret-tip vehicle's modifier and name); a black box with a one-pixel frame in the colour of the local side, 16 pixels below the pointer, kept inside the view; any movement, a mouse button, the end of its lifetime or the option off hide it | `game::name_tip` (the wait, the lifetime, the one choice per rest, the layout, placement and look), `Session::update_name_tip`, `choose_name_tip` and `name_tip_text` (claims 21 to 23), `build::tip_turret_index` (claim 23's *t* for a rotating-barrel type), the tooltip option in the settings (`tool_tips`, on unless turned off, a row in the options menu) and the middle button. It replaces the round 8 interim (`hud::name_tag`, `Session::hover_tag`), which is removed. Seven points the claims leave open are asked in [Q-2687](../../spec/requests/Q-2687-the-name-tips-open-points.md), each marked `design: Q-2687` (wrapping, the inset, a box as wide as the view, the further blocking view state, the options menu, a blocked wait end, the pointer over an interface element). Tests `claim_20_wait_lifetime_and_one_choice`, `claims_20_and_26_what_hides_it`, `claim_24_layout`, `claim_24_placement`, `claim_25_look_and_the_text_limit`, `claims_24_and_25_the_drawn_tip` (`name_tip.rs`), `claim_21_which_objects_get_a_tip`, `claim_22_the_name_by_kind_and_disguise`, `claim_23_the_turret_tip_modifier`, `claims_20_24_25_a_tip_shows_after_the_wait` (`client_name_tip_tests.rs`), `the_name_tip_index_of_a_rotating_barrel_type` (`turret_index_tests.rs`), `the_tooltip_option_is_on_unless_turned_off` (`settings_tests.rs`) | Answered and built; Q-2687 open. Lane frames `n54-*-zoom.png`: with the pointer at rest on a unit, no tip at +900 ms, the tip at +1000 ms (a black box 34 by 20 pixels, 16 pixels below the pointer, framed in the local side's colour) and still at +2000 ms; gone once the pointer moves. Note 49's after frames show the tip in place of the interim's tag |
| [Q-2674](../../spec/requests/Q-2674-how-a-types-display-name-is-resolved-from-the-string-table.md) | 54 | S-0004 r29 claim 17: a display name is the string-table text of the type's label, matched without regard to letter case after normalisation; a type with no label has an empty name; a missing entry shows a placeholder | `game::strings` (`StringTable`, `label`, `resolve`: the match, the normalisation, the first of two entries whose labels differ only in case kept, with a warning). The string table's file, layout and decoding and the placeholder's marker word are not given, so no table is read yet: asked in [Q-2686](../../spec/requests/Q-2686-the-string-table-file-its-decoding-and-the-missing-label-marker.md). Until then a display name shows our stand-in, the type's section name, and a fixed label shows itself. Tests `claim_17_matching`, `claim_17_normalisation`, `claim_17_labels_and_resolving` (`strings.rs`) on synthetic tables | Answered; built on synthetic tables, the table's reading waits on Q-2686 |

*Lanes (round 8b).* The sweeps, the arena, the AI tests, the quick lane and the render lane ran on
main at a143ecc before and this round rebased on a143ecc after, so they predate intakes 104, 99
and 105. The goldens and the checks ran on the round rebased on 5e9008b. The M6c, M7a and M7b sweeps (seeds 1
to 16) match seed by seed. The arena (`--record all --threads 2`) matches line by line apart from
timings: 15 of 155 scored pairings failing (12 of 96, 2 of 24, 1 of 35), M-0019 0 of 6, M-0021 10
of 50, M-0020 1 of 25 checked. `glasnost-ai --include-ignored`: the same tests, all pass. The AI
quick lane (seeds 7 to 10, maps 0, 2 and 8; 36 games): 31 games the same; 5 end at another tick,
all on map 0, the map with water, where grapples happen. Claim 25 gives the grab animation the
creation draws of S-0109 claims 3 and 13 from the scenario stream, so a game with a grab takes
other draws from then on. Of the 5, four keep their winner and one that had no winner by tick
120 000 is won by house 0 at tick 88 537 (seed 7). The goldens did not move. Seen on main, not
from this round: in note 49's scene the garrisoned civilian building has experience of its own
and draws a badge while its occupant is inside, before the lane's fight begins (also at 7a2ab68; round 8's lane
at e782d3b showed no experience), the case note 49 describes; passed to the lead.

**Try it (round 8b, rendering and UI).**
- Notes 45 and 46: send a giant squid at an enemy ship. Once it grabs the ship you see the squid's grab animation on the ship, in the squid owner's colour, drawn upright while the ship rolls at each push and changing with the ship's facing; it goes when the squid lets go, when the ship sinks and when the squid dies. The ship's roll, the wakes and the splashes are as before.
- Note 47: build a hovercraft and park it on land, on water and on a slope, then drive it about. Its shadow lies on the ground or the water under it, clearly visible, and stays put while the craft bobs.
- Note 54: rest the pointer on a unit or building without moving it. After one second a small black box with a coloured frame shows its name 16 pixels below the pointer, for ten seconds. Any movement, a press or release of any mouse button or turning *Name tips* off in the settings (from the main menu or the options menu) hides it. Over the black shroud it shows the shroud's text; an enemy spy in disguise shows the name of what it poses as; a cloaked enemy shows nothing. The names are still the internal type names, until Q-2686 lets us read the string table.

### Round 8c, rendering and UI (answers to Q-2686 and Q-2687)

The lead's answers replace round 8b's interims for note 54: S-0015 (Q-2686, the string table file)
and S-0111 r40 (Q-2687, the name tip's open points). Built in M8 step A3 intake 109 (ledger:
[intake109.md](intake109.md)); lane `crates/app/tests/r8_render_lane.rs`
(`i109_the_name_tip_shows_the_display_name`, `i109_the_install_string_table_reads`), frames in
`local-lane/m8/i109/render/before/` (main at c410e7f, no table read: the type's section name) and
`after/`. The departures these answers name come without a switch: judged in
[c7-play-fixes.md](c7-play-fixes.md), *A3 intake 109*.

| Request | Notes | Answer | Built | Status |
|---|---|---|---|---|
| [Q-2686](../../spec/requests/Q-2686-the-string-table-file-its-decoding-and-the-missing-label-marker.md) | 54 | S-0015: the string table is the install's `ra2md.csf` (never `ra2.csf`); a 24-byte header, entries of labelled texts, each stored unit the complement of its text unit, a text ending before a stored 0 or 65535; a lookup gives the entry's first text (empty for an entry with none); a label no entry matches shows the placeholder, the marker word, a colon and the label between apostrophes; a damaged file is used as far as its whole entries | `csf::StringFile::read` (the layout, the decoding, the warnings), `strings::{StringTable::from_bytes, read, placeholder, resolve}`; `install::prepare` reads the table once when the program starts (a missing or bad one stops start-up), the client hands it to each game, and every display name and fixed label resolves through it | Built. Our stand-in names now show only in a game built without an install (tests) |
| [Q-2687](../../spec/requests/Q-2687-the-name-tips-open-points.md) | 54 | S-0111 r40 claims 20, 24, 25, 26: the pointer leaving the window is no event; a held left or right button pressed over the view, an order block, a moving view and the options menu block the tip; the text is chosen again in every drawn frame while it shows (another text replaces it, none hides it); the box has margins 8 and 4, wraps at the last space within W − 8, flips with its last row on the pointer's row, and its text starts 4 right and 2 below its corner | `name_tip` (`NameTip::update`, `lines`, `size`, `place`, `INSET`), `Session::{view_button_held, update_name_tip, pointer_left, open_options, close_options}` | Built (a button held in the route mode: Q-2736 open, interim: it blocks) |

**Try it (round 8c, rendering and UI).**
- Q-2686: start a skirmish, rest the pointer on a unit or building for one second. The box now shows the unit's or building's real display name, not its internal type name. A type whose label the string table lacks shows the word MISSING, a colon and the label between apostrophes.
- Q-2687: rest the pointer on a unit: the box is a little roomier than before (the text starts 4 pixels in from the left and 2 down). Put the pointer within a box height of the bottom of the view: the box flips above the pointer, its last row on the pointer's row. Hold the left or right button down on the map and move the pointer a little: no box shows while the button is held, nor after you let go until you move and rest again. Open the options menu with a box showing: it goes and does not come back after the menu closes. Select a unit while the box shows: no change; destroy the unit under the box: the box goes (or shows what is then under the pointer). Take the pointer off the window and back: the wait runs from the last move inside the window.

## Round 9 (2026-11-06, implementer findings on main)

Findings of the intakes and of round 8b's lanes, not notes of the lead's. Lane outputs are in
`local-lane/m8/a4-r9/` (`before/` on main at bf6a19b, `final/` on this round's tree, `diag/` for
the builds of other commits).

### Round 9, simulation and AI

| # | Finding | Class | Specs | Request | Status |
|---|---|---|---|---|---|
| 1 | AI games that never end while one side is winning (intakes 99 and 104) | Our AI (D14). Two kinds were found, on builds whose AI is main's. (a) A winner sending a trickle: on 1f8a819 (`ai-sides 120000 10 2`, sides 0 and 2) house 1, with some 40 units, sent waves of five to seven at a civilian building garrisoned by eight of house 0's infantry, for 40 000 ticks; its drainer and other units stood with a *move* toward an object no longer in play (a building destroyed under them), which `scripted::idle` counted as busy, so no wave took them again. (b) No side winning: both houses with no credits and no army left to finish the other (intake 104 seed 8 map 0, intake 105 seed 7 map 0 sides 1 and 2); that is an economy matter, not a stalled winner. On main both of the brief's cases end: seed 10 map 2 sides 0 and 2 at tick 34 600, seed 8 map 0 sides 0 and 1 at 43 829 | none (our design); S-0309 claim 27 (a removal drops targets, not destinations) | none | Fixed (a) (`scripted::idle`): a unit whose *move* names an object no longer in play counts as idle, so the next wave or the guard takes it. On 1f8a819 with this alone the game of (a) ends at tick 92 640, house 1 the winner (before, not over at 120 000). A second rule tried with it (a guard of `GUARD_MIN` once the house has explored every enemy start and sees no enemy building that builds) changed that game's end by 716 ticks only and rarely held, since the starts the waves visit include the map's unused ones; it was dropped. (b) is left as a seam: a house with no income and buildings to spare could sell some to fund an army. Test `a_move_toward_a_removed_object_leaves_the_unit_idle` (`ai.rs`). The lane's `GLASNOST_AI_STALE=n` lists every *n* ticks the player houses' objects that hold a removed object as destination or target, and the rockets whose flight has not started |
| 2 | Rockets left at health 0 that never detonate (intake 104) | The specs are silent. An infantry unit on a sabotage order was a spawner's controller target; in tick *t* the rocket was placed and released at the unit's cell, and the unit went inside its target building in the same tick. S-0506 claim 14 fixes the aim at the rocket's first move from its destination's heading point, and says nothing of a destination removed before that move; claim 26's re-orders do not start a flight; claim 19 tests detonation only in flying phases, and claim 20 keeps a destroyed rocket above the terrain in play. So the rocket stood at its placement point, in *move*, speed 0, until the game's end. Not reproduced on main (that game takes another course). The rocket was not what kept intake 104's game alive: house 1 still had some 20 buildings | S-0506 claims 7, 12, 14, 19, 20, 24, 26; S-0309 claim 27; S-0414 claims 3, 4 | [Q-2711](../../spec/requests/Q-2711-a-rocket-whose-destination-is-removed-before-its-first-move.md) (whether and where it flies, whether it stays, whether it counts for its house's survival) | Answered by S-0506 r34 and built in intake 110 ([intake110.md](intake110.md)): a rocket whose destination is not in the air at the launch has its aim and flight fixed at the launch (`spawn::launch`, `rocket::fix_at_launch`), so it flies to that aim and detonates whatever its destination does; going inside also resets the controller (claim 11). A destination in the air at the launch keeps r20's rule (Q-2748 interim) |
| 3 | Aircraft 898 and 910 keeping orders toward removed object 801 (intake 103) | The specs give what was seen. Both are rocket missiles (the fixture's first aircraft type), not jets, so S-0506 governs them and S-0501's destination drop (claim 32) does not (S-0501 Terms leave rocket missiles out). A rocket in flight keeps the aim fixed at its first move (claim 14) and ignores later orders, whatever becomes of the object it was launched at, until it detonates by claim 19; a destination kept toward a removed object is so not a departure for a rocket in flight. Not reproduced on main: seed 5 ends at tick 22 605; of 24 rockets placed, 22 left play within 92 ticks, 2 were in cruise at the end | S-0506 claims 12, 14, 19, 26; S-0501 Terms, claim 32 | none (the one case the specs leave open is item 2's) | Recorded |
| 4 | A garrisoned civilian building gains experience and a badge (note 49's lane, `note_49_a_veteran_goes_into_a_building`) | The specs are silent on one step. In the entry tick, in the building's turn, the building (still the civilian house's, one occupant inside) takes one of the occupant's house's infantry as its target by its passive search (S-0309 claim 2 since round 8; nothing allies the civilian house with a player's). S-0508 claim 7's owner change then makes the occupant's house its owner later in the same turn, and the target is kept. 81 ticks later its shot kills three of its own house's infantry. These are allied kills of value 0, so by S-0311 claims 7 and 8, F3 and `quirks.zero_kill_value_caps` the building's own experience goes to the cap, and it draws the elite insignia (S-0111 claim 18). The experience is the building's, not its occupants' | S-0508 claims 1, 7; S-0309 claims 2, 25, 27 to 29, table C; S-0311 claims 7, 8, F3; S-0111 claim 18; S-0203 | [Q-2712](../../spec/requests/Q-2712-a-garrisons-target-across-the-occupation-owner-change.md) (where the owner change falls in the building's turn; whether a holder keeps its target across its own owner change) | Answered by S-0309 r120 claim 30 and S-0508 r13 claim 7, built in intake 110 ([intake110.md](intake110.md)): the owner change drops the building's target inside the change (`econ::events::owner_changed`), and the order our engine had (the change after the behaviour processing, the per-tick search and the firing) is now the spec's. The building no longer fires on its own house's infantry after the entry, so it gains no experience from them and shows no badge for it |

**Lanes (round 9, simulation and AI).** Before is main at bf6a19b; after is this round's tree,
rebased on cc5c004 (main's new commits are spec files only). The M6c, M7a and M7b sweeps (seeds
1 to 16): M7a and M7b match seed by seed (smallest counter 1 in M7a, at seed 9, and the drain's
smallest 3 in M7b, before and after); M6c keeps every seed's smallest counter (the smallest over
the seeds 2 before and after) and moves a few medians and extremes by one or two (engineer
orders min 5 before, 4 after; deploy orders median 157 before, 169 after). The arena (`--record
all --threads 2`) matches line by line apart from timings: 12 of 96, 2 of 24 and 1 of 35 scored
pairings failing, M-0019 0 of 6, M-0021 8 of 50, M-0020 1 of 25 checked; each record unchanged.
`glasnost-ai --include-ignored`: 255 tests pass before, 256 after (the new one). The AI quick lane (seeds 7 to 10, maps 0, 2 and 8; 36 games): every game ends with a
winner before and after; 10 games are the same and 26 take another course; median end 43 025
before, 43 080 after; the longest 76 412 before, 112 412 after (seed 7, map 2, sides 1 and 2: a
long, even game, not a stall; the houses built 230 and 316 army units and held up to 3 and 4
construction yards at once, and house 1 won where house 0 had won at 46 403 before); two games
change winner. The B-track exits: houses that held two construction yards at once 37 of 72 before and
after (the first such tick at a median of 28 597 before, 28 190 after); a main superweapon first
stood ready 103 times at a median of 22 230 before, 108 times at 23 813 after (the earliest
13 588 in both); dominator launches 26 before, 31 after. Four houses, free for all (`four_houses`,
map 30, seeds 1 to 3, 30 000 ticks): the same as before; no game ends by tick 30 000 (two-house
games end at a median of about 43 000), one yard lost (seed 1, house 2, tick 8 197), seeds 1 and 3
the same tick for tick and seed 2 with a few more objects gone. Of the goldens only
`duel-naval` moved (its AI houses free such units too); it is regenerated. The formats stay at
save 107 and AI game 71: no state is added.

**Try it (round 9, simulation and AI).**
- Item 1: play a long game against the computer and let it get the upper hand. Once your base is down to a few buildings, its units no longer stand about the map: those that were heading for something you have lost come back into its waves, and it finishes you off.
- Item 2: nothing to see in normal play; the case needs a rocket launched in the very tick its target enters a building. Since intake 110 such a rocket flies to where it was aimed and bursts there.
- Item 3: launch rockets at a unit that then dies or leaves: the rockets fly on to where they were aimed and burst there.
- Item 4: put one infantry unit into a civilian building with a few of your own infantry standing next to it. Before intake 110 the building could fire on your own units in the first moments and show an elite badge. Since intake 110 (S-0309 r120, S-0508 r13) it drops the target it took as the civilians' building when it becomes yours, so it does not fire on your units and gets no badge from them.

### Round 9b, AI end-games

Implementer findings on main at 07d549c. Lane outputs are in `local-lane/m8/a4-r9b/`: `before/` is
main, `c/` is this round's tree (variant C below), and `a/` and `b/` are the variants tried and
dropped.

| # | Finding | Class | Specs | Request | Status |
|---|---|---|---|---|---|
| 1 | `skirmish::plays_to_the_last_side` fails on main (not over at 40 000 ticks) | Our AI (D14). At tick 40 000 house 1 had a lab and a harvester left and nothing that builds. House 0 had three walkers resting at home, four jets, a helipad and no credits. With three ground units, the guard (`GUARD_MIN` 2) left one waiting, against a wave of six or more, or half of that for patience, so no wave went again. Fixing the guard alone was not enough: once the lab fell, the walkers rested at the cleared enemy base, and the last harvester stood on cells house 0 had not explored. A house that sees no enemy does not finish, so the game went on | S-0414 claim 5 (a defeat is announced, so the house knows how many enemies are left); S-0409 claim 19 (no fog in a skirmish: the explored cells are what the house sees); S-0214 Terms (armed) | none | Fixed (`Script::finishing`). A house *clearly winning* has looked at the enemy base: it has explored the enemy start nearest each enemy building it sees, or it sees no enemy building and every enemy start is cleared. What it sees fighting there (army units and armed buildings, by cost) is worth at most a third of its own army (`FINISH_SHARE` 3, design). Against an enemy it sees nothing build from, waves are of one unit, so whatever waits goes. A cleared base sends the army to the unexplored cell nearest it, through the existing fallback. With one enemy house left, the guard goes too. Against an enemy that still builds (a factory, a yard, or a vehicle that deploys into one), the guard stays and waves keep the plan's first size instead of growing. A clearly winning house's waves do not count towards the growth. The synthetic skirmish now ends at tick 37 944 with house 0 the winner (before: not over at 40 000) |
| 2 | Starved games that no side wins | Our AI (D14). Intake 104's seed 8 map 0 and intake 105's seed 7 map 0 sides 1 and 2 both end on main (39 180 and 49 264). The starved game on main is seed 10 map 8 sides 1 and 2, not over at 120 000. At the end, house 0 had 12 army units, no credits and five harvesters. House 1 had one aircraft and four infantry, 20 buildings with factories, 88 credits and no harvesters. Neither house was selling, because house 0 still had income. House 0's waves went for patience every 3 000 ticks, grown to a size its army no longer reached, and died against a base that still builds | S-0408 claims 1, 3 (selling: the refund, and only sellable buildings) | none | Fixed by item 1's rule for an enemy that still builds: that game ends at tick 85 588, house 1 the winner. Two other variants were tried. In variant A (no wave change against an enemy that builds), the game is the same as on main. In variant B (waves of one with the guard kept), it ends at 30 878, but the synthetic skirmish then ends only at 53 518, because house 0 fed its army away one unit at a time, so B was dropped. Selling (`sell_when_broke`) also changes: one factory per kind is now kept while the house has anything else to sell, army or none. On main a house with no army sold its costliest building first, often its war factory, and lost the means to build the army its refunds paid for. It sells out only when nothing else is left |
| 3 | Aircraft in the end-game | The specs give what was seen. In the synthetic skirmish's end-game, a jet with ammunition attacks the nearest seen enemy it can hurt, at any distance. Jet 636 had 3 attack orders from tick 36 150 to the end at 37 944; on main it had 7 from 36 150 to 39 555, at the lab and then the harvester. The other three jets stand at the single helipad's base with no ammunition. Only one can dock (S-0501 claims 22 to 24), so they never reload. The fixture has no pad-aircraft list, so S-0501 claim 30's limit never stopped the house from building more jets than it has docks. Real data has the list | S-0501 claims 22 to 24, 30 | none | Recorded; nothing changed. A seam for our AI: a person builds no pad aircraft beyond their docks |

**Lanes (round 9b).** Before is main at 07d549c; after (`c/`) is this round's tree on 07d549c. It
was rebased onto 0051f51, whose new commits touch no simulation or AI code, and the tests were run
again there.

- `glasnost-ai --include-ignored`: 255 passed and 1 failed before (`plays_to_the_last_side`), 263
  passed after (the seven new tests).
- AI quick lane (seeds 7 to 10, maps 0, 2 and 8; 36 games): 35 games end before and 36 after. House
  0 wins 21 before and 23 after; house 1 wins 14 before and 13 after. The median end is 46 839
  before and 45 707 after. The longest finished game is 90 369 before and 85 588 after; the one
  unfinished game before is item 2's.
- B-track exits:
  - houses that held two construction yards at once: 39 of 72 before, 34 after;
  - houses with a main superweapon ready: 28 before, 26 after;
  - dominator launches: 45 in 14 games before, 40 in 16 games after.
- Sweeps 1 to 16. The smallest minimum is kept: M6c 0 and 0, M7a 1 and 1, M7b drain 3 and 3.
  - M6c seeds 3 and 4 end a little later (8 356 to 8 841, and 8 640 to 9 060). The jumpjet-orders
    minimum goes from 3 to 1, and crates picked up from 8 to 3.
  - M7a and M7b counters move by a few.
- Four houses, free for all (`four_houses`, map 30, seeds 1 to 3, `GLASNOST_FH_DELAY=in`, 60 000
  ticks):
  - seed 2 is the same tick for tick;
  - seed 3 ends at 52 706 with house 2 the winner (before: not over at 60 000);
  - seed 1 takes another course from about tick 8 000. House 2's yard falls at 8 784 instead of
    51 169, and house 0 is out at 19 798.
- Goldens. These moved and are regenerated on the rebased tree: `skirmish`, `duel`,
  `duel-diagonal`, `duel-naval` and `duel-m7b` (their AI houses now take other courses).
- Formats. The AI game format goes from 72 to 73, since a saved AI game now plays on to another
  course. The save format stays at 108 and the wire format at 12: no state is added.

**Try it (round 9b, AI end-games).**
- Item 1: play against the computer and lose most of your base, but keep one building that does not build, such as a power plant or a lab, plus a unit hidden in a far corner. The computer no longer leaves a few tanks at home: it sends everything, finishes the building, then spreads out to look for the hidden unit.
- Item 2: play a long game where both sides run short of money and you keep a factory. Once the computer is well ahead, it keeps sending waves of the same size, guard at home, instead of waiting for bigger and bigger ones.
- Item 3: nothing changes. A computer with more jets than helipad places still has idle jets on the ground.
