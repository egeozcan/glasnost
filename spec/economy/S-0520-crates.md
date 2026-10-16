---
id: S-0520
title: Crates (placement, replacement, pickup and power-up effects)
topic: economy
revision: 4
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-16
measurements: []
requests: [Q-0987, Q-1033]
supersedes: []
---

# S-0520: Crates (placement, replacement, pickup and power-up effects)

## Summary

How skirmish crates are placed at random, expire and are replaced, who picks them up and when, how a crate's power-up is chosen and which power-ups turn into money, and what each of the nineteen power-ups does. Crates created by deaths (S-0312 claims 14 and 20) and by the trigger action (S-0524 claim 4) are placed by those specs and collected by this one. The game-start crate count is S-0412 claim 10's. Campaigns and network games are out of scope.

## Terms

- **Crate.** A cell holding a crate overlay: an overlay type with the crate flag (Import mapping). The cell's overlay data value is the crate's **contents**: 0 … 18 name a fixed power-up, any other value means random contents (claim 8). New random placements store 255, not 19 or 20. A trigger can instead explicitly store another random-contents value under S-0524 claim 4; its number-slot value exactly 20 leaves the newly placed value 255 unchanged.
- **Crate places.** The scenario's capacity for 256 independently expiring crates (S-0524 claim 4), with priority numbers 0 … 255. Placement uses the lowest free number. Priority is observable through the replacement draws when several crates expire together; it does not require any particular storage representation.
- **Wooden-crate overlay, water-crate overlay.** The two overlay types the rules name for land and water crates (S-0524 claim 4, Import mapping there).
- **Power-up.** One of the nineteen effects a crate can give, numbered 0 … 18 in the order of the table in claim 8. Each has an imported weight, animation, water flag and argument *A* (a double). These four values make up the **power-up table**.
- **Resource stage-picture count.** The integer count limiting a resource's stages in claim 24. For every image number that establishes a run under S-0112 claim 10, it is 12 pictures, numerically equal to that spec's flat-run length *F*. It is not *F* + *S*, nor the sprite frame count *N*. See claim 24 for backing and tests.
- **Collector.** The object that picks up the crate (claim 6). Its house is the **collecting house**.
- **R(*a*, *b*).** An inclusive ranged request from the scenario random stream under S-0202 claim 4. One request can consume multiple raw draws through rejection, or none for equal bounds. Raw draws below are from that same stream.
- **Crate point *P*.** The centre of the crate's cell at its floor height: (256*x* + 128, 256*y* + 128, *h*), with *h* the terrain surface height at the cell centre (S-0106 claims 1 and 2), without a bridge-deck lift. Coordinates are integer leptons.
- **Crate radius ρ.** A game-wide distance in leptons (Import mapping, F3).
- **Crate area.** An object is inside the crate area when its distance *d* from *P* (F4) is less than ρ. For a building, the distance is measured to its reference point (S-0204 claim 4). For every other object it is measured to its position.
- **Demolition warhead.** As in S-0524 claim 5.
- **Flame warhead.** The rules' game-wide flame-damage warhead (Import mapping).
- **Gas warhead.** The warhead the rules register under a fixed name (Import mapping).
- **Crate ownership counts.** Counts of objects in existence, including objects off the map and in production (S-0414 claim 4). Types marked insignificant or not scored are excluded. The building count excludes buildings that count as vehicles (S-0414 claim 4). The vehicle count includes vehicles, ships and those buildings that count as vehicles. The infantry count excludes nominal survivors (S-0414 claim 4) and absorbed infantry (S-0312 claim 24). Type-specific base-unit, refinery and harvester counts count objects in existence of the named type, with the same type-flag exclusions; a deployed base unit is a building, not a vehicle of its former type.

## Behaviour

### Placement, lifetime and replacement

1. **Crate places.** At most 256 crates placed through crate places exist at once. A placement that finds every place occupied does nothing and takes no draw. Removing a crate through pickup or expiry frees its place.
   - Backing: `measurable` (scenario CR1)
   - Confidence: high

2. **Game-start crates.** With the crate setting on, S-0412 claim 10 fixes how many crates appear at game start. Each one is a random placement (claim 3), with consecutive draw groups in placement order: one placement's complete draw group precedes the next placement's first request.
   - Backing: `measurable` (scenario CR1)
   - Confidence: high

3. **Random placement.** When no crate place is free, placement has no effect and takes no draw. Otherwise it makes at most 1000 candidate attempts, stopping if one succeeds. Each candidate is determined by two ranged requests, with *x* requested before *y* as specified by F5; a candidate outside the map's cell array is treated as ordinary land. Around that candidate, the game uses the single-cell nearby search and the settings stated in S-0212 claim 25 and the trigger-crate row of T5: *float* locomotion class for a *water* candidate and *track* otherwise, *normal* movement zone, no reach region, deck mode off, no height condition, no further test, no no-overlay condition, bridges allowed, and no reference cell. This search takes no draw and requires an empty occupancy record, so the selected cell cannot contain an object. The candidate attempt fails without further draws if the search finds no cell, or if its selected cell is outside the height-aware usable area or has an overlay. For the first attempt meeting all those conditions, the first free crate place receives random contents and a water-crate overlay on *water*, or a wooden-crate overlay otherwise. Any draws caused by overlay placement precede the lifetime request; the lifetime uses S-0524 claim 4's formula and bounds (half to twice the regeneration field, in game minutes) and begins at the current tick. Placement redraws the cell without revealing it to any house. If no attempt succeeds within the limit, no crate appears and no later retry occurs.
   - Backing: `measurable` (scenario CR1) for ordinary candidates. Out-of-array candidate treatment is `unmeasured`: CR1 supplies no safe map fixture that reaches that case.
   - Confidence: high for the draws, the search and the overlay choice; medium for the out-of-array candidate

4. **Expiry and replacement.** In a skirmish with the crate setting on, expiry occurs in the crate-update part of the tick (S-0203 claim 1, tick-order row 16) when elapsed time reaches the lifetime. Simultaneously expired crates have replacement draw groups in increasing place-priority order. Each replacement sees its predecessor's place as free and all earlier replacements as already present, and makes one random placement (claim 3), which can fail. A replacement's priority cannot exceed its predecessor's, since that predecessor's place is free. Consequently the replacement cannot expire again in that update, even with a zero-tick lifetime. With the crate setting off, the update does nothing: no crate expires, including trigger crates (S-0524 claim 4, subject to that spec's optional expiry departure). Each expiry, like each pickup with the crate setting on (claim 11), removes one crate and adds at most one random crate. Only death crates (claim 5) and trigger crates raise the count, and a failed replacement lowers it.
   - Backing: `measurable` (scenario CR2)
   - Confidence: high

5. **Other crate sources.** Crates from destroyed crate-carrying vehicles (S-0312 claim 14) have random contents. Crates from a building's crate beneath (S-0312 claim 20) hold the money power-up (0) when the type says so and random contents otherwise. Trigger crates hold the contents S-0524 claim 4 gives. All of them occupy crate places, expire and are collected by this spec's rules.
   - Backing: `measurable` (scenario CR3)
   - Confidence: high for the contents values; the placement rules are those specs'

### Pickup

6. **Who collects, and when.** A crate is collected by a ground mover when it commits to its next step into the crate's cell, before it arrives there. Ground movers are vehicles using the drive, ship, hover or walking-vehicle locomotor, and walking infantry. A jumpjet collects the crate of the cell it lands on, when it lands. An object moving with its own teleporting locomotor, of any kind including aircraft, attempts pickup in its landing cell after placement and cell-arrival effects (S-0207 claim 20). The following arrivals do not themselves collect:
   - aircraft flight or landing with the flying locomotor.
   - superweapon chronoshift (S-0513 claims 37–40), a separate arrival path from own-locomotor teleporting.
   - tunnelling, drop-pod or rocket movement.
   - an object created on the cell or unloaded onto it.

   A ground mover that is blocked by the pickup (claims 7 and 13) does not commit that step into the cell. Its next movement target is cleared; this does not guarantee that its standing order will never try again. A jumpjet or own-locomotor teleporter has already landed before pickup: a pickup result that would block a ground step does not undo that landing.
   - Backing: `measurable` (scenario CR4), except creation-only, drop-pod and rocket arrivals are `unmeasured`: no isolated legal fixture for creation directly on an existing crate, a drop-pod arrival, or a surviving rocket arrival onto one has been established. These exclusions are inferred from the pickup entry points, not completed tests.
   - Confidence: high for the step timing of vehicles and infantry, jumpjet landing, own-locomotor teleport pickup and flying-locomotor exclusion; medium for the other exclusions

7. **Pickup gates.** The table applies at each pickup attempt of claim 6. For a ground step, "enters" means the mover proceeds normally and "blocked" means it does not commit the step. For a jumpjet or own-locomotor teleport landing, placement has already happened and neither outcome reverses it.

   | Condition | Outcome |
   |---|---|
   | The cell's overlay lacks the crate flag | no pickup, the mover enters |
   | The collecting house's country has the passive flag (skirmish) | no pickup, the crate stays, the mover enters |
   | The overlay has the crate-trigger flag and the collector carries a tag | the tag is raised with the crate-pickup event (49), with the collector as causing object (S-0011 claim 8). If the collector is no longer alive afterwards, nothing more happens: the crate stays and a ground step is blocked |
   | The overlay has the crate-trigger flag and the pickup continues | the scenario's any-crate-picked-up mark associated with event 50 is set |
   | Otherwise | the pickup continues (claim 8) |

   The no-pickup conditions suppress all later outcomes in the table. A tagged raise that kills the collector leaves both the crate and the any-crate mark unchanged by this attempt; its actions cannot observe a mark newly set by this attempt. A surviving pickup sets the mark only for a crate-trigger overlay. This establishes the mark, not event 50's satisfaction at a poll or a trigger firing. Event 50 belongs to both the polled and code-matched sets in S-0011; their interaction remains open.
   - Backing: `measurable` (scenario CR5)
   - Confidence: high

8. **Choosing the power-up.** If the crate's contents are 0 … 18, that is the power-up. New random crates store contents 255: this applies to random placement, its replacements, random death crates and trigger placement with number-slot value exactly 20. Other trigger values are stored as S-0524 claim 4 specifies; in particular 19 stores 19, 21 stores 21, 255 stores 255, and 276 stores 20. All four select random contents at pickup. For contents outside 0 … 18, one ranged draw *r* = R(1, Σ*w*) picks it, where Σ*w* is the sum of the nineteen imported weights. The chosen power-up is the first in table order whose running weight total reaches *r* (F1). Claims 9 and 10 can then change it.

   | No. | Power-up | No. | Power-up |
   |---|---|---|---|
   | 0 | money | 10 | speed |
   | 1 | unit | 11 | firepower |
   | 2 | heal all | 12 | nuclear missile |
   | 3 | cloak | 13 | invulnerability |
   | 4 | explosion | 14 | veteran |
   | 5 | napalm | 15 | weather storm |
   | 6 | squad | 16 | gas |
   | 7 | darkness | 17 | resource |
   | 8 | reveal | 18 | drop pod |
   | 9 | armour | | |
   - Backing: `measurable` (scenarios CR6 and CR20)
   - Confidence: high

9. **Forced base unit.** The power-up becomes *unit*, with a forced base unit, when all of these hold:
   - the collecting house's crate building count is zero (Terms).
   - its available credits (S-0404 claim 4) exceed 1500.
   - its type-specific crate count for its base-unit type is zero: that type is the first entry of the game-wide base-unit list that its country may own.
   - the bases setting is on.

   This override applies even to crates with fixed contents. Claim 10's conversions also apply to this forced unit, so it can still become money.
   - Backing: `measurable` (scenario CR7), except nominal-survivor accounting is `unmeasured`: no reliable fixture producing that state has been identified (Open questions).
   - Confidence: high for the conditions and building exclusions; medium for every off-map and production case

10. **Conversions to money.** After claims 8 and 9, a power-up that cannot apply becomes *money* (0):

    | Power-up | Becomes money when |
    |---|---|
    | unit | the house's crate vehicle count exceeds 50, or the crate cell's land type is *water* or *beach* |
    | squad | the house's crate infantry count exceeds 100, or the crate cell's land type is *water* or *beach* |
    | cloak | the collector's own cloak flag is already set |
    | armour | the collector's armour multiplier is not exactly 1.0 |
    | speed | the collector's speed multiplier is not exactly 1.0, or the collector is an aircraft |
    | firepower | the collector's firepower multiplier is not exactly 1.0, or the collector has no primary weapon |
    | veteran | the collector's type cannot gain experience, or the collector is elite |
    | any power-up | the crate cell's land type is *water* and the power-up's water flag is off |

    The water-flag restriction applies to the result of the other conversions, including money (which stays money). These eligibility conditions take no draws.
    - Backing: `measurable` (scenario CR7), except nominal-survivor accounting is `unmeasured` for the reason in claim 9.
    - Confidence: high for the tests and type-flag exclusions; medium for off-map and nominal-survivor count cases

11. **Pickup results and draw precedence.** A continuing pickup removes its crate and frees its place. With the crate setting on, it produces at most one random replacement (claim 3). The effect sees the old crate absent and any successful replacement present; replacement draws precede every effect draw. In particular, a resource increase cannot germinate a cell occupied by that replacement.

    Squad (6) gives the money effect. The argument *A* belongs to the resulting power-up: squad and every claim-10 conversion to money use the money argument. The effect is specified in claims 12 to 25.

    The resulting power-up's imported animation, when present, appears at (256*x* + 128, 256*y* + 128, *h* + 200), with its creation draws (S-0202 claim 10). Its creation draws follow the effect's draws, including any caused by damage or deaths. A successfully placed unit has no such animation (claim 13); a power-up turned into money shows the money animation.

    A surviving ground collector may then enter the cell, except after a successful unit power-up (claim 13). A jumpjet or own-locomotor teleporter has already landed (claim 6).
    - Backing: `measurable` (scenarios CR6, CR8)
    - Confidence: high

### Effects

12. **Money (0).** The collecting house receives *R*(*b*, *b* + 900) credits, one ranged draw, where *b* is the argument truncated to an integer (F2).
    - Backing: `measurable` (scenario CR8)
    - Confidence: high

13. **Unit (1).** The selected type is determined by the first applicable rule in this priority order. A type named by the unit-crate field takes precedence, including over a forced base unit. Otherwise, a forced base unit uses the collecting house's base-unit type. Otherwise, when the house has a positive type-specific crate count for either of the first two types in the game-wide refinery list and zero counts for both of the first two types in the game-wide harvester list, the selected type is the first harvester-list type its country may own (`quirks.crate_harvester_list_prefix`, Edge cases). If that condition holds but no harvester-list type is ownable, selection instead uses the random rule. In every other case, including when none of those rules selects a type, the game repeatedly requests a type uniformly by rules-order index from the *n* vehicle types, using R(0, *n* − 1) for each request. The first requested type that is accepted is selected. A type is accepted only if it has the crate-goodie flag; a type in the base-unit list is additionally refused when the bases setting is off. When the bases setting is on, being in the base-unit list does not prevent acceptance if the crate-goodie requirement is met.

    The unit is created for the collecting house (with its creation draw, S-0202 claim 10), facing 0, and offered placement at the crate cell centre at ground height. This is a direct placement attempt, not the first candidate of the nearby search. It accepts only the awarded unit's *clear* entry outcome under S-0209 claim 5 and S-0211 claim 10, with no approach direction or source elevation, ground-layer occupants and ground occupancy record, and no forced-placement exemption. Every non-clear outcome refuses placement: traversable terrain alone is insufficient. An ordinary stationary vehicle of the collecting house in the crate cell therefore refuses this placement. The collector is not ignored: its actual listing and reservations at pickup count normally, but an incoming collector outside the cell is not treated as already occupying it merely because it requested pickup. Special occupant relations and crushability retain their entry-table results rather than imposing a blanket empty-cell test.

    If that fails, S-0212 claim 25's nearby search runs around the crate cell: the type's locomotion class, *normal* movement zone, no reach region, deck mode off, no height condition, no further test, no no-overlay condition, bridges allowed, no reference cell. Unlike direct placement, this search requires the empty occupancy record of S-0212 claim 25 and uses the stated *normal* movement zone, not the awarded type's own zone. A search result is offered the same direct placement test, once, at its cell centre. Selection by the search is not a guarantee of successful placement, and a refused result does not start another search. If the search finds no cell or that placement fails, the unit is deleted (its creation draw stays taken) and the power-up gives money: R(*b*, *b* + 900) credits as in claim 12, with the money animation, but with *b* the truncated argument of the *unit* power-up, not of money (`quirks.crate_unit_fallback_unit_argument`, Edge cases).

    After a successful placement a ground collector does not commit its step into the crate cell (claim 6). A jumpjet's or own-locomotor teleporter's landing is not reversed. The new unit stands in the crate cell or nearby.

    **In our engine** (a departure for AI houses). With the bases setting on, the original also refuses a base-unit-list type for a computer-controlled house unless the unit is forced. By the project lead's decision (glossary *AI house*), an AI house follows the rules for human-controlled houses, so our engine accepts the type for every player house. Neutral and special houses retain the original's non-human restriction. There is no switch. The original's rule is recorded for AI designers.
    - Backing: `measurable` (scenario CR9)
    - Confidence: high for the order of rules, the draws and the fallback to money; medium for the placement height

14. **Heal all (2).** Every object taking simulation turns and owned by the collecting house, anywhere on the map, is offered a hit of (its health − its type's strength), in signed integer health points. The hit ignores defences, carries the no-escape marker, uses the demolition warhead, and has no attacker and no house (S-0302 claim 1). An ordinary damaged object is thereby restored to full strength, subject to S-0302 claims 7, 8 and 11: this is a healing hit, not an unconditional health assignment. Buildings, vehicles, infantry and aircraft are included. Passengers in ordinary closed transports or inside buildings are out of play and are not included. Passengers in open-topped transports remain in play and are included even though they are off the map (S-0203 claim 3).
    - Backing: `measurable` (scenario CR10) for ordinary on-map and closed/open-topped transport states. Other off-map states are `unmeasured`: no exhaustive reachable-state fixture set has been established (Open questions).
    - Confidence: high for the hit and ordinary/open-topped transport distinction; medium for other off-map states

15. **Cloak (3).** Every building, vehicle, infantry unit and aircraft on the map inside the crate area gets its own cloak flag set (S-0516 claim 2). Every owner is included, enemies too. Terrain objects, projectiles and objects outside the map are not affected.
    - Backing: `measurable` (scenario CR11)
    - Confidence: high

16. **Explosion (4).** With *D* the argument truncated to an integer (F2), the collector takes a hit of *D* that ignores defences, has no no-escape marker, uses the demolition warhead, and has neither attacker nor house. The effect also produces five blast points, including when that direct hit kills the collector. Those blasts see any deaths and trigger changes caused by the direct hit, and their draws follow its resulting draws. For each point in their observable order, the game makes a ranged draw *r* = R(0, 512), obtains *Q* = N(*P*, *r*, *b*) using S-0312 claim 22's nearby-point formula (one raw draw, with no snapping to a cell centre), and deals area damage *D* at *Q* using the demolition warhead, no source or house, and allowing resource reduction (S-0303 claims 1 and 17). At that same point it creates the explosion animation selected by S-0303 claim 21's formula from *D* and the demolition warhead as if the land type were not *water*, so the splash case is not selected, and creates a drawing-only light flash. Each blast's damage and resulting deaths are resolved before the next blast's draws.
    - Backing: `measurable` (scenario CR12)
    - Confidence: high

17. **Napalm (5).** Let *C* be the collector's position (its reference point for a building) and *M* the midpoint of *P* and *C* (F6). The animation at position 0 of the animation type list appears at *M*. With *D* the argument truncated to an integer (F2), the collector takes a hit of *D* with the flame warhead: defences ignored, no no-escape marker, no attacker and no house. Area damage *D* also occurs at *M* with that warhead, no source or house, and resource reduction allowed.

    The initial animation's creation and immediate start effects precede the collector's direct hit, so that hit sees any resulting state changes; its animation draws precede any direct-hit draws. The area damage sees deaths and trigger changes caused by that hit and occurs even if the collector dies. The original argument supplies *D* independently for both damages.
    - Backing: `measurable` (scenario CR12) for damage, position, animation selection and draw precedence. A gameplay-changing initial animation's influence on the direct hit is `unmeasured`: CR12 has no discriminating state-change fixture.
    - Confidence: high

18. **Darkness (7) and reveal (8).** Darkness re-covers the whole map for the collecting house. Reveal reveals the whole map for it (S-0409 claim 15).
    - Backing: `measurable` (scenario CR13)
    - Confidence: high

19. **Armour (9) and firepower (11).** Every object inside the crate area whose armour multiplier (firepower multiplier) is exactly 1.0 has it multiplied by *A* in double-precision arithmetic (S-0302 claim 4). Buildings, vehicles, infantry and aircraft of every owner are included; terrain objects and projectiles are not. A multiplier that is not exactly 1.0 is left unchanged, so bonuses never stack. Objects outside the map are included when their last position lies inside the crate area (`quirks.crate_bonus_reaches_limbo`, Edge cases).
    - Backing: `measurable` (scenario CR14)
    - Confidence: high

20. **Speed (10).** Every vehicle, infantry unit or ship inside the crate area whose speed multiplier is exactly 1.0 has it multiplied by *A* in double-precision arithmetic (S-0207 claim 13, S-0208 claim 7). Aircraft and buildings are never affected. Objects outside the map are treated as in claim 19.
    - Backing: `measurable` (scenario CR14)
    - Confidence: high

21. **Nuclear missile (12).** If the rules hold a superweapon type of the nuclear-missile kind, the collecting house gets a one-time grant (S-0512 claim 12) of the first superweapon type, in rules order, whose action is the nuke action. If no type of that kind exists, nothing happens. A type of that kind with no nuke-action type is unsafe data (Edge cases).
    - Backing: `measurable` (scenario CR15)
    - Confidence: high

22. **Veteran (14).** Only when *A* > 0: every building, vehicle, infantry unit and aircraft on the map inside the crate area whose type can gain experience goes up one veterancy level (S-0311 claim 1) ⌈*A*⌉ times (F7). Each step sets the experience to exactly the next level, so partial progress is lost. An elite object stays elite. Every owner is included.
    - Backing: `measurable` (scenario CR16), except F7's negative starting-experience case is `unmeasured`: no legal fixture establishing negative experience has been identified.
    - Confidence: high

23. **Gas (16).** With *D* the argument truncated to an integer (F2), area damage *D* with the gas warhead, no source, no house and resource reduction forbidden, happens at the crate cell's centre and then at the centre of each of its eight neighbour cells in step-direction order 0 through 7. That is nine detonations: each sees any deaths or trigger changes from the preceding detonations, and its resulting draws precede the next detonation's draws.
    - Backing: `measurable` (scenario CR17)
    - Confidence: high

24. **Resource (17).** A ranged draw selects resource type *t* = R(0, *k* − 1) from the *k* resource types in rules order. Position 1 is treated as position 0. The crate cell then gains one stage of *t*, according to the stage-increase rule below. A further ranged request *n* = R(10, 20) determines the number of additional stage-increase attempts. For each of those *n* attempts, the game draws *r* = R(0, 768), obtains *Q* = N(*P*, *r*, *b*) using S-0312 claim 22's nearby-point formula, snaps *Q* to the cell centre, and applies one stage increase of *t* to the cell containing that centre.

    A stage increase has the following observable result. A germinable cell (S-0401 claim 10) becomes a new resource cell of *t* at stage 1, with S-0401 claim 11's draws. A cell already holding *t* gains one stage only when S-0401 claim 7's growth-step conditions hold; it is then offered to *t*'s spread list (S-0401 claim 8), receives no new growth-list entry from the crate, and retains any growth-list entry it already had. The count used here is the resource stage-picture count (Terms): 12 for every image number establishing a run in S-0112 claim 10, equal to *F*, not *F* + *S* or the sprite frame count *N*. Thus the highest resulting stage is 11, independent of the number of frames in the supplied sprite. An existing stage above 11 is unchanged, not reduced to 11. The general rule refuses the whole increase, including germination, when that count is at most one, and caps an existing cell at one less than the count; no supported image-number import establishes such a one-picture count. A cell holding another resource, and every other cell, remains unchanged.
    - Backing: `measurable` (scenario CR18) for type choice, draws, germination and stage changes. Exact growth/spread-list membership after an increase is `unmeasured`: CR18's subsequent timing cannot by itself separate retained entries, duplicate entries and independent scheduling. The count-at-most-one guard is also `unmeasured`: no supported import establishes it; CR18 states the reachable sprite-count control instead.
    - Confidence: high for the draws; medium for the growth conditions

25. **Power-ups with no dedicated effect.** Invulnerability (13), weather storm (15) and drop pod (18) only create their imported animation. Any effects of that animation still follow S-0109 claims 13–17. Squad (6) always gives money (claim 11).
    - Backing: `measurable` (scenario CR6)
    - Confidence: high

26. **Sounds and announcements.** These are local-player feedback only. They take no scenario draws and change no game state.
    - Money, reveal, firepower, armour, speed, unit and veteran each play their imported crate sound at *P* when the collector belongs to the local player. Heal all plays its own sound.
    - Armour, speed and firepower also give the matching upgrade announcement when an affected object belongs to the local player.
    - The nuclear missile adds its sidebar icon for the local player (S-0512 claim 12).
    - Backing: `measurable` (scenario CR8)
    - Confidence: medium

27. **Observable draw order of one pickup.** The scenario-stream draws of one pickup come in this order:
    - the tag raise's own draws (claim 7).
    - the power-up draw (claim 8, only for random contents).
    - with the crate setting on, the replacement placement's draws (claim 3).
    - the effect's draws in the order its claim gives.
    - the final animation's creation draws.

    The unit power-up takes its type draws, then the creation draw, then any draws its placement causes. Area damage and death effects bring the draws of S-0303 claims 17–22 and S-0312 claims 1 and 4–23.
    - Backing: `measurable` (scenario CR6, with the random-state recorder)
    - Confidence: high for the order; medium for draws inside damage, deaths and animation creation, which those specs own

28. **The power-up table is layered, not merged text.** Each applied rules layer of S-0006 claim 1 is interpreted against the values in force before that layer. With original behaviour, a layer lacking the power-up section changes none of its nineteen entries. A layer containing the section resets every omitted entry's weight to zero and its animation to no animation, while retaining its water flag and argument. The reset cannot select a separately defined animation with the reserved bare no-type name: neither an animation-list entry nor an ordinary creating animation reference registers that name (S-0003 claim 14; S-0004 claims 2 and 13). A section bearing that name alone creates no animation type either (S-0004 claim 7). Thus the reset always gives no animation for supported skirmish imports; it does not create an animation type. A present short entry changes only the tokens it supplies. Consequently a section in an intermediate layer can erase a weight or animation even when a still later layer omits the section; looking only at each key's final text cannot reproduce this. Section presence means a section that survives the empty-entry rules of S-0006 claim 3. S-0006 claim 5 already specifies this reset rule and owns `quirks.override_resets_keys`: with it off, an omitted entry retains all four held values instead.
    - Backing: `measurable` (scenario CR19)
    - Confidence: high

## Formulas

**F1. Power-up choice (claim 8).** With integer weights *w*₀ … *w*₁₈ and *W* = Σ *w*ᵢ, *r* = R(1, *W*). The power-up is the least *k* with *w*₀ + … + *w*ₖ ≥ *r*. Integer arithmetic with signed 32-bit wrap, no rounding. Without integer overflow, a zero or negative entry cannot itself be chosen: if its preceding total is below *r*, adding that entry cannot reach *r*. Negative entries can reduce later running totals. Summation uses signed 32-bit arithmetic (S-0205 claim 3); a non-positive total is unsafe data (Edge cases). CR6 tests positive, zero and negative entries with a positive non-overflowing total. Overflowing totals are `unmeasured`: no safe fixture with a fully defined resulting selection has been established; confidence medium.

**F2. Truncated argument (claims 12, 16, 17, 23).** *b* = trunc(*A*), the double-to-integer conversion toward zero of S-0205 claim 7 (its out-of-domain rules apply). Money: amount = R(*b*, *b* + 900), with *b* + 900 in signed 32-bit integer arithmetic.

**F3. Crate radius (Import mapping).** For a read decimal *v* (double): if *v* = −1 exactly, ρ keeps its previous value. Otherwise ρ = trunc(*v* × 256), with the product rounded as S-0205 claim 5 says and the conversion of S-0205 claim 7. Units: *v* in cells, ρ in leptons.

**F4. Crate-area distance.** With (*Δx*, *Δy*, *Δz*) the signed integer lepton differences between *P* and the object's measured point, the squared terms and their sum are calculated in double-precision floating point, rounded under S-0205 claim 5. Then *d* = trunc(√(*Δx*² + *Δy*² + *Δz*²)), using S-0204 claim 5's table square root (`quirks.table_root_distance`), with conversion toward zero under S-0205 claim 7. Inside means *d* < ρ, a strict integer comparison in leptons.

**F5. Random candidate cell (claim 3).** With *W*ₘ and *H*ₘ the map width and map height of the map size (glossary): *x* = 1 + R(0, *W*ₘ + *H*ₘ − 2), then *y* = 1 + R(0, *W*ₘ + *H*ₘ − 2). Both lie in 1 … *W*ₘ + *H*ₘ − 1. Integers, exact.

**F6. Napalm midpoint (claim 17).** *M* = (trunc((*P*ₓ + *C*ₓ) / 2), trunc((*P*ᵧ + *C*ᵧ) / 2), trunc((*P*_z + *C*_z) / 2)), signed integer division truncated toward zero. Units: leptons.

**F7. Veteran steps (claim 22).** The number of steps is the count of integers *k* ≥ 0 with *k* < *A*, compared in double precision: ⌈*A*⌉ for *A* > 0, and 0 otherwise. One step maps an experience value *e* to 2.0 when 1.0 ≤ *e* < 2.0, to 1.0 when 0.0 ≤ *e* < 1.0, and to 0.0 when *e* < 0.0. An elite object (*e* ≥ 2.0) is unchanged.

**F8. Multiplier update (claims 19, 20).** When the starting multiplier is exactly 1.0, the resulting multiplier is *A*. The multiplication is in double precision and follows S-0205 claim 5's rounding rule; with a starting multiplier of exactly 1.0 the product is exactly *A*. Other starting multipliers are unchanged.

**Lifetime (claim 3).** S-0524 claim 4's trigger-crate lifetime formula applies unchanged: one draw R(0, 2147483646), bounds 450*C* and 1800*C* ticks, with *C* the regeneration field.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Crate capacity | 256 | crates | shared with S-0524 claim 4 |
| Placement attempts | 1000 | attempts per random placement | claim 3 |
| Power-up count | 19 | power-ups | numbered 0 … 18 |
| Fixed-contents range | 0 … 18 | overlay data value | other values are random |
| Forced-base-unit credit threshold | 1500 | credits | strictly more than |
| Unit power-up vehicle limit | 50 | vehicles | more than this gives money |
| Squad power-up infantry limit | 100 | infantry | more than this gives money |
| Money spread | 900 | credits | R(*b*, *b* + 900) |
| Animation height | 200 | leptons above *h* | claim 11 |
| Explosion blasts | 5 | blasts | claim 16 |
| Explosion spread | 0 … 512 | leptons | claim 16 |
| Gas detonations | 9 | detonations | crate cell and 8 neighbours |
| Resource repeats | 10 … 20 | cells | claim 24 |
| Resource spread | 0 … 768 | leptons | claim 24 |
| Crate radius default | 640 | leptons | absent key (2.5 cells) |
| Regeneration default | 10 | game minutes | absent key; S-0524 claim 4 |

## State and ordering

| State | Event | Result |
|---|---|---|
| place free | random placement succeeds | place occupied, lifetime drawn and started |
| place occupied | lifetime reached, crates on | crate removed, one random placement made |
| place occupied | lifetime reached, crates off | nothing (crate remains) |
| place occupied | ground-step pickup attempt, jumpjet touchdown or own-locomotor teleport landing | claims 6 to 11 |
| place occupied | pickup blocked by a dying tagged collector | crate remains, mover stops |
| place occupied | passive-country collector | crate remains, mover enters |

Ordering consequences:
- The replacement crate of a pickup exists before the effect happens, and the effect sees it. For example, a resource stage increase that falls on the replacement crate's cell finds an overlay there, so that cell is not germinable (S-0401 claim 10).
- The tag raise happens before the power-up is chosen. A trigger that kills the collector cancels the pickup and keeps the crate.
- Simultaneous expiries consume replacement draw groups in increasing place-priority order. A replacement cannot expire again in that update, even with zero lifetime (claim 4).

## Edge cases and quirks

- **`quirks.crate_harvester_list_prefix`** (default on). The harvester rule of claim 13 tests only the first two entries of the refinery list and of the harvester list. A house whose only refinery is a later list entry gets no harvester from the rule. Off: every entry of both lists is tested, and the rule applies when the house owns any listed refinery and no listed harvester.
- **`quirks.crate_unit_fallback_unit_argument`** (default on). When a unit power-up cannot place its unit (claim 13), the money it gives uses the unit power-up's argument as the base *b*. Off: it uses the money power-up's argument, exactly as a money crate would.
- **`quirks.crate_bonus_reaches_limbo`** (default on). Armour, firepower and speed (claims 19, 20) do not require the object to be on the map. An object inside a transport, a building or other off-map state whose last recorded position lies inside the crate area is boosted. Off: only objects on the map are affected, as for cloak and veteran.
- **Position-based data selection, not a quirk.** Resource position 1 maps to position 0 (claim 24); napalm uses animation position 0 (claim 17). These are the effect definitions, not failed type lookups: no separately named intended type has been established. List reordering deliberately changes these results. No repair alternative or switch is specified. Backing: `measurable` (CR12, CR18). Confidence: high.
- **Gas warhead default, not a quirk.** If the named gas warhead is absent, a warhead with all fields at their absent-key defaults is created and used. This is normal named-data/default selection, not an established defect; it has no switch. Backing: `measurable` (CR17). Confidence: high.
- **`quirks.crate_phantom_place`** (default on). If a selected cell passes the placement tests but a terrain object prevents its overlay appearing, the place still becomes occupied and takes its lifetime draw. It remains occupied until expiry. Off: the failed overlay placement takes no place or lifetime draw. The nearby search normally rejects these cells, so this conditional result is `unmeasured`: no reachable stock search result that exercises it has been identified; confidence medium.
- **Non-stacking eligibility, not a quirk.** Exact equality to 1.0 is the eligibility rule, not a remembered once-only upgrade flag (claims 10, 19 and 20). A collector with a different multiplier converts the matching power-up to money; neighbours with different multipliers are skipped individually. A multiplier still equal to 1.0 remains eligible, regardless of earlier pickups. This defines the non-stacking game rule; no evidence establishes it as a defect, so no switch is proposed. Backing: `measurable` (CR7, CR14). Confidence: high.
- **Unsafe data.** In each of the following cases our engine warns at import and the stated outcome applies:
  - no vehicle type can be accepted by the unit power-up's draws: the original never stops redrawing. Our engine gives money instead.
  - a nuclear-missile-kind superweapon type exists but no type has the nuke action: the original fails. Our engine gives no grant.
  - a weight total of zero or less: our engine gives money.
  - no base-unit-list type that the house's country may own, when claim 9 would test one: the original fails. Our engine treats the house as owning one, so the override does not apply.
  - a missing flame warhead for napalm: our engine skips the hit and the area damage.

## Import mapping

| Key | Form | Concept | Default when absent |
|---|---|---|---|
| `[Powerups]` `Money`, `Unit`, `HealBase`, `Cloak`, `Explosion`, `Napalm`, `Squad`, `Darkness`, `Reveal`, `Armor`, `Speed`, `Firepower`, `ICBM`, `Invulnerability`, `Veteran`, `IonStorm`, `Gas`, `Tiberium`, `Pod` | list: weight (integer), animation type name, water flag (`yes` or `no`), argument (decimal; a value containing `%` anywhere is multiplied by 0.01) | power-ups 0 … 18 in this order | Applied separately in each rules layer (claim 28), not just to each key's final text. Key absent while that layer's section exists: weight 0, no animation, water flag and argument kept (S-0006 claim 5). The reset's lookup text is `NONE`, but that name cannot be registered as an animation by the type list or creating references (S-0004 claims 2 and 13); this lookup never creates a type. A standalone `[NONE]` section does not make it reachable. A present short list retains omitted tokens. No section in that layer: all four held values survive. At a fresh program start: built-in weights 50, 20, 1, 3, 5, 5, 20, 1, 1, 10, 10, 10, 1, 3, 1, 1, 1, 1, 1 in power-up order, no animations, water flags off, arguments 0.0 |
| `[Animations]` entries, animation references and an art section named `[NONE]` | type name or type section | reserved-name controls for the reset (claim 28) | `NONE` and `<none>` in any ASCII letter case register no animation through the list or ordinary creating animation references (S-0003 claim 14; S-0004 claims 2 and 13). Merely supplying the art section creates no type; an absent declaration likewise creates none |
| `[CrateRules]` `CrateRadius` | decimal, cells | ρ (F3) | 640 leptons; a value of exactly −1 keeps the previous value |
| `[CrateRules]` `CrateRegen` | double, game minutes | lifetime bounds | 10 (S-0524 claim 4) |
| `[CrateRules]` `CrateMinimum`, `CrateMaximum` | integer | game-start count | S-0412 claim 10 |
| `[CrateRules]` `UnitCrateType` | vehicle type name | unit-crate type (claim 13) | none |
| `[CrateRules]` `WoodCrateImg`, `WaterCrateImg` | overlay type name | wooden-crate and water-crate overlays | S-0524 claim 4 |
| `[CrateRules]` `HealCrateSound` | sound name | heal-all sound | none; an unknown name keeps the previous value |
| `[CrateRules]` `FreeMCV` | boolean | read, no observed effect | ignored |
| `[CrateRules]` `CrateImg`, `SoloCrateMoney`, `SilverCrate`, `WoodCrate`, `WaterCrate` | — | campaign only | ignored in a skirmish |
| `[AudioVisual]` `CrateMoneySound`, `CrateRevealSound`, `CrateFireSound`, `CrateArmourSound`, `CrateSpeedSound`, `CrateUnitSound`, `CratePromoteSound` | sound name | crate sounds for money, reveal, firepower, armour, speed, unit, veteran | none; an unknown name keeps the previous value |
| `[CombatDamage]` `C4Warhead` | warhead name | demolition warhead | none |
| `[CombatDamage]` `FlameDamage` | warhead name | flame warhead | none (Edge cases) |
| warhead section named `GAS` | warhead | gas warhead | created with defaults (Edge cases) |
| `[General]` `BaseUnit` | list of vehicle types | base-unit list (claims 9, 13) | empty |
| `[General]` `HarvesterUnit` | list of vehicle types | harvester list (claim 13) | empty |
| `[AI]` `BuildRefinery` | list of building types | refinery list (claim 13) | empty |
| vehicle type `CrateGoodie` | boolean | may come from a unit crate | no |
| vehicle, infantry, aircraft and building type `Trainable` | boolean | can gain experience (claims 10, 22) | yes (S-0311 claim 8) |
| type `Insignificant`, `DontScore` | boolean | exclusions from crate ownership counts (Terms, claims 9, 10 and 13) | no (S-0414 claim 4) |
| building `UndeploysInto`, art `Foundation`, undeploy vehicle `Harvester` | vehicle reference, foundation shape, boolean | buildings counted as vehicles in crate ownership counts | S-0414 claim 4 |
| country `MultiplayPassive` | boolean | passive country (claim 7) | no |
| overlay type `Crate` | boolean | crate flag | no |
| overlay type `CrateTrigger` | boolean | crate-trigger flag (claim 7) | no |
| superweapon type `Type` | kind (the nuclear-missile kind name, S-0512 claim 1) | nuclear-missile kind (claim 21) | S-0512 claim 1 |
| superweapon type `Action` = `Nuke` | action word | nuke action (claim 21) | no action |
| resource type `Image` | integer image number | resource stage-picture count used by claim 24 | a value other than −1 establishing a run sets the count to 12, equal to S-0112 claim 10's *F*; absent or −1 leaves the held value unchanged, and a type never given a run is unsafe under S-0112 claim 12. Sprite file frame count does not set this value |
| trigger crate number slot | signed integer, exact reduction modulo 256 except the explicit value 20 | stored contents (claim 8) | value exactly 20 retains 255 from new crate placement; every other value stores its reduction under S-0524 claim 4 |
| lobby crate and bases settings | boolean | crate setting, bases setting | S-0412 claims 1 and 10 |

## Measurement scenarios

Use the glossary's measurement setup: stock data, director and mods off except the named test data below, a purpose-made skirmish map, one human and one bystander computer player, standard mode, 800 × 600, game-speed index 6. The observed area is the whole map for placement, shroud and heal-all tests; otherwise it is the test enclosure and its crate destinations. Keep the bystander separated, and apply its void rule. Use normal input or scripted mouse and keyboard input, tick-labelled captures, and read-only recording of credits, health, positions and random-stream state. Do not write a seed into the oracle. Repeat deterministic variants twice; use 30 independently started games for random variants unless a larger count is stated, recording each observed seed and state.

**Named test data.** A test copy of `rulesmd.ini`, removed after each run, may change only the fields a scenario names by role: crate weights, arguments, animations, water flags, lifetime and count settings, the forced unit type, type eligibility flags and lists, and explicitly stated test-unit or warhead properties. CR19 may additionally supply animation-list entries, creating animation references and a named test copy of the art file defining a harmless visible animation for its reserved-name controls. A purpose-made map supplies trigger crates with fixed contents (S-0524 claim 4) and separated test enclosures. Prefer those fixed crates with the crate setting off to isolate effects. Keep a counted building and a base unit unless the scenario tests their absence, to prevent claim 9 masking the intended effect. Named data is an explicit exception to stock-data-only tests, not an engine modification.

- **CR1 — placement (claims 1–3).** Use an open mixed-land-and-water map, then one with almost every cell blocked by objects or overlays. Set the starting count to a known positive *n*. Record loading draws, candidate cells predicted by F5, selected cells predicted by S-0212 claim 25 at that tick, and visible crate cells after a separate reveal. Expected: two ranged requests per attempted candidate, x then y, any overlay-placement draws on success, then one lifetime request. Count raw draws with S-0202 claim 4's rejection rules rather than assuming one raw draw per request. There are at most 1000 attempts per placement and no delayed retry for an exhausted placement. Each successful cell is empty of objects, uses the overlay for its land type, and placement itself explores no cells. With a count above 256 and enough room, the 257th placement takes no draw. Falsified by another candidate formula or search result, unexplained draws, a crate under an object, exploration from placement, or a 257th occupied place. Compare game-start counts with S-0412 claim 10, not the number of computer players.
- **CR2 — expiry (claim 4).** Use a short positive regeneration argument and record each crate's appearance, predicted lifetime and disappearance tick. With crates on, each successful replacement appears in its predecessor's expiry tick, with draws predicted by claim 3. Compare simultaneous expiries with the crate-place order established by earlier trigger placements and freed places. Block all legal replacement destinations in a control: expiry lowers the count rather than retrying next tick. With crates off, trigger crates remain beyond twice the regeneration duration. In a named zero-regeneration control with one crate and enough free cells, each tick has exactly one replacement draw group, not repeated same-tick expiries. A premature/late expiry, wrong replacement draw order, deferred retry, repeated zero-lifetime expiry in one update or expiry with crates off falsifies the corresponding part.
- **CR3 — death crates (claim 5).** Destroy a crate-carrying vehicle and buildings configured to leave money or random crates, with crates off. Use S-0312 claims 14 and 20's qualifying destruction fixtures. Collect each resulting crate with a house that cannot receive the base-unit override. Record credits, effect and stream state. Expected: vehicle and non-money building crates take the weighted choice request, while money building crates skip it and take the money request. Any different contents or draw signature falsifies the claim. Leave a second copy uncollected with crates on and test expiry as CR2.
- **CR4 — pickup timing and movers (claim 6).** Put a money crate one step ahead of drive, ship, hover, walking-vehicle and infantry movers; separately land a jumpjet on a crate. Record position and credits each tick. Ground effects precede arrival at the centre; jumpjet effects occur on landing. For a successful unit crate, a ground mover must not commit that step, but a jumpjet's landing must remain completed. Teleport a stock chrono miner onto a fixed money crate, with an existing base preventing the forced-base-unit override, as in S-0207 claim 20's landing-cell fixture. Expected: placement and cell-arrival effects precede pickup on the jump tick. Repeat with a successful unit crate: pickup must not reverse the completed landing. Named type data may give infantry and aircraft the teleporting locomotor to test the same rule regardless of kind. Separately test superweapon chronoshift arrival, tunnelling emergence, flying-locomotor aircraft landing, parachute/drop arrival and unloading onto a crate cell where legal; none alone gives a pickup. For chronoshift observe placement, arrival effects and the end of the arrival hold (S-0513 claims 37–40) before ordering a normal ground step. Compare normal ground entry afterwards. An effect solely on an excluded arrival, no effect on an eligible entry, or arrival before the ground pickup falsifies its row. If ordinary placement cannot create an object directly on an existing crate, that creation-only exclusion remains a negative-call-site inference, not a completed measurement.
- **CR5 — gates (claim 7).** Add a passive-country test house, distinct from the bystander, and order or trigger its unit across a crate: no effect and the crate stays. Compare a non-crate overlay. A tagged collector on a crate-trigger overlay raises event 49 with itself as the causing object. Configure that event to kill the collector: the crate remains, with no choice/effect draws and no event-50 mark. In a control with no killing action, read-only recording must show the any-crate mark being set. A named test overlay with the crate-trigger flag off still gives an effect but no event 49 or mark. Record event outcomes, the mark, overlay survival and stream state. Any contrary result falsifies the corresponding gate. Separately record event-50 trigger firings and their timing relative to polls, with control triggers distinguishing a poll from a matching raise; those observations investigate the open interaction, and no firing is predicted from the mark alone.
- **CR6 — choice and draw precedence (claims 8, 11, 25, 27).** Use two nonzero weights and 200 random-content pickups, recording the stream state. Repeat with a zero entry and a negative entry between positive entries, keeping a positive non-overflowing total. Expected: neither non-positive entry is selected, and the negative entry reduces later running totals as F1 predicts. Predict each choice from its ranged value and F1, not frequency alone. In separate fixed-content runs test every number 0 … 18 and random codes 19, 20 and 255. Enable replacement for one run: its draws precede effect draws and the final animation draws. In a tagged control add a trigger that consumes a known ranged request: it precedes choice. Give visible, non-damaging animations to kinds 13, 15 and 18: only those animations occur; kind 6 gives money with the money argument and animation. Disable crates in controls. Wrong predicted effects, extra/missing requests or reversed draw groups falsify the relevant claim. Predict the final animation at terrain-centre height plus 200 leptons, including on a slope or bridge cell; a deck lift falsifies that height rule.
- **CR7 — overrides and conversions (claims 9, 10).** Keep the house undefeated during the test. Compare no counted building, available credits above 1500, no base-unit type in existence and bases on, against controls changing exactly one condition, including credits exactly 1500 and a base unit waiting in production. Expected: only the qualifying case forces the base unit, even from fixed money contents. Repeat with an insignificant/not-scored building and with a building that counts as a vehicle: those do not block the override. For each row of claim 10, compare a qualifying collector with an eligible control, preventing other conversions. Use 50 versus 51 counted vehicles and 100 versus 101 counted infantry, including a ship, passenger and production item where relevant. Include buildings that count as vehicles and excluded type flags. Collect fixed unit and squad crates on beach and water. For the final water gate compare enabled/disabled water flags. Record effect, credits and draws: a failed eligibility test yields money with its argument, no extra choice request. An unchanged effect, wrong threshold, or wrong argument falsifies the corresponding row. The unarranged nominal-survivor subcase is listed in Open questions.
- **CR8 — money and local feedback (claims 12, 26).** Give the money argument an invented fractional positive value and then a negative value within F2's conversion domain. For 100 pickups of each, record exact credit changes and the ranged values. Expected: each delta equals R(trunc(*A*), trunc(*A*) + 900), not merely a sampled minimum/maximum. Observe local and nonlocal collectors receiving each sound-producing power-up, with an allied local object inside an enemy collector's upgrade radius. Expected: crate sounds only for the local collector, upgrade announcements when any local object is boosted, and the nuclear icon as CR15 specifies. An amount mismatch, nonlocal collector sound, missing eligible announcement or scenario draw caused solely by feedback falsifies that part. Use audio/video recording for presentation; do not infer a missing sound from a muted run.
- **CR9 — unit choice and placement (claim 13).** Use a refinery and no harvester, then a harvester present, a harvester in production, and a refinery only at list position 2. Expected: the first ownable harvester in the first case, no forced harvester in the controls. A named forced unit type overrides both harvester and base-unit choices. For random choice, record and replay acceptance of every type request against eligibility and the bases setting. Place a crate in a cell impassable for the awarded type but reachable by its collector, then make every fallback cell impassable: expected nearby placement in the first, deletion and money in the second. In a separate control, let a passive-country vehicle stand on the crate without collecting it and make that house allied to the collecting house. Collect with a ground mover from outside the cell. Expected: the stationary ally refuses direct placement even on traversable land; the fallback must qualify and then pass the awarded unit's entry test. Record the collector's actual listing/reservations rather than counting its intended arrival as occupancy. Repeat with a passive enemy infantry unit on the crate and an awarded crusher, choosing a fixture whose entry-table result is clear. Expected: direct placement may use that occupied cell, whereas the nearby search's empty-record test would refuse it; a blanket empty-cell direct test is falsified by success there. The existing S-0209 entry-table scenarios own the detailed special-occupant predicates. In both controls record positions, occupancy and placement outcome read-only at the pickup tick; unexplained success on a non-clear entry outcome or refusal on a clear one falsifies the direct-test contract. Distinct invented unit and money arguments distinguish the fallback's unit argument. Record created type, facing, cell, first-placement height, collector step, credits, animation and stream state. Expected initial facing is 0 facing units; another initial facing falsifies that rule. Unexpected types, an accepted ineligible draw, a skipped creation request, a ground collector committing the blocked step, a unit animation on success, or fallback credits based on the money argument falsify the relevant part. Compare computer collectors separately: the original refuses a random base unit as stated in claim 13, while our AI-house departure is tested on our engine, not against the oracle.
- **CR10 — heal all (claim 14).** Damage owned infantry, vehicles, aircraft and a building at widely separated positions, an enemy control, and three owned passengers before loading one into an ordinary closed transport, one into an open-topped transport and one into a garrisonable building. Collect a fixed heal-all crate and unload or evict all three passengers afterwards. Record health before/after. Expected: ordinary on-map owned objects and the open-topped passenger restored to strength, the enemy, closed-transport passenger and garrison occupant unchanged. Add named test data for an immune bridge repair hut and an undemolishable building: resolve those hits by S-0302 claims 7 and 8 rather than expecting an unconditional reset. A different owner/range filter, healing of the closed-transport passenger or garrison occupant, failure to heal the open-topped passenger, or health change contrary to those hit rules falsifies the claim. Other unusual off-map states remain open.
- **CR11 — cloak and area boundary (claim 15, F4).** Put own and enemy infantry, vehicles, aircraft and buildings at predicted distances just below, exactly at and above ρ, including different heights and a multi-cell building whose reference point distinguishes it from its foundation corner. Include a loaded passenger and terrain object. Collect cloak, then observe cloaking after normal cloak delays. Expected: the own cloak flag applies only to on-map actor kinds with integer distance strictly below ρ, independent of owner. A boundary object, passenger or terrain object cloaking, or a qualifying object failing to cloak for reasons other than S-0516 claims 3–8's normal restrictions, falsifies that part. A fractional radius in named data tests F3's conversion.
- **CR12 — explosion and napalm (claims 16, 17).** Use a durable collector, nearby friendly and enemy targets, and resource cells; record health, deaths, animations and stream state. Predict all five explosion points from the radius and raw draws, and the area hits using S-0303 claims 3–17; the collector's direct hit uses S-0302 claims 8 and 9 with defences ignored. Expected: exactly five blast requests, each blast's death draws preceding the next point's draws, and non-water damage-animation selection even for a water fixture allowed by its water flag. Napalm's first animation is at F6's midpoint, preceding the direct hit and area damage at that midpoint. Sloped/bridge fixtures distinguish terrain-centre height from deck height. Unexplained health loss, wrong points, house kill credit, splash-only selection on water, or wrong draw precedence falsifies the corresponding part. Predict resource reduction with S-0303 claim 17 rather than demanding that every blast destroy resources. Swap two visibly distinct, non-damaging animation types at list positions 0 and 1 in named test data: napalm's initial animation must follow position 0, not the former type; a fixed former type falsifies position-based selection. In a lethal-direct-hit variant, attach a destroyed-by-anything trigger to the collector with a known draw-consuming action. Expected: its draws precede the first explosion-point request, and all five blasts still occur. For napalm, give the initial animation a known immediate start draw: it precedes the lethal-hit trigger's draw, which precedes area-damage death draws. Reversed groups or suppressed area effects on collector death falsify the corresponding precedence. The exact influence of a gameplay-changing initial animation on the direct hit remains `unmeasured`: this scenario isolates draws, not a discriminating health/state change.
- **CR13 — darkness and reveal (claim 18).** Explore a remote area, keep another remote area unexplored, and place current sight sources away from both. Collect darkness and reveal separately; record explored cells for the collector and an enemy. Expected: S-0409 claim 15's whole-map result for the collecting house only, subject to its ongoing sight rules. A radius-limited result or another house's exploration changing solely from this effect falsifies the claim.
- **CR14 — armour, firepower and speed (claims 19, 20).** Use own and enemy actors just inside, exactly at and outside ρ, a building, an aircraft, an unarmed actor, and a loaded passenger whose last position is inside ρ. Use invented non-unit factors. Test armour with known incoming hits, firepower with outgoing hits, and speed with timed equal-distance movement, applying the owning specs' rounding. Unload the passenger for its tests. Expected: eligible factors initially equal to 1.0 become *A*; non-unit factors remain unchanged on another crate, and a factor of 1.0 remains eligible. Armour/firepower include buildings, aircraft and unarmed neighbours; speed excludes buildings and aircraft. Enemy objects and the passenger are boosted when otherwise eligible. Boundary mismatches, stacking, an owner filter or speed changes for excluded kinds falsify the relevant rule. Our engine separately tests the off state of the off-map bonus switch.
- **CR15 — nuclear grant (claim 21).** Use a fixed nuclear crate without a prior grant. Record grant, sidebar, charge/readiness and consumption after firing against S-0512 claim 12. With named test data, make the first nuke-action type different from the nuclear-kind type, then remove all nuclear-kind types. Expected: the first action-matching type is granted only when a nuclear-kind type exists; the absent-kind case does nothing. The missing-action unsafe case is tested only for our engine's warning/no-grant departure. A repeatable grant, wrong type, or grant in the absent-kind case falsifies its part.
- **CR16 — veteran (claim 22).** Use trainable own/enemy actors of each kind, an untrainable actor, an elite actor, a passenger, and actors at/inside/outside the radius. Named arguments 0, a negative value, 0.5, 1.0, 1.5 and 2.0 distinguish zero and ceiling step counts. Give some actors partial progress before pickup. Expected: F7's exact resulting levels, with partial progress reset, no owner restriction, no off-map or untrainable promotion, and no change at elite. Read ranks and progress read-only, or infer progress from subsequent controlled kills. Any other level, retained partial progress or wrong boundary falsifies the corresponding part.
- **CR17 — gas (claim 23).** Use a durable ground collector and infantry spread over the nine cell centres, with resources and a bridge/slope control. Predict health changes using nine sequential area damages at terrain-centre heights, centre first and neighbour directions 0 … 7. Record stream state and destruction effects to distinguish death order where areas overlap. Resource quantities must not be reduced by these detonations. A different count/order, deck-height centres, attributed attacker/house or gas-caused resource reduction falsifies the relevant part. If individual detonations cannot be separated in stock data, use named small-spread gas-warhead data and durable targets to isolate the centres. For the absent-name default, use a named rules copy omitting the gas warhead declaration, its section and all other references to that name. Compare a second copy explicitly defining that named warhead with every field at its documented absent-key default. Predict both sets of health changes under S-0302 and S-0303; identical damage and draw signatures are expected. A failure to execute the absent-name effect or a different damage/draw result falsifies the fallback.
- **CR18 — resource (claim 24).** Use a flat germinable enclosure, plus existing cells of the selected and another resource, blocked cells, a slope, and cells near the stage cap. Record stream values and resource cells immediately around pickup, separating ordinary growth turns. Predict the chosen type, initial-cell stage request and 10 … 20 further requests with S-0312 claim 22's nearby-point formula, including the snapped centres. Expected: 11 … 21 stage-increase requests, not necessarily that many changed cells; duplicates, blocked cells and stage caps can reduce the changes. Position 1 always maps to 0. Swap distinguishable resource definitions between list positions 0 and 1 in named data and repeat: the replacement must still follow position 0, not its former type identity. Selecting the former type or position 1 falsifies the position rule. Existing resource grows only under claim 24's conditions; germination takes its own draws. With the map growth switch off, existing cells do not grow, but germinable cells may still appear. A wrong selected type, point, request count or stage falsifies that part. Observe subsequent growth/spread timing as an exploratory check only, not as proof of exact list membership. Use named sprite files with only one frame, then with a different frame count, leaving the resource's image number unchanged. Record stages read-only rather than relying on a possibly absent picture. Expected: the stage-picture count remains 12 and the cap remains 11, not the sprite frame count minus one. Repeat at stages 10, 11 and above 11: respectively one increase to 11, no increase and no change of the above-cap stage. Any sprite-dependent cap or lowering of an existing above-cap stage falsifies the claim. The count-at-most-one guard has no legal fixture established: image-number imports always set the count to 12, so a one-frame sprite is not a test of that guard. Snapping can put a centre farther than 768 leptons from *P*, so that distance alone is not a failure.
- **CR19 — import controls (claim 28, Import mapping).** In a fresh process, use named base-rules and mode-file copies plus a purpose-made map as three layers. Give one effect in the base layer an invented positive weight, a visible harmless animation, an enabled water flag and an invented argument. In variant A the mode has a power-up section containing only another effect, and the map omits that section. In B the mode omits the section too. In C use A's mode but let the map restate the tested effect with only weight and animation. Keep a positive total weight in every variant. Collect fixed-content crates to observe the tested effect even when its weight is zero, and random-content crates to check weighted choice against the recorded stream. Expected: A has weight zero and no animation, retaining the base water flag and argument; B keeps all four base values; C uses the map weight/animation and the base water flag/argument. In separate A controls, supply an art section bearing the reset's reserved bare no-type name, then also attempt registration by an animation-list entry and by an ordinary creating animation reference (S-0004 claim 13). Repeat with mixed letter case. None of these controls may make the reset produce an animation. As a positive control, use an ordinary non-reserved name for the same harmless art and explicitly supply that animation in the tested effect's entry: it must appear. Record tick-labelled captures at pickup and the stream state, so missing art cannot masquerade as reserved-name rejection. A visible reset animation in any reserved-name control falsifies claim 28; absence in the positive control invalidates the fixture. A three-layer final-text-only importer incorrectly gives A the base weight/animation. Wrong draw-predicted selections, animation, water eligibility or argument-derived effect falsify the corresponding rule. Include a section containing only empty entries: it counts as absent, not as A's reset. Use percent-bearing arguments and a radius of exactly −1 to distinguish their parse rules, and compare ignored skirmish fields while holding effective fields fixed. These are proposed tests, not completed measurements.
- **CR20 — stored contents (claim 8).** With crates off, create isolated trigger crates using number-slot values 19, 20, 21, 255 and 276; separately record random game-start, replacement and random death crates before collection, with crates on only where required. Use a read-only cell recorder, or a save/export that preserves overlay data; do not infer the stored value solely from a random pickup. Expected stored values for the five triggers are 19, 255, 21, 255 and 20 respectively; ordinary random placements store 255. Record pickup effects and stream state afterwards: every one uses claim 8's weighted choice. Any other stored value or a fixed-content pickup falsifies that part. Number-slot 20 and a value reducing to 20 distinguish the explicit random request from an explicitly stored random code.

## Open questions

- Event 50's interaction between polled and code-matched evaluation (S-0011 claims 6 and 8). Claim 7 establishes only its associated scenario mark; CR5 must not infer poll satisfaction or firing from that mark alone.
- Heal-all eligibility in unusual off-map states beyond ordinary closed transports, buildings and open-topped transports (claim 14). CR10 tests those ordinary cases; the broader state coverage remains `unmeasured` because no exhaustive set of reachable states has been established, confidence low.
- Exact nominal-survivor crate-count behaviour beyond the common cases (claims 9 and 10). The exclusion is traced, but a reliable stock fixture that produces such survivors has not been identified (S-0414 claim 4); that subcase is `unmeasured`, confidence medium.
- The final placement height of a fallback unit on slopes or bridges is not specified: only the first placement's terrain-centre height is established (claim 13). This subcase is `unmeasured` pending a trace of placement's height adjustment, confidence low.
- The read-only free-unit flag of the crate rules shows no observable effect. If a measurement finds one, this spec needs a revision.
