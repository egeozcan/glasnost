---
id: S-0315
title: Suppression and panic (infantry fear, prone and panic in combat; parasite suppression)
topic: combat
revision: 5
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-13
measurements: []
requests: [Q-0087]
supersedes: []
---

# S-0315: Suppression and panic (infantry fear, prone and panic in combat; parasite suppression)

## Summary

The combat view of infantry fear: what raises an infantry unit's fear level, and what the prone
state and the panic sequence that fear selects change in combat. Most of the rules are owned by
other specs, which this spec cites by claim; it owns the complete list of fear triggers, the one
trigger no other spec covers (a fraidycat that runs out of ammunition), the summary of effects, and
the separate *suppression* of parasites: damage or healing done to a unit that carries a parasite
can make the parasite die when it leaves.

## Terms

Shared terms used here are defined in the glossary: *infantry*, *vehicle*, *house*, *behaviour*,
*tick*, *known attacker*, *scatter*, *scatter request*, *weapon*, *warhead*, *parasite, victim*,
*human-controlled*, *computer-controlled*, *AI house*, *quirk switch*, *importer warning*,
*recorder*. In addition:

- **Fear level, prone state, fraidycat, fearless, cyborg, can crawl.** As S-0107 (Terms): the fear
  level is an integer from 0 to 300 per infantry unit; the prone state means lying down; the other
  four are infantry type properties (S-0107 and S-0208; Import mapping lists the flags that this
  spec reads: fraidycat, fearless, cyborg and can crawl). A unit whose type cannot crawl never lies down
  (S-0107, claims 6 and 15); the only such unit that can be prone is a crippled cyborg (S-0208,
  Terms and claim 8; S-0302, claim 15).
- **Panicking.** An infantry unit whose fear level is above 199, so that walking shows the *panic*
  sequence (S-0107, claim 7).
- **Ammunition.** An object's count of shots left, which S-0301 owns; a type with no ammunition
  limit never runs out.
- **Delivered amount *d*.** The amount a hit delivers to the host, S-0302's delivered damage
  *d*₀: taken before the host's defences, before the versus value, before the distance from the
  detonation point and before an inert map reduce it (S-0302, claims 5 and 6).
- **Healing hit.** A hit whose delivered amount is negative (S-0302, claim 6), whether or not it
  changes the target's health.
- **Host.** The victim of a parasite (glossary *parasite, victim*) while the parasite is inside or
  attached to it.
- **Suppression threshold.** An integer field of a parasite's type (Import mapping).
- **Suppression timer.** A countdown in ticks per parasite; the parasite is *suppressed* while it
  runs.
- **Sonic warhead.** A warhead whose type has the sonic property (S-0214, claim 6; Import
  mapping). A hit by a sonic warhead is a *sonic hit*, whatever its delivered amount. No other
  property of the warhead or of the weapon matters here; in particular the weapon's own sonic-wave
  flag (S-0302, Terms) is a different property.
- **Naval type.** A vehicle type with the naval flag (S-0004, claim 11; Import mapping). A
  *naval parasite* is a parasite of a naval type.
- **Leaving the host.** Any way a parasite stops being attached to a host while both still
  exist. The known ways are: the host takes a sonic hit (of any delivered amount), a healing hit
  whose warhead is not sonic, or a repair step; it unloads or undeploys; it is moved by a
  superweapon; it has its locomotion replaced by an effect (only a naval parasite leaves this
  way); and the parasite's own release rules (left to S-0614, parasites, planned). A hit that
  claim 6's vehicle-factory exception covers is not one of these ways.

## Behaviour

### Fear

1. **What changes the fear level.** Apart from the team scripts of the original's computer player
   (Edge cases), which our engine does not have, an infantry unit's fear level is 0 when it is
   created and changes only through these rules:

   | Trigger | Change | Owner |
   |---|---|---|
   | the unit survives damage | rises, or is set, as S-0107's table gives (with or without a known attacker, fearless or not, fraidycat or not) | S-0107, claim 16 |
   | every tick, a fear level above 0 | falls by 1, except for a fearless type | S-0107, claim 15 |
   | a fraidycat fires its last shot, under the conditions of claim 2 | set to 300 | this spec, claim 2 |

   Nothing else raises fear: not a nearby explosion that does no damage to the unit, not the
   death of a neighbour, not the sight of an enemy. The fearless veterancy ability (S-0311, claim 3)
   makes damage raise the fear level only in the fraidycat case of S-0107 claim 16, and does not
   stop the decay. Vehicles, aircraft and buildings have no fear.
   - Backing: `measurable` (scenario P1 for the absence of other triggers; S-0107 scenarios M6
     and M15 for the damage and decay rows; scenario P2 for the third row)
   - Confidence: high (the triggers); medium (that no trigger other than the original's team
     scripts exists)

2. **A fraidycat that runs out of ammunition panics.** When an infantry unit of a fraidycat type
   fires a shot that leaves it with no ammunition (S-0301), its fear level becomes 300 at once, and,
   when its current behaviour is *attack* or *hunt* (S-0215), *guard* is queued as its next
   behaviour (queued, not made current at once, in the sense of S-0215; as any queue request it
   replaces a behaviour already queued, S-0215 claim 7). A unit in any other behaviour, *guard*
   included, has its fear set and nothing queued. A unit whose type has no ammunition limit never
   triggers this. Neither does a shot fired from inside a transport, nor a shot of a garrisoned
   building: the occupants of a garrisoned building do not fire themselves, the building fires with
   an occupant's weapon (S-0301, S-0313), so no occupant's ammunition is spent and no occupant's
   shot is made. The fearless flag and the fearless ability do not prevent the panic
   (`quirks.fraidycat_ammo_panic_ignores_fearless`, default on; see Edge cases); a fraidycat whose
   type is fearless then keeps the level 300, since its fear never decays (S-0107, claim 15), while
   one that is fearless only through the ability decays from 300 like any other unit. From then on
   S-0107's rules apply: it decays from 300, walks with the *panic* sequence while above 199, and,
   while it has no movement and no destination, scatters while above 50 (S-0107, claim 15, the flee
   rule, which does not depend on the owner). Infantry never regain ammunition over time (S-0301,
   claim 11), so the unit stays out of ammunition.
   - Backing: `measurable` (scenario P2: parts a to c for the fear, part d for the queued *guard*
     read from the behaviour slots with the recorder, as S-0215 scenario B1 reads them, part e for
     the garrison)
   - Confidence: medium (that the trigger is the shot itself, not a later refused attempt to fire,
     so the panic starts at the shot that empties the ammunition; the queued *guard* and that it
     is queued rather than started; the exemption of shots from inside a transport, which rests on
     such passengers being off the map, and of occupants of a garrisoned building, which rests on
     the building being the firer; that the fearless flag and ability do not prevent it: the
     original makes no exception for them here, which scenario P2 part b tests)

### What the prone state and panic change

3. **Effects of the prone state in combat.** While an infantry unit is prone:
   - damage it takes from a positive hit is changed by the warhead's prone factor (S-0302, claim
     5, which owns the rule and its lower bound),
   - it fires with the prone firing sequences and their fire frames (S-0107, claims 11 and 12),
   - it moves at the prone speed of S-0208 claim 8: slower than walking when its type can crawl,
     and faster when it cannot, which is possible only for a crippled cyborg
     (`quirks.prone_noncrawler_speedup`, S-0208's switch),
   - a prone unit of a human-controlled owner asks (not forced) for *get-up* only when
     a non-empty destination is reassigned that is already its current destination, unless
     its type is a fraidycat or a cyborg (S-0107, claim 15). The same cell or the same object
     counts; occupying the destination cell alone does not. A first or different destination,
     clearing a destination, and merely queuing a waypoint do not trigger this request.
     It happens at destination assignment, whether by an applied order or later by a
     behaviour, not at the start of locomotion. No order family guarantees it: move,
     attack-move, attack pursuit, area guard, scatter, enter and capture all use the same
     rule when they assign a destination; S-0107 claim 15 gives the Q-0087 order table.
     The request can be refused by S-0107 claim 6 and is not saved or retried by movement.
     If accepted, prone protection and prone firing end when *get-up* starts, not when its
     image frames finish. Thus a first move can begin crawling, rather than getting up
     first. Fear-driven get-up is a separate rule (S-0107, claim 15). Our AI houses follow
     the human-controlled rule; neutral and special houses make no destination-driven
     request (glossary *AI house*),
   - a cyborg that is prone dies without being crippled first (S-0302, claim 15) and without a death
     sequence (S-0107, claim 17).
   No other effect of the prone state on combat was found: it does not change the unit's range,
   its rate of fire, whether others can target it, or the chance that a shot hits it.
   - Backing: `measurable` (through the owners' scenarios; scenario P3 for the absence of other
     effects; scenario P5 and S-0107 scenario M4 for destination-driven get-up),
     `unmeasured` (the fraidycat exception to destination-driven get-up: such a type never
     lies down in stock play, as S-0107 claim 15 explains)
   - Confidence: high for the cited effects and the repeated-destination test; medium for
     the fraidycat exception and the absence of other effects

4. **Effects of panic.** A panicking unit walks with the *panic* sequence (S-0107, claim 7) at its
   ordinary speed (S-0208, claim 10). Panic changes nothing else in combat: the unit's weapon,
   range, rate of fire and the damage it takes are those of the prone or standing state it is in.
   A fraidycat scatters as S-0107 gives (claim 15, the flee rule; S-0208, claims 11 to 15).
   - Backing: `measurable` (S-0208 scenario W5, part c; scenario P3)
   - Confidence: medium

5. **Lying down while moving.** By S-0107 (claim 15, the lie-down rule) an infantry unit of a
   human-controlled house lies down only when it has no movement destination and no movement in
   progress, while one of a computer-controlled house in the original also lies down in the middle
   of a move. In our engine an *AI house* follows the rule for human-controlled houses (glossary *AI
   house*): its infantry do not lie down while moving. The original's behaviour is recorded for
   the designers of our AI only.
   - Backing: `measurable` (scenario P6 below)
   - Confidence: high (the original's rule, as S-0107)

### Parasite suppression

6. **Damage to a host suppresses its parasite.** When a host takes a hit whose warhead is not
   sonic, whose delivered amount *d* is at least 0 and greater than the suppression threshold *T*
   of its parasite's type, and whose attacker is not the parasite itself, the parasite's
   suppression timer is set to 2*d* − *T* ticks (wrapped as Formulas gives), replacing any time
   left, longer or shorter. The
   parasite counts as suppressed at a later tick only when fewer than 2*d* − *T* ticks have passed
   since the tick of the hit; when exactly that many have passed it is no longer suppressed.
   Since *d* is the delivered amount, the host's defences, the versus value and the distance from
   the detonation point do not shorten the time: a host whose armour halves the damage, a hit that
   the versus value reduces to 0, and a host at the edge of a detonation are all suppressed as if
   the hit had landed in full. Nor does it matter whether the hit changes the host's health at all:
   a hit that a row of S-0302 claim 8 stops (for example an attacker allied with the host and a
   warhead that does not affect allies, or a host that is temporarily invulnerable) still counts
   here, for suppression and equally for claim 7's sonic and healing rules. The one exception is
   a host that is a vehicle standing inside the vehicle factory it is in contact with: a hit on it
   counts for nothing here either. Any other hit leaves the timer as it was: a hit with *d* ≤ *T*,
   the parasite's own hits, and every sonic hit, whatever its delivered amount (a sonic hit makes
   the parasite leave instead, claim 7). A healing hit whose warhead is not sonic and a repair step
   on the host (S-0314, claim 3, a step that goes ahead because its cost, possibly 0, can be paid)
   make the parasite leave (claim 7); no suppression time is observable in those two cases.
   - Backing: `measurable` (scenario P4: parts c, c2, d, i, j for the formula and the conditions,
     k for the versus value, k2 for the distance, k3 for the defences, m for the hits that S-0302
     claim 8 stops); `unmeasured` (the vehicle-factory exception: a parasite cannot practically be
     attached to a vehicle that has not yet left its factory, so the case cannot be set up)
   - Confidence: medium (the formula and the conditions, and that stopped hits still count); low
     (the exact boundary tick of the suppression time, and the vehicle-factory exception)

7. **A suppressed parasite dies when it leaves.** A parasite that leaves its host while it is
   suppressed is removed from the game at once instead of coming out: it is not killed by damage,
   so it leaves no death effects, gives no kill credit, experience or score to anyone (S-0311),
   and simply disappears. When it leaves without being suppressed (the timer has run out, or it
   was never started), it comes out on a cell next to the host (where is left to S-0614,
   parasites, planned); when it finds no cell where it can stand, it is removed in the same silent
   way. A hit on a host that is a vehicle standing inside the vehicle factory it is in contact
   with counts for nothing in this claim either (claim 6): it makes no parasite leave. Two kinds
   of event decide the outcome differently:
   - **A sonic hit** on the host makes the parasite leave, and its outcome depends only on earlier
     hits: the parasite comes out alive unless an earlier hit had suppressed it. This holds for
     any delivered amount of the sonic hit, a negative one included: a sonic healing hit frees an
     unsuppressed parasite alive, and the healing rule below does not apply to it.
   - **A healing hit whose warhead is not sonic, or a repair step,** on the host makes a parasite
     that is not naval leave and be removed silently, whether or not it was suppressed and however
     long ago the host was last damaged. A healing hit counts whether or not it changes the host's
     health: it removes the parasite of a host at full health, and of a host that a healing
     detonation reaches at 8 leptons or more from its point, which heals the host by nothing
     (S-0303, claim 13; S-0302, claim 6).

   A *naval* parasite is not affected by suppression: it comes out alive after any earlier damage,
   a healing hit and a repair step included, and is removed silently only when it finds no cell
   where it can stand. The cell it tries may also differ from that of a parasite that is not
   naval (left to S-0614).
   - Backing: `measurable` (scenario P4: parts a and e for healing, repair and the silent removal,
     b, c and l for the sonic hit, f for the naval case, g for no free cell, h for healing that
     changes no health)
   - Confidence: medium (removal on leaving while suppressed, on healing and on repairing; that
     the removal is silent; that a sonic hit decides by earlier hits whatever its own
     amount; that healing counts without a health change); low (the naval case, the full list of ways to
     leave, what happens when the host itself is destroyed while it carries a parasite, and the
     removal when no cell is found, for either kind of parasite)

## Formulas

> suppression time *t* = W(2*d* − *T*) ticks, for a hit whose warhead is not sonic, with
> delivered amount *d* ≥ 0 and *d* > *T*. *d* and *T* are 32-bit signed integers, and W(*x*) is
> the 32-bit wrap of S-0205 claim 1: *x* reduced modulo 2³² into the range −2³¹ … 2³¹ − 1. So *t*
> differs from 2*d* − *T* whenever 2*d* − *T* leaves that range, for example for a large *d* or,
> with *T* < 0, a smaller one. Exact integer arithmetic apart from that wrap, so no rounding
> applies; no floating point is involved. When *t* ≤ 0 the parasite is not suppressed: its timer
> counts as already run out.

Fear arithmetic is S-0107's (Formulas F5). Prone speed is S-0208's (Formulas F5).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Fear set when a fraidycat runs out of ammunition | 300 | fear points | Claim 2 |
| Panic threshold | above 199 | fear points | S-0107 |
| Suppression time factor | 2 | ticks per damage point | Claim 6 |
| Suppression threshold, absent-key default | 0 | damage points | Claim 6 |

## State and ordering

Each event on a host with a parasite, other than the host's own destruction (Open questions),
matches exactly one of the parasite rows. A hit on a host that is a vehicle standing inside the
vehicle factory it is in contact with matches only the first parasite row; in every other row a
hit means a hit on a host that is not in that state.

| State | Event | Result |
|---|---|---|
| fraidycat with ammunition, on the map | fires its last shot | fear 300; from *attack* or *hunt*, *guard* is queued (replacing a queued behaviour); from any other behaviour nothing is queued |
| parasite attached; host a vehicle standing inside the vehicle factory it is in contact with | host takes any hit (sonic, healing or other) | stays attached; timer unchanged |
| parasite attached | host takes a hit whose warhead is not sonic, with *d* ≥ 0, *d* > *T*, not from the parasite itself | stays attached; suppressed for 2*d* − *T* ticks (Formulas), replacing any time left |
| parasite attached | host takes a hit whose warhead is not sonic, with *d* ≥ 0, and *d* ≤ *T* or from the parasite itself | stays attached; timer unchanged |
| parasite (not naval) attached, suppressed or not | host takes a healing hit whose warhead is not sonic, or a repair step | removed silently |
| parasite (not naval) attached, suppressed | host takes a sonic hit, or the parasite leaves by any way other than the row above | removed silently |
| parasite (not naval) attached, not suppressed | host takes a sonic hit (any *d*), or the parasite leaves by any way other than a healing hit whose warhead is not sonic or a repair step; a free cell exists | comes out alive (where is left to S-0614) |
| naval parasite attached, suppressed or not | host takes a sonic hit (any *d*), a healing hit whose warhead is not sonic or a repair step, or the parasite leaves by any other way; a free cell exists | comes out alive (where is left to S-0614) |
| either of the two rows above | the same event, no free cell | removed silently |

Ordering within one hit on a host, as observable consequences: a sonic hit, whatever its own
delivered amount, frees alive (when a free cell exists) a parasite that earlier hits had not
suppressed and removes one that they had suppressed, unless the parasite is naval; its own amount
never suppresses the parasite and never triggers the healing rule. A healing hit whose warhead is
not sonic, or a repair step, on a host whose parasite is not naval always removes the parasite
silently. None of this applies to a hit on a host that is a vehicle standing inside the vehicle
factory it is in contact with, which changes nothing for the parasite (claim 6). Whether the
parasite leaves before or after the hit's own health change is not claimed; it matters only when the hit destroys the host, which is part of the open question on hosts
destroyed while they carry a parasite.

## Edge cases and quirks

- **The original's computer team system** can set the fear level of its infantry to 300 or back to 0
  by actions of its team scripts (fearless types ignore both).
  That system is out of scope; our AI houses have no such action and act through orders only. The
  behaviour is recorded for the designers of our AI.
- **Computer-owned engineers hunt when hurt.** In the original, an infantry unit of an engineer
  type (Import mapping) that belongs to a computer-controlled house and is in *guard* or *area guard*
  switches to *hunt* when it survives damage. This is recorded for the designers of our AI only: an
  *AI house*'s engineers follow the rule for human-controlled houses and keep their behaviour.
- **A suppression threshold below 0** makes every non-negative hit whose warhead is not sonic
  suppress the parasite, for longer than twice the damage, except where 2*d* − *T* wraps
  (Formulas): the time is then shorter, and a time of 0 or less leaves the parasite unsuppressed.
  This is claim 6's formula applied to
  unusual data, not a separate behaviour, so it has no switch. Reproduced; the importer warns.
- **Healing near a host removes its parasite** (claim 7, outside claim 6's vehicle-factory
  exception): a healing detonation that reaches the host
  without healing it, because the host stands 8 leptons or more from its point, still removes a
  parasite that is not naval. This follows from claim 7's rule that the delivered amount, not the
  health change, decides, so it has no switch of its own.
- **A fearless fraidycat stays at fear 300 for good** after running out of ammunition (claim 2),
  because a fearless type's fear never decays (S-0107, claim 15). Quirk
  `quirks.fraidycat_ammo_panic_ignores_fearless` (default: on, the original). On: claim 2's
  trigger sets the fear level to 300 whether or not the unit is fearless by type or by ability. Off:
  claim 2's trigger does nothing to a unit that is fearless by type or by ability: its fear level
  stays as it was and no *guard* is queued for it, and every other unit is as before. S-0107's own
  switch for the same effect after a hit (`quirks.fraidycat_fear_ignores_fearless`) is
  separate; the two can be set independently.
- **Repeated destination required for get-up.** S-0107 owns
  `quirks.prone_get_up_repeated_destination` (default: on, the original). On: claim 3's
  destination-driven request requires the same current non-empty destination. Off: any
  non-empty destination assignment can request get-up, with the other owner, type and
  sequence conditions unchanged. This spec adds no separate switch.
- **The prone non-crawler speed-up** belongs to S-0208 (`quirks.prone_noncrawler_speedup`); this
  spec names no switch of its own.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, infantry type section, `Fraidycat`, `Fearless` | fraidycat, fearless (claims 1, 2; fraidycat also excludes destination-driven get-up in claim 3) | boolean | no (S-0107) |
| Rules, infantry type section, `Cyborg` | cyborg, including exclusion from destination-driven get-up (claim 3) | boolean | no (S-0107) |
| Art, infantry image section, `Sequence`; referenced sequence section, `Up` | get-up sequence that claim 3 requests | section name; integer triple and optional facing, as S-0107 | no section; zero image-frame count, so the request is refused (S-0107) |
| Rules, type section, `Ammo` | ammunition limit (claim 2) | integer | owned by S-0301 (no limit) |
| Rules, infantry type section, `Engineer` | engineer (Edge cases, the original's computer-only rule) | boolean | no |
| Rules, warhead section, `Sonic` | sonic warhead (Terms) | boolean | no |
| Rules, vehicle type section, `Naval` | naval type (Terms; claim 7) | boolean | no (S-0004) |
| Rules, type section, `SuppressionThreshold` | suppression threshold *T* of a parasite type (claim 6) | integer | 0 |
| Rules, warhead section, `ProneDamage` | prone factor (claim 3) | owned by S-0302 | S-0302 |
| Art, infantry image section, `Crawls` | can crawl (claim 3) | owned by S-0107 and S-0208 | yes |

## Measurement scenarios

All scenarios run under the measurement setup with named test files: a flat test map whose override
sections define the test types. Fear itself is not visible; it is read from its effects (the
*panic* sequence while walking, lying down, scattering), as S-0107's scenarios do. Any game speed.

**P1: no fear without damage (claim 1).** Own rookie infantry of a type that can crawl, not
fearless, standing in *guard*.
- *Part a (explosions and passers-by):* a friendly test unit force-fires at the ground 2 cells from
  them with a weapon whose warhead has a spread smaller than one cell, so the explosions reach none
  of them, and an enemy test vehicle that has no weapon drives past at 3 cells. *Observable:*
  whether any of them lies down. *Expected:* none does. Control: a hit that does 1 damage makes
  each hit unit lie down (S-0107, claim 16).
- *Part b (a neighbour's death):* an own infantry unit stands 2 cells from the group and is killed by
  a single shot whose warhead has a spread smaller than one cell, in view of the group. *Expected:*
  none of the group lies down.
- *Falsified if:* a unit lies down without being damaged. *Runs:* 3 per part, with different random
  seeds, and all must agree.

**P2: fraidycat out of ammunition (claim 2).** An infantry test type with the fraidycat flag, a
weapon and an ammunition limit of 2, in *guard* beside an enemy test vehicle in range, so that it
fires on its own (claim 2 does not depend on the owner). The test vehicle cannot hurt it.
- *Part a (fear):* *Observable:* after the second shot, whether the unit walks with the *panic*
  sequence and scatters (it has no ammunition left, so it cannot fire again). The recorder also
  reads the behaviour slots on every tick. *Expected:* the first two right after the second shot
  (not after a later refused attempt); the *panic* sequence ends about 100 ticks later (300 → 199)
  and the scattering after about 250 ticks (300 − 50); the slots show *guard* current and nothing
  queued throughout (the unit is in *guard*, so no behaviour is queued). Control: the same type with
  no ammunition limit never panics without being hurt.
- *Part b (fearless):* the same with a fearless fraidycat type, and again with a fraidycat that
  is fearless only through the veterancy ability. *Expected:* the fearless type panics after the
  last shot and still walks with the *panic* sequence 400 ticks later; the unit with only the
  ability panics too but no longer walks with the *panic* sequence 400 ticks later. Under
  `quirks.fraidycat_ammo_panic_ignores_fearless` off, neither panics and no *guard* is queued.
- *Part c (transport):* the same fraidycat fires its last shot from inside an open-topped transport
  (glossary *open-topped transport*), then gets out. *Expected:* no panic.
- *Falsified if* (parts a to c): no panic after the last shot, panic in the control or in part c, no
  panic in part b, or the wrong decay in part b (the fearless type no longer panicking at 400
  ticks, or the unit with only the ability still panicking then).
- *Part d (queued guard):* the same fraidycat type, human-controlled, is ordered to attack the
  test vehicle (so *attack* is current), and a second variant is placed in *hunt* by the test map
  with the vehicle in range. In a third variant a *move* to a nearby cell is ordered while the
  unit is firing, so that *move* is queued behind *attack*. *Observable:* the current and queued
  slots on every tick, from the recorder, and the unit's movement. *Expected:* at the end of the
  tick of the second shot the slots show *attack* (or *hunt*) current and *guard* queued, in the
  third variant too (*guard* replaces the queued *move*); *guard* becomes current, with nothing
  queued, on the first later tick on which the unit is ready to change (S-0215, claim 11), which
  is not before its firing and its scatter moves are over. *Falsified if:* the slots show *guard*
  current at the end of the tick of the shot (made current at once); or nothing is queued after the
  shot while *attack* or *hunt* is still current (behaviour unchanged); or *move* is still queued
  in the third variant, or the unit walks to the cell of that *move*; or *guard* never becomes
  current while the unit stands still with its fear at 50 or below.
- *Part e (garrison):* the fraidycat type, with the occupant flag set in the test file, occupies a
  garrisonable building that fires at the test vehicle for more than two shots, then leaves the
  building. *Expected:* no panic, inside or on leaving (the occupant's ammunition is not spent); once
  outside, the unit panics at its own second shot and not at its first. *Falsified if:* the unit
  panics inside, on leaving or at its first shot outside.
- *Runs:* 3 per part, with different random seeds; all must agree.

**P3: nothing else changes when prone or panicking (claims 3, 4).** Three variants of an own
infantry test unit, none of them a cyborg or a fraidycat, all given the same weapon and health high
enough that no part kills them, at the same range from the same test target of high health. Fear
decays by 1 per tick, and a hit with a known attacker raises a fear level below 100 only to 100,
while a hit on a fear level of 100 or more adds the increment of S-0107 F5 (S-0107, claim 16). So
each variant that needs fear is kept in its state by a *tickler*: a second enemy test unit whose
weapon hits the variant at a fixed interval from a known attacker; the tickler's hits land on ticks
that the firers' shots avoid (a run in which one coincides is repeated).
- *Variant A (standing control):* a type that cannot crawl, no tickler, fear 0 throughout.
- *Variant B (prone):* the same type except that it can crawl (art data only), with a tickler that
  does 1 damage every 20 ticks, so its fear stays at 80 or more. It must be seen lying down
  throughout; a run in which it gets up is invalid.
- *Variant C (panicking):* the type of variant A, with a tickler at an interval and damage that the
  test sets so that its fear stays above 199 (fear arithmetic is S-0107 F5). Its type cannot crawl
  and is not a cyborg, so it never lies down (S-0107, claims 6 and 15) and is never made prone by
  being crippled (S-0302, claim 15): it stays standing while it panics. *Checked* at the start and
  at the end of the measurement window by ordering it a short move, which must show the *panic*
  sequence and no lying down; a run that fails the check is invalid. It is not a fraidycat, so it
  does not scatter, and the window needs no movement.
- *Part a (dealing):* each variant fires at the test target 20 times. *Observable:* the range at
  which it starts firing, the ticks between shots, and the damage per shot dealt. *Expected:*
  equal in all three variants. *Falsified if:* any of the three differs between variants.
- *Part b (receiving):* an enemy firer with a perfectly accurate weapon fires 20 shots at each
  variant. *Observable:* the damage received per shot, read at the ticks of those shots. *Expected:*
  equal for A and C, and for B what S-0302's prone factor gives. *Falsified if:* any other
  difference.
- *Part c (targeting):* (1) An enemy unit is ordered to attack variant B, then variant A, from the
  same place. *Expected:* it fires at both. (2) An enemy unit in *guard* with both within range and
  equally distant, positions swapped between runs. *Expected:* its automatic choice follows S-0309
  alike for both, with no preference for either state. *Falsified if:* in (1) it refuses to fire
  at either variant; in (2) its choice follows the variant's state rather than its position.
- *Part d (being hit):* an enemy firer whose weapon is inaccurate, so that some shots miss, fires
  the same number of shots at A and at B (and C) in 40 runs. Its warhead has a prone factor of
  100 % and a versus value of 100 % against the variants' armour class, so every hit that reaches
  a variant lowers its health by at least 1 (S-0302, claims 5 and 6); shots that reduce health on
  the tick of a shot are counted as hits. *Expected:* the hit fraction is the same for each pair.
  *Falsified if:* the 99% confidence interval of the difference between two fractions excludes 0.
- *Runs:* parts a and b, 5 per variant, with different random seeds, and the damage numbers must be
  equal in every run; part c, 3 runs per case, all must agree; part d, as stated.

**P4: parasite suppression (claims 6, 7).** A parasite type that climbs into vehicles (a stock one,
with its suppression threshold set to 50 in the test map), attached to an own test vehicle with high
health. Every *sonic hit* below is made by a test weapon of an enemy test unit: an ordinary
projectile weapon (not a sonic-wave weapon) of damage 1, whose warhead has the sonic flag set in the
test file, unless the part says otherwise. Every test warhead fired by an own unit at the own
host has its affects-allies field on, except in part m, so that no part other than m depends on
the hits that S-0302 claim 8 stops. The host's owner keeps armour factors of 1.0, except in part k3.
*Observable* in every part, unless the part names more: after the probe, whether the parasite
comes out alive beside the host, stays attached, or is gone without coming out.
- *Part a (healing):* a healer vehicle of the own house repairs the damaged host once. *Expected:*
  the parasite is destroyed (no parasite comes out). Same with the host docking on a repair depot.
- *Part b (sonic):* a sonic hit lands on the host once. *Expected:* the parasite comes out alive.
- *Part c (suppressed, then sonic):* a test weapon with damage 100 hits the host (suppression time
  2 × 100 − 50 = 150 ticks), and a sonic hit lands on it 125 ticks later. *Expected:* the parasite
  is destroyed. Repeat with the sonic hit 175 ticks later: it comes out alive. The two probes lie
  clearly on either side of 150, and the later one lies below 200 (the time without the
  threshold), so they separate 2*d* − *T* from 2*d*.
- *Part c2 (boundary, only if scripted input (measurement setup) can place a hit on an exact tick):* the sonic hit 149 ticks
  after the hit, then 150 ticks after it. *Expected:* destroyed at 149, alive at 150.
- *Part d (below threshold):* damage 40, then a sonic hit at once. *Expected:* it comes out alive.
- *Part e (silent removal):* as part a, with the parasite type given a death weapon or explosion
  that is easy to see, and its house's kill and loss counts read from the score screen.
  *Expected:* no explosion where the parasite was, and no loss counted for its house.
- *Part f (naval exception):* a naval parasite type (one that grapples ships, with its suppression
  threshold set to 50) on an own ship; a test weapon with damage 100 hits the ship, then 125
  ticks later a sonic hit lands on it. *Expected:* the parasite comes out alive. Repeat with a
  healer repairing the ship: *Expected:* it comes out alive as well.
- *Part g (no free cell):* for a parasite that is not naval, and again for a naval one: a host with
  the parasite attached and every cell around it occupied by units or blocked, then a sonic hit.
  *Expected:* the parasite is removed silently (no explosion, no loss counted) in both cases.
- *Part h (healing that changes no health):* the host is at full health, and a damaged own test
  vehicle stands on the next cell. An own healer vehicle whose healing warhead is given a spread
  of 2 cells and the affects-allies field on in the test file is ordered to heal that neighbour once, so that the detonation
  reaches the host at 8 leptons or more from its point (S-0303, claim 13) and the host's health,
  read before and after, does not change. *Expected:* the host's parasite is destroyed, as in
  part a. *Falsified if:* the parasite stays attached. (A healer cannot fire at a host at full
  health directly, S-0310 claim 11, so the hit is delivered by the spread.)
- *Part i (the parasite's own damage):* the parasite type's suppression threshold is set to 0 in
  the test map, and its weapon is given damage *d*ₚ and a rate of fire in the test file such that
  the ticks between two of its hits on the host, read from the host's health drops, are fewer than
  2*d*ₚ. After its second hit, a sonic hit lands on the host fewer than 2*d*ₚ ticks after the
  parasite's latest hit (checked from the health record; a run that misses this is invalid).
  *Expected:* the parasite comes out alive. With threshold 0, a hit of *d*ₚ by anyone else would
  have suppressed it for 2*d*ₚ ticks, which had not yet passed, so a destroyed parasite means the
  parasite's own damage was counted. *Falsified if:* destroyed. Control: a test weapon with damage
  100 hits the host once (200 ticks) and a sonic hit lands 100 ticks later. *Expected:* destroyed.
- *Part j (a shorter second hit replaces the time left):* threshold 50; a test weapon with damage
  100 hits the host at tick *t* (150 ticks), a weapon with damage 60 hits it at *t* + 10 (70 ticks,
  so suppressed until *t* + 80 if the time is replaced), and a sonic hit lands at *t* + 100.
  *Expected:* the parasite comes out alive. Were the longer time kept (suppressed until *t* + 150),
  it would be destroyed. *Falsified if:* destroyed.
- *Part k (a hit reduced to nothing still counts):* threshold 50; an own test unit force-fires at the
  host's own cell with a weapon of damage 100 whose warhead has a versus value of 0 % against the
  host's armour class, so that the hit lands (S-0303) and the versus step brings it to 0 (S-0302,
  claim 6); the host's health is read before and after. A sonic hit lands 125 ticks later.
  *Expected:* the host's health is unchanged by the hit (a run in which it changes is invalid) and
  the parasite is destroyed. Control: a sonic hit 175 ticks later: it comes out alive.
  *Falsified if:* the parasite comes out alive after 125 ticks.
- *Part k2 (the edge of a detonation counts in full):* threshold 50; an own test unit force-fires
  at a cell next to the host with a weapon of damage 100, versus value 100 % against the host's
  armour class and a spread of 2 cells, so that the host is hit at a distance and loses less
  health than 100; a run in which it loses more than 87 is invalid (from an amount of 87 or less,
  a time of 2 × 87 − 50 = 124 ticks or less would have run out by the probe). A sonic hit lands 125
  ticks later. *Expected:* the parasite is destroyed (suppression from *d* = 100 lasts 150 ticks).
  *Falsified if:* it comes out alive.
- *Part k3 (the host's defences do not shorten the time):* threshold 50; the host's owner's country
  is given a vehicle armour factor of 2.0 in the test file (S-0302, claim 5), and an enemy test
  weapon with damage 100 and a versus value of 100 % against the host's armour class hits it once,
  so that it loses 50 health (a run in which it loses another amount is invalid). A sonic hit lands
  125 ticks later. *Expected:* the parasite is destroyed (from *d* = 100 the time is 150
  ticks, from the defended 50 it would be 2 × 50 − 50 = 50 ticks, run out long before). *Falsified if:*
  it comes out alive.
- *Part l (a sonic healing hit):* the host is first damaged by a test weapon with damage 40 (below
  the threshold, so not suppressed), and more than 200 ticks later an own healer vehicle whose
  healing weapon's warhead has the sonic flag set in the test file heals it once. *Expected:* the
  parasite comes out alive (the sonic rule decides, not the healing rule). *Falsified if:*
  destroyed. Repeat with a damage-100 hit 125 ticks before the sonic healing hit: *Expected:*
  destroyed (suppressed by the earlier hit).
- *Part m (a hit that S-0302 claim 8 stops still counts):* threshold 50. (1) An own test unit
  whose warhead has its affects-allies field off hits the host with damage 100; the host's health
  is read before and after and must not change (a run in which it changes is invalid). A sonic hit
  lands 125 ticks later. *Expected:* destroyed. (2) The same with an enemy test weapon of damage
  100 while the host is temporarily invulnerable (made so by the test house's invulnerability
  superweapon, its health unchanged by the hit). *Expected:* destroyed. (3) An own healer vehicle
  whose healing warhead has its affects-allies field off heals the damaged host once, with its
  health unchanged. *Expected:* destroyed (claim 7's healing rule). Control for each: the sonic hit
  175 ticks later in (1) and (2), which frees it alive. *Falsified if:* in (1), (2) or (3) the
  parasite comes out alive or stays attached.
- *Falsified if* (parts a to g): any outcome differs from the one expected; part c's probes
  measure the formula 2*d* − *T*, and part c2 the boundary.
- *Runs:* 3 per part (5 for part c's two timings), with different random seeds; all must agree.

**P5: first move versus repeated destination (claim 3; Q-0087).** A small flat skirmish
map, stock rules and art, director and mods off, game speed index 2. A human player's
ordinary crawl-capable soldier survives a hit and finishes lying down; a bystander computer
player is kept outside the observed area (the soldier and its route). Script input to give
it a move to a distant reachable cell while fear is still at least 50, then repeat that
same move while it is crawling and still has that destination. Use a fresh prone soldier
for a different-destination control and a move-to-occupied-cell control with no current
destination. Repeat with a prone cyborg for the type exception.
- *Observable:* record destination, stance and current sequence at each order boundary and
  each tick with the read-only recorder, plus drawn image frames and movement.
- *Expected:* first/different assignments and the occupied-cell control do not themselves
  start get-up; the repeated destination starts get-up at assignment when the sequence
  permits it, and ends prone state at that start. The cyborg does not get up from either
  assignment. Any fear-driven get-up before the test finishes invalidates that run.
- *Falsified if:* an eligible first/different assignment starts get-up, a repeated assignment
  with an interruptible current sequence fails to start it, the cyborg gets up from the
  assignment, or stance stays prone until get-up finishes. Three valid runs per case.
- *Order-family and refusal coverage:* S-0107 scenario M4 additionally varies attack-move,
  attack pursuit, area guard, queued route points, scatter, enter and capture, missing get-up
  image frames and a non-interruptible sequence. Its expected equality and timing results
  apply here; S-0213 owns which assignment each order makes.

**P6: lying down during a move (claim 5).** Use a purpose-made flat test map and named test data
   for two crawl-capable infantry test units, one owned by a human-controlled house and one by an
   original computer-controlled house. Neither is fraidycat or fearless. Keep both out of combat
   except for a single test hit from a known attacker that makes each survive with its fear level
   at 100; apply the hit only after each unit has begun a long, reachable move. Have the human
   player order the first unit to move by scripted input. Have a computer-player team script give
   the second unit its long, reachable move. Record each unit's movement destination, whether
   movement is in progress, its fear level (or the lie-down effect used by S-0107's measurement
   scenarios), and whether it lies down, from the hit until arrival. Void a run if a unit takes
   another hit, loses its destination, or fails to remain in movement.
   The team script and test data are measurement setup, not claims about our engine's AI.
   *Expected:* the human-owned original unit does not lie down while it has a destination or
   movement in progress; the original computer-owned unit lies down during its move while its
   post-hit fear remains above the lie-down threshold. On our engine, a corresponding AI-house
   unit follows the human-controlled rule, as the glossary's AI-house decision specifies.
   *Falsified if:* the human-owned original unit lies down during movement, or the original
   computer-owned unit does not lie down during a valid moving interval with fear above the
   threshold. Three valid runs per owner, with all runs agreeing.

## Open questions

- What happens to a parasite when its host is destroyed, suppressed or not (the original handles
  that case differently from the other ways of leaving), including a sonic hit that destroys the
  host.
- The complete list of ways a parasite leaves its host; this is left to S-0614 (parasites,
  planned), and this spec's claim 7 should then point to it.
- Whether a sonic hit that frees a parasite also makes the attacker drop its current target (the
  original appears to do so; not measured, low confidence).
