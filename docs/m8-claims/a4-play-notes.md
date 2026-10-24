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
| 5 | Harriers that lose their target before firing circle at random | Not confirmed. By the specs the aircraft flies home when a pad grants contact, and it circles the old firing spot only with no dock types (T2's guard row). Also a small bug: the held flight point's cruise height is recomputed on every read instead of once when the point is accepted | S-0501 claims 5, 6, 15, 17, 18, 32, 40; T1, T2; AC19 | Q-1755, Q-1756 | Partly fixed (A4-1). The held point's height is now fixed at acceptance (claim 6), so a landable aircraft no longer dips at the old spot before going home; a target removed without a kill is now dropped (claim 32) instead of being attacked forever. With one dock type the aircraft flies home and docks. The local lane shows 10 of 12 aircraft types have no dock types, 2 of them armed and landable: by T2 such an aircraft moves to its last firing spot and holds at cruise height, now after every attack (before it landed by the recomputed height). Q-1756 asks whether it should land; Q-1755 asks about a target taken off the map. Q-1755, Q-1756, Q-1775 and Q-1776 answered by S-0501 revision 40 and built at M8 step A3 intake 9: an aircraft loses its target at the moment the target is killed, taken off the map or removed, with claim 37's timer draw (claim 32); an armed aircraft with no dock types heads for its house's edge after every run, a human player's lands at a usable cell near that edge cell and rests in guard, a computer player's keeps the edge cell in area guard (claim 45); a pad-bound aircraft with no pad to go to crashes at the run end, credited to nobody (claim 46). Q-1837 (area guard above the flight height) and Q-1838 (a visiting type placed on the ground retreats at once) are open |
| 6 | Destroyed units leave spinning particles | The specs are silent. A sprite debris piece (a bouncing animation) plays in place with no motion and no landing, so a looping one never ends. This is the seam "bouncing motion and landing" | S-0109 Terms (bouncing type), claims 7, 22; S-0312 claim 4, F1; S-0202 claim 10 | Q-1762 | Waits for Q-1762. Animations are simulation state, so no interim end rule is invented. S-0317 (built at M8 step C5 part 2) does not explain this note: it covers the voxel-model debris only, and those pieces now fly, bounce and end (S-0317 claims 5 to 11); the sprite debris path is unchanged |
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
| 10 | "Random debris after a tank got destroyed, staying in place" (a screenshot shows a dark piece left on the ground beside the wreck spot) | Recorded only, no code change. Matches a sprite debris piece with no motion and no landing (Q-1762), or a voxel debris piece (Q-1702); which one cannot be told from the screenshot | S-0109 Terms (bouncing type), claims 7, 22; S-0312 claim 4, F1; S-0317 | Q-1762 (open); Q-1702 answered by S-0317 | Half settled at M8 step C5 part 2 (S-0317). It was not a voxel piece: until then a voxel debris type was handed to the animation path, where a name that is not also an animation type made no animation (at most the debug view's short-lived stand-in marker), so no voxel piece was ever drawn; since S-0317 every voxel piece flies, bounces and ends (its lifetime counter runs down each turn and a stop sets it to 0, claims 6 and 9). A piece left in place is a sprite debris piece: waits for Q-1762 |
| 11 | "Captured oil derricks do not animate, nor they get the player color" | Code bug, in the renderer, for the colour and for the animation after the capture; the specs are ambiguous for the animation before the first ownership change. The simulation's capture follows the specs: the engineer's capture changes the owner in place (same object). The map's buildings are drawn in the cell pass built at load, with their owner and health at load and their slot animations at their first image frame; the live drawing played their slots but never drew them, and nothing redrew them on an owner or health change. The colour is read from the owner at draw time for every building drawn live (and on the radar), so only the map's buildings were stuck | S-0509 claims 1, 7, 8; S-0214 table M; S-0103 claim 8; S-0108 claims 2, 11, 13; S-0109 claims 1, 5; S-0405 claims 8, 9; S-0515 claim 2 | Q-1772 (slots of a tech building before and at its first ownership change: S-0108 claim 11 starts them at set-up, S-0405 claim 9 stops the working animations of a building not in operation, and S-0108 claim 13, the only rendering rule for that, covers needs-power buildings only) | Fixed (`Live::redraw`): a map building whose owner or health differs from the start, or that has a running slot animation and is not an unmanned tech building, leaves the cell pass and is drawn live, in its owner's house colour, with its damage frame and its playing slots. A captured tech building is manned, so it now shows its new owner's colour and animates. Left alone for Q-1772: a tech building that has never changed owner keeps its load-time drawing (slots at their first frame), as before. Also fixed on the way: the map's buildings did not show damage in play (S-0108 claim 2), and the map's other animated buildings stood at their slots' first frame. Tests `mapview::capture_tests::*` (synthetic tech building captured by an engineer through a click; damaged; a building that needs no engineer animating from the start) |
| 12 | A screenshot with three small pieces left on the ground, two dark (one with a blue streak) and one tan, no wreck in view | Recorded only, no code change. As note 10: sprite debris pieces with no motion and no landing (Q-1762), or voxel debris pieces (Q-1702); the screenshot cannot tell which | S-0109 Terms (bouncing type), claims 7, 22; S-0312 claim 4, F1; S-0317 | Q-1762 (open); Q-1702 answered by S-0317 | As note 10 since M8 step C5 part 2: not voxel pieces (none was drawn before S-0317, and since it each one ends), so sprite debris pieces; waits for Q-1762 |

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
| 13 | "still particle issues", with a screenshot: a small blue-and-white piece on paving beside an infantry unit, a dark piece on the grass above it, two small tan pieces on a road beside vehicles | Recorded only, no code change. The lane scene `debris-left` kills enemy infantry by each death kind and enemy vehicles, and samples what stays on the map every 50 ticks up to 1000 ticks later: only bouncing animations (sprite debris pieces) stay, 10 of them at the end; every corpse animation ends inside the window, every voxel debris piece ends earlier, no other animation is endless and no particle system with live particles stays (the census counts empty systems too; they draw nothing). The tan pieces and the dark piece match sprite debris pieces (Q-1762). The blue-and-white piece beside an infantry unit matches a corpse animation, which ends (S-0107 claim 17); it is a leftover only if it was still there long after the death, and then it too is a sprite debris piece. Same before and after the note 15 fix | S-0107 claim 17; S-0109 Terms (bouncing type), claims 7, 22; S-0312 claim 4, F1; S-0317; S-0616 | Q-1762 (open) | Waits for Q-1762, as notes 6, 10 and 12. The lane's screenshots did not frame the kill spot, so there is no side-by-side crop |
| 14 | Prism tanks have no turret | Same case as note 3. The type the note names is one of the 2 multi-turret vehicle types: it imports four numbered turret parts and no plain turret part, so the renderer draws no turret until Q-1760 gives the selected index. The local lane's class counts over every voxel vehicle type: 2 multi-turret (none drawn until Q-1760), 15 with the turret flag and a single turret part (drawn), 1 with the turret flag and no turret part, 57 with neither | S-0105 claims 4, 8, 12, 16, 26; S-0007 claims 16 to 18 | Q-1760 | Waits for Q-1760 |
| 15 | Anti-air defences and units do not engage enemy aircraft on their own, or too late | Code bug. Aircraft in flight were never registered as tracked flying objects. An aircraft in flight is listed in no cell (S-0209 claim 13), and a range search that looks for aircraft finds them only through the tracked flying objects it scans first (S-0309 claim 9; an aircraft-only search scans no cell ring), so no automatic search ever found an aircraft in flight: anti-air-only defences fired only when ordered or after being hit. S-0303 Terms make an aircraft-kind object in ordinary flight a tracked flying object; S-0501 gives no events for when that starts and ends | S-0303 Terms, claim 11; S-0309 claims 1, 4, 5, 9, 20; S-0209 claim 13; S-0501 Terms; S-0215 claim 33; S-0308 claim 6 | Q-1845 (when an aircraft is a tracked flying object) | Fixed. `air::sync_listing` keeps the registration by Q-1845's interim rule: tracked exactly while on the map and in flight (S-0501's in-flight test), checked after every movement update, on placement, on dock release and on each cell crossing, its sector updated at each; untracked on the ground, inside a transport or building (`objects::take_inside`), when lifted off the map (`superweapons::warp`) and on removal. Tests in `claims_s0309_airborne.rs` (synthetic types, real order and tick path, human and computer owner): an anti-air-only building, an anti-air and anti-ground unit and an anti-air-only unit each acquire and fire at an enemy aircraft flying past on a move order; an aircraft is tracked exactly while in flight. The tracking has a second effect: a detonation above the terrain now reaches an aircraft in flight, at half distance (S-0303 claims 6 and 10; `area_damage_reaches_an_aircraft_in_flight_at_half_distance`), and under `quirks.flying_candidates_unfiltered` (S-0303 claim 9) an aircraft's own burst at its own height can hit it. Lane timings below |

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

Note 7 (seen again): the lead's statement and Q-1850 are in note 7's Round 1 row; nothing more is
filed and the code keeps to S-0417 claims 10 and 14 until Q-1850 is answered.
