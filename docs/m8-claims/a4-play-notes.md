# M8 step A4: the lead's play notes

Step A4 of [m8-plan.md](../m8-plan.md) starts from the lead's play notes. Each note becomes a code
fix when the code departs from an approved spec, a spec request when the specs are silent or the
note disputes them, or an importer check when the data may be at fault. A note is never a source of
behaviour by itself: a rule it states reaches the code only through a spec.

Requests for A4 are numbered Q-1750 to Q-1769.

## Round 1 (2026-10-21, a build of main from that day)

Triage was read-only, against the specs and the code at main 80f8a85.

| # | Note | Class | Specs | Request | Status |
|---|---|---|---|---|---|
| 1 | "Weird pathing issues" | The specs are silent in places. No departure from the specs found. | S-0213 claim 7 (interim group spread); S-0212 Terms (threat, coarse threat), claims 12a, 13, 17 to 23 | Q-1750 (group destinations), Q-1751 (cell threat); Q-1752 held until the note is narrowed down | Waits for details from the lead: units, terrain, symptom, and best a save with the tick |
| 2 | A group deploy should deploy all members while any is undeployed, never mix | Code bug | S-0213 *The deploy rule*; S-0214 Terms | Q-1753 (members that are still deploying or undeploying) | Fixed (A4-1, `targeting::key_command`): a garrisoned building first in the selection gets the only order; a selection mixing deployed and undeployed in-place deployers skips the deployed ones; every other member needs the deploy check and deploy readiness. Interims: Q-1753 (members in transition count as neither), Q-1754 (the timer after an infantry deploy is not modelled) |
| 3 | IFVs have no turret | Code bug, in the renderer: the importer keeps a multi-turret type's numbered turret parts, but the renderer reads only the single turret part | S-0105 claims 12, 16, 26; S-0007 claims 16 to 18 | Q-1760 (which turret model is shown) | Plumbing done (A4-1): the renderer draws `turret{k}` and `barrel{k}` of the selected index and keys the image by it. The index function gives no index until Q-1760 is answered, so IFVs still show no turret. The local lane confirms the premise: both multi-turret vehicle types import `turret0`, none a plain `turret` |
| 4 | Patriot missiles can target ground units | Code bug for a person's attack order: table B row B7's anti-ground and being-sold conditions are missing. Retaliation against a ground attacker follows the specs | S-0214 table B row B7; S-0309 claims 5, 18; S-0306 claim 23 and Import mapping | Q-1761 (retaliation, and B7 over aircraft); Q-1763 (the lead's follow-up of 2026-10-22: in the original an anti-air-only building never fires back at ground attackers, and no object gets a fire-back exception for a target it cannot otherwise attack) | Fixed (A4-1): row B7 gives nothing for a first-slot projectile that is not anti-ground, a pulse cannon or a building being sold, over ground units and aircraft alike (Q-1761 open for aircraft). Importer check: the 2 buildings whose primary projectile is anti-air and not anti-ground import anti-ground = no; no importer bug. Retaliation unchanged; it waits for Q-1761 and Q-1763 |
| 5 | Harriers that lose their target before firing circle at random | Not confirmed. By the specs the aircraft flies home when a pad grants contact, and it circles the old firing spot only with no dock types (T2's guard row). Also a small bug: the held flight point's cruise height is recomputed on every read instead of once when the point is accepted | S-0501 claims 5, 6, 15, 17, 18, 32, 40; T1, T2; AC19 | Q-1755, Q-1756 | Partly fixed (A4-1). The held point's height is now fixed at acceptance (claim 6), so a landable aircraft no longer dips at the old spot before going home; a target removed without a kill is now dropped (claim 32) instead of being attacked forever. With one dock type the aircraft flies home and docks. The local lane shows 10 of 12 aircraft types have no dock types, 2 of them armed and landable: by T2 such an aircraft moves to its last firing spot and holds at cruise height, now after every attack (before it landed by the recomputed height). Q-1756 asks whether it should land; Q-1755 asks about a target taken off the map |
| 6 | Destroyed units leave spinning particles | The specs are silent. A sprite debris piece (a bouncing animation) plays in place with no motion and no landing, so a looping one never ends. This is the seam "bouncing motion and landing" | S-0109 Terms (bouncing type), claims 7, 22; S-0312 claim 4, F1; S-0202 claim 10 | Q-1762 | Waits for Q-1762. Animations are simulation state, so no interim end rule is invented |
| 7 | Barracks units with no rally point stay at the door and block later units | Follows the specs: the unit rests in guard and keeps the factory's one contact place, so later releases are busy and refunded | S-0417 claims 10, 14 and Open question 3; S-0404 claims 7, 13 | Q-1757 | Waits for Q-1757 |
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
  Q-1775 (no spec gives a crash; no change until it is answered).
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
  in flight, such as jumpjet-locomotor units (S-0309 claim 9). Which one the lead saw is open: the
  lead is asked which enemy units were fired on and whether they were flying.
- Note 5 (computer aircraft circling). No code or importer bug. Only the two pad-bound aircraft
  types have dock types; the computer player had a pad for every one it built, and with no pad a
  pad-bound aircraft in flight crashes, by S-0501 claim 18 and T2 (seen in the lane). The armed,
  landable types with no dock types end their run in *guard* (T2), queue *move* to their
  destination (claim 33), which after a strafing run is the target object (T1), keep the holding
  mark (claim 6 with Q-1756's workaround) and stay above the target at cruise height. That is the
  lead's "circled over their target". Q-1776 corrects Q-1756's premise (destination: the target,
  not the last firing spot) and asks about armed non-landable types with no dock types. Q-1775's
  question 1 is answered by claim 18 for pad-bound types; the data has no type with dock types that
  is not pad-bound.
