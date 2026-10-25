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
| 3 | IFVs have no turret | Code bug, in the renderer: the importer keeps a multi-turret type's numbered turret parts, but the renderer reads only the single turret part | S-0105 claims 12, 16, 26; S-0007 claims 16 to 18 | Q-1760 (which turret model is shown) | Plumbing done (A4-1): the renderer draws `turret{k}` and `barrel{k}` of the selected index and keys the image by it. The index function gives no index until Q-1760 is answered, so IFVs still show no turret. The local lane confirms the premise: both multi-turret vehicle types import `turret0`, none a plain `turret`. Seen again 2026-10-24 (Round 3): the plumbing is in place on main of that day, nothing else is missing; still waits for Q-1760 |
| 4 | Patriot missiles can target ground units | Code bug for a person's attack order: table B row B7's anti-ground and being-sold conditions are missing. Retaliation against a ground attacker follows the specs | S-0214 table B row B7; S-0309 claims 5, 18; S-0306 claim 23 and Import mapping | Q-1761 (retaliation, and B7 over aircraft); Q-1763 (the lead's follow-up of 2026-10-22: in the original an anti-air-only building never fires back at ground attackers, and no object gets a fire-back exception for a target it cannot otherwise attack) | Fixed (A4-1): row B7 gives nothing for a first-slot projectile that is not anti-ground, a pulse cannon or a building being sold, over ground units and aircraft alike (Q-1761 open for aircraft). Importer check: the 2 buildings whose primary projectile is anti-air and not anti-ground import anti-ground = no; no importer bug. Retaliation: fixed (A3 intake 5) by S-0306 revision 33 and S-0309 revision 38 (Q-1761 questions 1 and 2, Q-1763 answered): 2 of 2 such building types have land-targeting mode 1, so the firing check's land row refuses every ground object on land to them and they no longer fire back (S-0309 claim 18, first case); lane scene `aa-only` (`m7b_lane.rs`): before, each held the infantry attacker as its target until it died, for a human and a computer owner; after, neither holds or fires at a ground attacker. Q-1761 question 3 (row B7 over aircraft) stays open |
| 5 | Harriers that lose their target before firing circle at random | Not confirmed. By the specs the aircraft flies home when a pad grants contact, and it circles the old firing spot only with no dock types (T2's guard row). Also a small bug: the held flight point's cruise height is recomputed on every read instead of once when the point is accepted | S-0501 claims 5, 6, 15, 17, 18, 32, 40; T1, T2; AC19 | Q-1755, Q-1756 | Partly fixed (A4-1). The held point's height is now fixed at acceptance (claim 6), so a landable aircraft no longer dips at the old spot before going home; a target removed without a kill is now dropped (claim 32) instead of being attacked forever. With one dock type the aircraft flies home and docks. The local lane shows 10 of 12 aircraft types have no dock types, 2 of them armed and landable: by T2 such an aircraft moves to its last firing spot and holds at cruise height, now after every attack (before it landed by the recomputed height). Q-1756 asks whether it should land; Q-1755 asks about a target taken off the map. Q-1755, Q-1756, Q-1775 and Q-1776 answered by S-0501 revision 40 and built at M8 step A3 intake 9: an aircraft loses its target at the moment the target is killed, taken off the map or removed, with claim 37's timer draw (claim 32); an armed aircraft with no dock types heads for its house's edge after every run, a human player's lands at a usable cell near that edge cell and rests in guard, a computer player's keeps the edge cell in area guard (claim 45); a pad-bound aircraft with no pad to go to crashes at the run end, credited to nobody (claim 46). Q-1837 (area guard above the flight height) and Q-1838 (a visiting type placed on the ground retreats at once) were answered by S-0501 revision 41 and built at intake 13: area guard falls back only at exactly the cruise height, and a type that is not landable, placed on the ground, rests in guard and takes orders |
| 6 | Destroyed units leave spinning particles | The specs are silent. A sprite debris piece (a bouncing animation) plays in place with no motion and no landing, so a looping one never ends. This is the seam "bouncing motion and landing" | S-0109 Terms (bouncing type), claims 7, 22; S-0312 claim 4, F1; S-0202 claim 10 | Q-1762 (answered in part by S-0319 r9); Q-1846, Q-1847 | Fixed at M8 step A3 intake 15 part 1 (S-0319 r9, `bounce`; ledger [intake15a-S-0319.md](intake15a-S-0319.md)). S-0317 (built at M8 step C5 part 2) covered the voxel-model debris only. Since S-0319 a sprite debris piece flies from its creation, lands and ends there with its landing effects (claims 2 to 8), so a looping one no longer stays. Lane `debris-left` (below, note 13): bouncing animations left at +1000 ticks went from 10 to 0 |
| 7 | Barracks units with no rally point stay at the door and block later units | Follows the specs: the unit rests in guard and keeps the factory's one contact place, so later releases are busy and refunded | S-0417 claims 10, 14 and Open question 3; S-0404 claims 7, 13 | Q-1757; Q-1850 | Waits for Q-1757. Seen again 2026-10-24 (Round 3): the lead says the blocking "is 100% wrong": in the original, later infantry units still come out. Recorded as Q-1850 (the lead's report against S-0417 claims 10 and 14, a follow-up to Q-1757). The code keeps to the claims until Q-1850 is answered |
| 8 | A chrono miner should teleport only when it returns to a refinery, but it teleports to any click | Disputes an approved claim: every destination teleports while teleporting locomotion is active, a person's move order included. The claim has no measurement record | S-0207 claims 14, 20 (a); S-0214 claim 11; S-0402 Terms, claims 9, 17 | Q-1759 | Waits for Q-1759 |

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
| 13 | "still particle issues", with a screenshot: a small blue-and-white piece on paving beside an infantry unit, a dark piece on the grass above it, two small tan pieces on a road beside vehicles | Recorded only, no code change. The lane scene `debris-left` kills enemy infantry by each death kind and enemy vehicles, and samples what stays on the map every 50 ticks up to 1000 ticks later: only bouncing animations (sprite debris pieces) stay, 10 of them at the end; every corpse animation ends inside the window, every voxel debris piece ends earlier, no other animation is endless and no particle system with live particles stays (the census counts empty systems too; they draw nothing). The tan pieces and the dark piece match sprite debris pieces (Q-1762). The blue-and-white piece beside an infantry unit matches a corpse animation, which ends (S-0107 claim 17); it is a leftover only if it was still there long after the death, and then it too is a sprite debris piece. Same before and after the note 15 fix | S-0107 claim 17; S-0109 Terms (bouncing type), claims 7, 22; S-0312 claim 4, F1; S-0317; S-0616 | Q-1762 (answered in part by S-0319 r9); Q-1846, Q-1847 | Fixed at M8 step A3 intake 15 part 1 (S-0319 r9, `bounce`; ledger [intake15a-S-0319.md](intake15a-S-0319.md)), with notes 6, 10 and 12. The same scene on main 707b61f and after the step (`local-lane/m8/a3-15a/debris-left-before` and `-after`; 14 kills, 6 of them vehicles): before, 10 bouncing animations at +50 ticks and still 10 at +1000; after, 3 at +50 and 0 from +100 on. Corpses, voxel pieces and other animations end as before. The lane's screenshots still do not frame the kill spot, so there is no side-by-side crop |
| 14 | Prism tanks have no turret | Same case as note 3. The type the note names is one of the 2 multi-turret vehicle types: it imports four numbered turret parts and no plain turret part, so the renderer draws no turret until Q-1760 gives the selected index. The local lane's class counts over every voxel vehicle type: 2 multi-turret (none drawn until Q-1760), 15 with the turret flag and a single turret part (drawn), 1 with the turret flag and no turret part, 57 with neither | S-0105 claims 4, 8, 12, 16, 26; S-0007 claims 16 to 18 | Q-1760 | Waits for Q-1760 |
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
filed and the code keeps to S-0417 claims 10 and 14 until Q-1850 is answered.

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
| 18 | An oil derrick animated with no house colour | Recorded, no code change: the colour follows the specs and the animation is Q-1772's. Lane, map 0: 5 map buildings need an engineer and have running slot animations; none has the terrain-palette art flag (S-0103 claim 8's exception). Fresh: neutral, unmanned, drawn in the cell pass (standing still), light grey. Damaged: drawn live, its 2 slots play, still light grey. Captured by the local house or by the enemy (277 ticks): manned, drawn live, slots play, the capturer's colour. So what the lead saw matches a derrick never captured but drawn live, most likely because it had been hit (or it was protected, flashing or being erased), animating in the neutral house's light grey (S-0103 claim 10). A captured derrick drawn without its capturer's colour was not seen in any case. Q-1772 question 1 asks whether an unmanned tech building's slots play at all; until it is answered the interim draws one live (and so animates it) once its owner or health changes, which is why a damaged one animates and a fresh one does not. If it is seen again, look at its health bar (damaged?), its owner in the hover name, its radar dot's colour, and whether income arrives | S-0103 claims 8, 10; S-0108 claims 2, 11; S-0405 claims 8, 9; S-0509 claim 8; S-0515 claim 2 | Q-1772 (open; covers the animation half). No new request | Recorded. Test `a_damaged_unmanned_tech_building_animates_in_the_neutral_colour` (`capture_tests.rs`) pins today's interim: drawn live after the damage, its slot plays, in the neutral house's palette and not a player's |
| 19 | "Some problems with debris still" (screenshot: small pieces near infantry and on open ground) | See intake 15 part 1 (S-0319, bouncing animations), which was not on the lead's build. S-0319 is not on main at this round's rebase (0d08efc). From the picture: small flat pieces lying still, one light one beside the infantry, one on the open ground to the north-east and two darker ones near the track, the same look as notes 12 and 13, which round 3's lane matched to sprite debris (bouncing animations); voxel debris ends since C5 part 2 (S-0317). So they are of the kind S-0319 ends | S-0109 Terms (bouncing type); S-0312 claim 4; S-0317; S-0319 | Q-1762 | Waits for intake 15 part 1; to be confirmed on a build with it |

**By eye (note 16).** Build a carrier, let it send its planes at a target, and sink it while they
are out: each plane falls and disappears when it reaches the ground or the water, within a second
or two; none stays lying on the water or the ground.

**By eye (note 17).** Select any armed unit and hover an enemy mirage standing still (it looks like
a tree): the attack cursor shows, and a plain left click attacks it. With nothing selected, a click
on it selects nothing. Your own mirage still selects with a click.

**Checks.** The simulation changed (notes 16 and 17; the computer player clicks through
`targeting::click` and `cursor` too). Before on main 0b935c1; after, the arena on this branch on
0b935c1 and the sweeps and goldens on it rebased on main 0d08efc (main's changes in between touch
no simulation code). Oracle arena: the CSVs are byte-identical before and after (M-0001 31 of 96 pairings failing, 152 of 172 unanimous outcomes
reproduced; M-0002 13 of 24, 43 of 47; M-0003 2 of 35, 61 of 64). The M6c, M7a and M7b seed
sweeps 1 to 16 print the same lines before and after (smallest minimum M6c 4, M7b drain 3). The
sim-gate goldens hold unchanged (`cargo test --release -p glasnost-sim-gate -- --include-ignored`
without the update flag), so in the gate games the changed paths do not change play. No state
was added (the killed mark already exists), so the save format stays 76 and the AI game format 38.
