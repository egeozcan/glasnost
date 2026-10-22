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
| 2 | A group deploy should deploy all members while any is undeployed, never mix | Code bug | S-0213 *The deploy rule*; S-0214 Terms | Q-1753 (members that are still deploying or undeploying) | Fix briefed |
| 3 | IFVs have no turret | Code bug, in the renderer: the importer keeps a multi-turret type's numbered turret parts, but the renderer reads only the single turret part | S-0105 claims 12, 16, 26; S-0007 claims 16 to 18 | Q-1760 (which turret model is shown) | The plumbing is briefed. The model choice waits for Q-1760 |
| 4 | Patriot missiles can target ground units | Code bug for a person's attack order: table B row B7's anti-ground and being-sold conditions are missing. Retaliation against a ground attacker follows the specs | S-0214 table B row B7; S-0309 claims 5, 18; S-0306 claim 23 and Import mapping | Q-1761 (retaliation, and B7 over aircraft); Q-1763 (the lead's follow-up of 2026-10-22: in the original an anti-air-only building never fires back at ground attackers, and no object gets a fire-back exception for a target it cannot otherwise attack) | Fix of B7 briefed, plus a check of the imported anti-ground flag. Retaliation waits for Q-1761 and Q-1763 |
| 5 | Harriers that lose their target before firing circle at random | Not confirmed. By the specs the aircraft flies home when a pad grants contact, and it circles the old firing spot only with no dock types (T2's guard row). Also a small bug: the held flight point's cruise height is recomputed on every read instead of once when the point is accepted | S-0501 claims 5, 6, 15, 17, 18, 32, 40; T1, T2; AC19 | None yet | A diagnostic test, the cruise-height fix and a check of the imported dock list are briefed |
| 6 | Destroyed units leave spinning particles | The specs are silent. A sprite debris piece (a bouncing animation) plays in place with no motion and no landing, so a looping one never ends. This is the seam "bouncing motion and landing" | S-0109 Terms (bouncing type), claims 7, 22; S-0312 claim 4, F1; S-0202 claim 10 | Q-1762 | Waits for Q-1762. Animations are simulation state, so no interim end rule is invented |
| 7 | Barracks units with no rally point stay at the door and block later units | Follows the specs: the unit rests in guard and keeps the factory's one contact place, so later releases are busy and refunded | S-0417 claims 10, 14 and Open question 3; S-0404 claims 7, 13 | Q-1757 | Waits for Q-1757 |
| 8 | A chrono miner should teleport only when it returns to a refinery, but it teleports to any click | Disputes an approved claim: every destination teleports while teleporting locomotion is active, a person's move order included. The claim has no measurement record | S-0207 claims 14, 20 (a); S-0214 claim 11; S-0402 Terms, claims 9, 17 | Q-1759 | Waits for Q-1759 |
