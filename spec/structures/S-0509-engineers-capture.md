---
id: S-0509
title: Engineers and capture
topic: structures
revision: 3
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# S-0509: Engineers and capture

## Summary

What happens when an infantry unit reaches a building it has been sent to, and in particular what
an engineer does there: capture an enemy building, repair a friendly one, be refused, or (under a
setting that a skirmish never uses) damage the building. It also gives the order of the trigger
raises around a capture, the fate of the engineer, the takeover of vehicle-like buildings, and what
a building's change of owner does to the things linked to it. Skirmish only; network games and
campaigns are out of scope.

Neighbouring specs own the rest:
- the orders and cursors that send an engineer (S-0214 claims 7 and 10, table I),
- the *capture* behaviour while the engineer walks (S-0215 claim 28),
- the arrival moment (S-0208 claim 22),
- bridge repair through a hut (S-0511 claims 17, 18 and 21),
- spies and other entrants that are not engineers (S-0510 claims 1 to 10), garrison occupation
  (S-0508), cash-producing and other tech buildings (S-0515),
- the hijack of vehicles (S-0524 claim 11),
- a trigger's change of owner (S-0011 claim 24), which has the building-ownership effects of
  claims 12 to 16 below.

## Terms

- **Building entry.** The moment an infantry unit in the *capture*, *area guard* or *patrol*
  behaviour arrives at the end of a walking step (S-0208 claim 22) in a cell that holds a building,
  while that building is its destination or its target. The conditions are the same as S-0510
  claim 1's. Claims 1 to 11 happen at that moment, within the same tick.
- **Used-up engineer.** An engineer whose building entry ends with its removal (claim 10). It is
  removed from play without being destroyed: it is not killed.
- **Arrival-friendly building.** For an entering engineer, a building that is one of these:
  - owned by the engineer's own house,
  - owned by a house that the engineer's house counts as allied (the one-way alliance of the
    glossary, read from the engineer's house),
  - of a garrisonable type and owned by a passive neutral house (S-0214 Terms).

  Unlike S-0214's *engineer-friendly building*, it does not depend on the type's repairable flag,
  on the building being vehicle-like, or on erasure.
- **Building ownership change.** A building's change from one owning house to another, whatever
  causes it: an engineer capture (claim 8), a takeover (claim 17), a map trigger (S-0011 claim 24)
  or any other transfer that the owning specs name. It has no effect when the new house is the old
  one.
- **Multi-engineer rule.** A game setting under which a single engineer cannot capture a healthy
  enemy building but damages it instead (claim 6). In a skirmish it is always off (S-0412 claim 1).
- **Construction yard.** Here, a building whose type has the construction-yard flag, as in
  S-0214 Terms, not a building selected from a computer player's yard list.
- **Lost-to-capture mark.** A house mark set when an engineer captures one of its buildings
  (claim 11).

## Behaviour

### Building entry

1. **When a building entry happens.** An infantry unit makes a building entry when it arrives at the
   end of a walking step (S-0208 claim 22) in a cell that holds the building, while its behaviour
   is *capture*, *area guard* or *patrol* and the building is its destination or its target.
   Standing in the building's foundation for any other reason, or in any other behaviour, is not a
   building entry. Two cases are excluded and handled elsewhere:
   - an occupier or assaulter type in *capture* whose destination is a building makes S-0508's
     entry attempt (S-0508 claim 2) instead, whether or not it can occupy that building. This takes
     precedence even for a vehicle-like building: refusal there does not become a takeover,
   - otherwise a destination building that is vehicle-like follows claim 17 instead.
   - Backing: `measurable` (scenario E1) for *capture* with a building destination. `unmeasured`
     for the target-only case and *area guard*/*patrol*: no proposed order fixture establishes
     those arrival states independently. The garrison exclusion is tested by S-0508 claim 2.
   - Confidence: high (*capture*, the destination-or-target test); medium (*area guard*, *patrol*)

2. **The general entry raise.** At every building entry, before any other effect of this spec, the
   building's tag receives one entry raise with the entering unit as causing object. This happens
   for every unit that makes a building entry (claim 1), whatever the outcome: capture, repair,
   refusal, damage, nothing, a bridge repair hut and infiltration. A raise has no per-tick de-duplication: a repeating trigger fires
   once per raise. The entered-by raise listed in S-0511 claim 17 for a hut is this raise; a hut
   gets no second one. Claim 1's garrison-route exception never receives this general raise:
   S-0508 claim 3 gives one garrison entry raise on success and none on refusal. After the general raise:
   - an engineer follows claims 3 to 11,
   - a spy or other infiltrator follows S-0510 (claims 1 to 9),
   - any other unit is refused and scattered as S-0510 claim 10 gives.
   - Backing: `measurable` (E2, E5 and E13) for capture, repair, refusal and no-effect entry.
     S-0510 claim 9 and S-0511 claim 17 own infiltration and hut tests. `unmeasured` for the
     damaging-hit case (claim 6) and the inaccessible arrival states of claim 1.
   - Confidence: high

### Engineer outcomes

3. **Outcome of an engineer's building entry.** *H* is the building's health and *S* its strength
   (both integers). Where row conditions overlap, the outcome in the higher row takes precedence. Row order states
   precedence between outcomes, not the order or method of testing.

   | Row | Building | Outcome | Engineer |
   |---|---|---|---|
   | 1 | a bridge repair hut, whoever owns it | S-0511 claims 17, 18 and 21 | S-0511 |
   | 2 | arrival-friendly, *H* = *S* | refused (claim 4) | stays |
   | 3 | arrival-friendly, *H* < *S* | repaired (claim 5) | used up |
   | 4 | not arrival-friendly, the multi-engineer rule on and the health ratio above the condition-red fraction (F1) | damaging hit (claim 6) | used up |
   | 5 | not arrival-friendly, type not capturable | nothing happens to the building (quirk below) | used up |
   | 6 | not arrival-friendly, being sold or being erased | refused (claim 4) | stays |
   | 7 | not arrival-friendly, any other | captured (claims 7 and 8) | used up |

   Rows 2 and 3 do not test the type's repairable flag or whether the building is being sold. Those
   tests belong only to the cursor and the order (S-0214 table I). With the proposed sale-repair
   quirk off, the sale-time refusal described under Edge cases overrides rows 2 and 3.

   The row is chosen after claim 2's raise, from the state at that moment. A trigger action fired
   by that raise can therefore change the outcome. For example, an action that gives the building
   to the engineer's house turns a capture into a repair or a refusal.
   - Backing: `measurable` (scenarios E2 to E5 and E13) for rows 2, 3, 5, 6 (erasure) and 7,
     including sale-time repair in E3. `unmeasured` for row 4 (claim 6), the enemy sale case of
     row 6 (no controlled sale fixture), and the missing repairable test in row 3 (no proposed
     fixture can give the order before bypassing that cursor gate).
   - Confidence: high (rows 2, 3, 6 and 7); medium (rows 4 and 5, and the missing sale test in rows 2 and 3)

4. **Refusal.** A refused engineer has its destination cleared and receives a scatter request whose
   threat point is the building's centre, forced and move-anywhere (S-0208 claims 11 to 13). It is
   not used up and makes no further raise. Its behaviour is not changed by the refusal. What it
   does next follows S-0215 claim 28 for its behaviour, now without a destination. An engineer sent
   by a capture order has no target (S-0214 table M), so it does not walk back to the building.
   - Backing: `measurable` (scenarios E3 and E5) for survival, scattering and no re-entry.
     `unmeasured` for destination clearing, unchanged behaviour and the exact scatter-request
     options: those scenarios do not distinguish these from other requests with the same movement.
   - Confidence: high

5. **Repair.** The building's health becomes its strength. If the building's own repair was
   switched on (S-0213 claim 13, S-0314), it is switched off. Its damage-dependent animations are
   chosen again for the new health. The game-wide building-repaired sound plays at the building's
   centre. The engineer is used up (claim 10).
   - Backing: `measurable` (scenario E3)
   - Confidence: high

6. **Damaging hit (multi-engineer rule only).** The engineer deals the building one hit of amount
   *A* (F2), with the game-wide demolition warhead (glossary), at distance 0, with the engineer as
   attacker and no attacking house, ignoring defence bonuses and not preventing passenger escape.
   S-0302 resolves the hit. The building is not captured, even if it survives. The engineer is used
   up (claim 10). The test is made even for a type that is not capturable. In a skirmish the rule is
   always off, so this row never applies there.
   - Backing: `unmeasured` (the setting cannot be turned on in a skirmish)
   - Confidence: medium

7. **Capture does not depend on health in a skirmish.** With the multi-engineer rule off, an
   engineer that makes a building entry into a capturable building that is not arrival-friendly, not
   being sold and not being erased captures it at any health, full health included. The capture
   threshold of S-0214 (table I row I7, Formulas F2) changes only the cursor and the order (the
   *damaging entry* action). It plays no part when the engineer arrives.
   - Backing: `measurable` (scenario E4)
   - Confidence: high

### Capture

8. **Capture and its ordering.** A capture gives the building a second entry raise, with the engineer
   as causing object, on top of claim 2's. It sets the lost-to-capture mark (claim 11) of the house
   that owns the building at that moment, and makes a building ownership change to the engineer's house (claims 12 to 16), with its
   announcements (S-0701 claim 39). If the engineer carries a tag whose trigger chain includes the
   carry-over flag, the building receives that tag (S-0524 claim 10). All of it happens in the
   tick of the building entry. Observable order:
   - Both entry raises precede the capture's ownership and tag changes. Without intervening
     trigger changes they see the old owner and tag. Trigger actions can themselves change that
     state, and later raises see those changes.
   - The tag carry-over happens after both entry raises. Its order relative to the ownership
     change is not established by E6; this spec leaves that relative order unresolved.
   - The scatter of claim 9 comes after the ownership change.
   - The engineer's destroyed raise (claim 10) comes after the ownership change. When the tag
     carried over, that raise acts on the tag the building now shares.

   - Backing: `measurable` (scenarios E2 and E6) for raises before takeover, carry-over before
     the destroyed raise, and takeover before that raise. `unmeasured` for scatter ordering
     (claim 9 has no reliable fixture) and which owner receives the lost-to-capture mark after
     a second-raise transfer (E7 has no such intervening transfer).
   - Confidence: high (the raises, the carry-over, the ownership change and the established
     ordering); medium (scatter ordering and mark attribution after an intervening transfer)

9. **Infantry standing on a captured building.** After the ownership change, every infantry unit on
   the map whose position is in a cell holding the captured building, and whose destination is none
   or this building, receives a scatter request with no threat point, forced and move-anywhere
   (S-0208 claims 11 to 13). Units of every house count.
   - Backing: `unmeasured`: no stock setup reliably leaves an idle infantry unit standing in a
     building's foundation at the moment of capture
   - Confidence: medium

### The used-up engineer

10. **Removal of a used-up engineer.** At the end of a repair, a damaging hit, a capture, or row 5
    of claim 3, the engineer's own tag receives one destroyed-by-anything raise with the engineer as
    causing object (S-0524 claim 10). The engineer is then removed from play. It is not killed:
    - there is no death sequence and no death effect of S-0312,
    - no kill or loss is counted for any house (S-0414),
    - no experience is awarded (S-0311).
    - Backing: `measurable` (scenarios E2 to E4) for repair/capture removal, the raise and absence
      of death effects; E4 also tests capture kill/loss counts. `unmeasured` for experience
      (no attacker is present in these fixtures), repair and row 5 kill/loss counts (no isolated
      score-screen control), and damaging-hit removal (claim 6). E13 covers row 5 removal.
    - Confidence: high (the raise and the removal); medium (no kill or loss counted)

11. **Lost-to-capture mark.** A house receives this mark when an engineer captures one of its
    buildings (claim 8). Nothing else sets it: a takeover (claim 17), a trigger transfer and every
    other ownership change leave it alone. Nothing clears it during the game. Trigger event code 3
    tests this mark for its specified house: whether that house has ever lost a building to an
    engineer capture. No other gameplay effect of the mark is specified here.
    - Backing: `measurable` (scenario E7) for capture, trigger transfer and persistence for the
      observed interval. `unmeasured` for every other transfer and persistence for an arbitrary
      game duration: E7 cannot exercise every transfer or duration.
    - Confidence: high

### Building ownership change

12. **Effects owned elsewhere.** A building ownership change has the effects that these specs give:

    | Effect | Owner |
    |---|---|
    | Capture/loss announcement selection and playback, subject to eligibility below | S-0701 claim 39 |
    | Old owner's counts drop, new owner's grow; a gain counted as a kill | S-0414 claims 4, 20 and 21 |
    | Power totals worked out again | S-0405 claim 7 |
    | Sight and the shroud for both houses | S-0409 claim 8 |
    | Factory counts and production lines checked; production abandoned | S-0403 claim 10, S-0404 claims 8 and 15 |
    | The primary-building mark cleared | S-0213 claim 13, S-0402 Terms |
    | A refinery's place in the holding order; no free unit after a capture; a master's slaves | S-0402 Terms, claims 18 and 23 |
    | Self-healing totals; the captured mark and automatic repair | S-0314 claims 10 and 13 |
    | Pad totals | S-0501 claim 30 |
    | Survivors of a later sale (crew, never an engineer) | S-0408 claim 7 |
    | A running delayed-death countdown is kept | S-0302 claim 16, S-0312 claim 3 |
    | Experience kept | S-0311 claim 13 |
    | Superweapons granted by buildings | S-0512 claim 3 |
    | Cloak field, sensor array, gap generator | S-0516 claims 20 and 23 |
    | A missile silo's spawner | S-0506 claim 13, S-0513 claim 13 |
    | Persistence of infiltration effects, not a building spy mark | S-0510 claims 5, 7 and 8 |
    | Occupants and their houses are unchanged | S-0508 claim 17 |
    | An empty civilian garrison building returns to the civilian house at its next turn | S-0508 claim 8 |
    | Manned state, cash and secret-lab effects | S-0515 claims 2 to 8 |

    Capture/loss announcements are eligible only when the local player is the old or new owner,
    the new owner's country is not passive, and the cause of the transfer requests an announcement.
    Engineer capture (claim 8) and vehicle-like takeover (claim 17) request one. Trigger owner
    changes (S-0011 claim 24) and occupation ownership changes (S-0508 claims 7 and 8) do not.
    Occupation's own garrison/abandonment feedback is separate (S-0508 claims 6 and 8).
    Eligibility is not a promise of audible playback: S-0701 claim 39 owns the further selection
    and playback conditions.

    - Backing: `measurable` (E14) for announcement eligibility in its tested transfers.
      `unmeasured` for passive new owners and transfers between two non-local houses, which E14
      does not independently arrange. The dependency index is `unmeasured` here, not an additional
      claim of independent confirmation; the owning claims specify their actual rules and tests.
    - Confidence: high (announcement eligibility); medium (completeness of this index)

13. **A planted bomb is disarmed.** If a timed bomb planted by an infantry unit is on the building
    (S-0312 claim 7), the ownership change removes it without a detonation. It never goes off later.
    The exception is a building of a garrisonable type: there the bomb stays and keeps counting
    down.
    - Backing: `measurable` (scenario E8)
    - Confidence: medium

14. **Contacts follow or are sent away.** Each object in contact with the building at the moment of
    the change is asked whether it is moving. It counts as staying when it has no destination or its
    movement has stopped. A staying contact whose position is within 63 leptons of the building's
    docking point (F3) changes owner together with the building and keeps its contact and any tether
    link. For a factory of vehicles, the distance is not tested. Every other contact is told to
    leave and loses its contact and tether link with the building. Examples are a harvester
    unloading at a captured refinery, a vehicle on a captured repair depot and a vehicle just
    leaving a captured factory.
    - Backing: `measurable` (scenario E9) for ordinary docked-contact transfer and moving-contact
      rejection. `unmeasured` for the exact 63-lepton boundary, contact/tether retention or removal,
      and stopped factory contacts far from the dock: no proposed fixture establishes those
      states independently of the ownership change.
    - Confidence: medium

15. **Building placement reset.** When the new owner is the local player's house and the building is
    a construction yard, the local player's building placement in progress, if any, is cancelled.
    - Backing: `measurable` (scenario E10)
    - Confidence: low

16. **Computer-only sale (original only).** In the original, the following happens when all of
    these hold:
    - the new owner is computer-controlled,
    - the building's initial owner (the house it belonged to when it was created) is another house,
      and that house is not passive,
    - the game is not a campaign.

    Upgrade removal uses the top-position selection and count changes of S-0408 claim 5 until
    the upgrade count becomes 0. Each removed upgrade credits the new owner with its full refund:
    S-0408 claim 1 with refund percentage 1, including its refund override and current country
    and cost-bonus factors, not necessarily the upgrade's imported cost. A turret upgrade resets
    the count to 0, so lower recorded upgrades need not be removed by this operation. The building
    is then put up for sale. **In our engine** this automatic sale never applies to AI player houses:
    they keep captured buildings by the same rules as human houses (glossary AI house).
    **Note for AI designers:** the original's computer player sells qualifying captured buildings
    at once, receiving the full refunds of the removed upgrades. Our AI can choose ordinary sale
    orders instead.
    - Backing: `measurable` (original only; scenario E11) for immediate sale and upgrade refund
      after capture from a player house. `unmeasured` for the initial-owner and passive-house
      exceptions, multiple-upgrade removal and the turret count reset: E11 does not vary these
      conditions independently.
    - Confidence: medium

### Vehicle-like buildings

17. **Takeover of a vehicle-like building.** A vehicle-like building (S-0214 Terms) is taken over by
    any infantry unit whose behaviour is *capture*, *area guard* or *patrol* and whose destination
    is that building, when the unit arrives at the end of a walking step in the building's own cell,
    except for claim 1's prior garrison route. An occupier or assaulter type in *capture* is refused
    there under S-0508 claim 2, not consumed by takeover. Otherwise the unit's type, including its
    spy flag, plays no part. Neither alliance nor ownership is tested. The building's tag receives one
    entry raise with the unit as causing object, which sees the old owner. Then:
    - a building ownership change to the unit's house follows, with announcements,
    - claim 9's scatter follows,
    - if the unit carries a tag whose trigger chain includes the carry-over flag, the building
      receives it after the ownership change. Claim 8 leaves capture's corresponding relative
      order unresolved.

    The unit's own tag then receives a destroyed-by-anything raise, and the unit is removed from play
    without being killed, as in claim 10. The lost-to-capture mark is not set.
    - Backing: `measurable` (scenario E12, with a test-map override, since stock data has no
      vehicle-like building type) for a vehicle thief's entry raise, takeover, consumption and
      lost-to-capture mark. `unmeasured` for other infantry types, own-house takeover and the
      *area guard*/*patrol* cases (no order sets up their arrival), for scatter (claim 9), and
      for carry-over ordering (E12 has no independently timed tag-transfer fixture).
    - Confidence: low

## Formulas

**Floating-point convention.** F1 and F2 use S-0205 claims 4, 5 and 7: every arithmetic result
has a 53-bit significand, rounded toward zero under `quirks.chop_rounding` (default original),
or to nearest, ties to even, with that switch off. Integer operands are converted exactly.
Integer conversions always truncate toward zero. These formulas are `unmeasured` here because
the multi-engineer rule is unreachable in skirmish (claim 6).

**F1. Health ratio test (claim 3 row 4).** For positive integer strength *S* and integer health *H*,
*r* is the floating-point quotient *H* / *S*, rounded once as above, not an integer quotient.
The dimensionless ratio is compared exactly with the imported double-precision condition-red
fraction *c*. Row 4 applies when *r* > *c*, strictly; equality does not qualify.

**F2. Damaging-hit amount (claim 6).** *S* and *H* are integers, *c* is the condition-red fraction
(double precision).

> *D* = *H* − trunc(*S* × *c* × 0.5)
>
> *Y* = ((1 − *c* × 0.5) × 0.5) × *S*
>
> *A* = trunc(*Y*) if *Y* < *D*, else *D*

Each floating-point multiplication and subtraction is rounded by the convention above, not
with a wider significand. In *D*, multiply *S* by *c*, then by 0.5. In *Y*, multiply *c* by
0.5, subtract that result from 1, multiply by 0.5, then multiply by *S*. *D* is an integer in hit points; its integer
subtraction is exact in the ordinary health range. *Y* stays fractional, in hit points, until the
final truncation. The comparison with *D* is exact, with *D* converted to floating point.
The hit then passes through S-0302 like any other hit, where warhead modifiers and armour apply.

**F3. Contact distance (claim 14).** The three-dimensional difference in leptons between the
building's docking point for that contact and the contact's position, with integer components. Its
length is taken by the table square root of S-0204 claim 5, which gives an integer. The contact is
within the distance when that length is at most 63. At this size the table root equals the exact
length rounded down.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Entry raises on a capture | 2 | raises | Claims 2 and 8 |
| Entry raises on any other building entry or takeover | 1 | raises | Claims 2 and 17 |
| Contact follow distance | 63 | leptons | Claim 14. Not tested for a factory of vehicles |
| Damaging-hit distance | 0 | leptons | Claim 6 |
| Damaging-hit factors | 0.5, 1 | none | F2 |

## State and ordering

| State | Event | Result |
|---|---|---|
| engineer walking in *capture*, *area guard* or *patrol* | building entry, arrival-friendly building at full health | one entry raise; refused; engineer scatters away from the building centre |
| same | building entry, arrival-friendly building damaged | one entry raise; repaired; engineer's destroyed raise; engineer removed |
| same | building entry, capturable building that is not arrival-friendly, not sold and not erased | two entry raises; capture and tag carry-over under claim 8's partial ordering; infantry scatter (claim 9); engineer's destroyed raise and removal |
| same | building entry, capturable building being sold or erased, not arrival-friendly | one entry raise; refused |
| same | building entry, building of a type that is not capturable, not arrival-friendly | one entry raise; engineer's destroyed raise; engineer removed |
| any infantry in those behaviours, except claim 1's garrison route | arrival in a vehicle-like destination building's own cell | entry raise, ownership change, infantry scatter, tag carry-over, unit's destroyed raise, unit removed |

The tables summarise the default outcomes, not a total order of effects. In particular, they
establish no order between capture's tag carry-over and ownership change. Backing and confidence
are those of the numbered claims.

Ordering consequences:
- Neither the capture's ownership change nor its tag carry-over occurs between its two entry
  raises. Either raise can therefore act on the old house's building, unless trigger actions
  themselves have changed its ownership (S-0524 claim 10).
- When the engineer's tag carries over, its destroyed raise acts on the tag that the building now
  shares (S-0524 claim 10).
- The engineer is removed in the same tick as the capture (claim 8). Trigger actions raised
  before removal can still affect objects through the shared tag (E6). No other object takes
  its ordinary turn between the ownership change and removal. That last assertion has high
  confidence but `unmeasured` backing: E6 does not independently probe another object's turn
  within the arrival update.

## Edge cases and quirks

- **Engineer lost on a building it cannot capture.** An engineer that enters a building that is not
  arrival-friendly and whose type is not capturable (claim 3 row 5) is used up and changes nothing.
  This can happen when the building was arrival-friendly when the order was given and stopped
  being so before the arrival (an alliance broken while the engineer walks).
  `quirks.engineer_lost_on_uncapturable` (default on, the original). With the switch off, such an
  entry is refused as in claim 4 and the engineer stays.
- **Full-health capture.** An engineer captures an enemy building at full health (claim 7). This is
  the original's design in a skirmish, not a bug. No switch.
- **Repair of a building that is being sold.** Rows 2 and 3 of claim 3 do not test a sale. An
  engineer that reaches a damaged arrival-friendly building during its sale repairs it and is
  used up; at full health it is refused instead. Proposed `quirks.engineer_repairs_during_sale`
  (default on, original). With it off, sale-time entry is refused as in claim 4, regardless of
  health, after the general entry raise. This proposal needs the spec owner's sign-off.
  Claim 3's ordinary precedence table and the state table describe the default setting.

## Import mapping

| Key | Section | Feeds | Parse form | Absent default |
|---|---|---|---|---|
| `Engineer` | infantry type | engineer type flag (claims 2 to 10) | boolean | no |
| `Capturable` | building type | capturable type (claim 3 rows 5 and 7) | boolean | no |
| `BridgeRepairHut` | building type | bridge repair hut (claim 3 row 1) | boolean | no |
| `CanBeOccupied` | building type | garrisonable type (Terms, claim 13) | boolean | no |
| `Repairable` | building type | not read at a building entry (claim 3); cursor and order only (S-0214) | boolean | yes |
| `NeedsEngineer` | building type | tech building effects on ownership change (S-0515) | boolean | no |
| `UndeploysInto` | building type | vehicle-like building (claim 17, S-0214 Terms) | type name | none |
| `MultiplayPassive` | country section | passive neutral house (Terms) | boolean | no |
| `ConditionRed` | `[AudioVisual]` | condition-red fraction *c* (F1, F2) | decimal (S-0003), kept in double precision | 0.5 (S-0302 claim 12) |
| `BuildingRepairedSound` | `[AudioVisual]` | building-repaired sound (claim 5) | sound name | none (silent) |
| `C4Warhead` | `[CombatDamage]` | demolition warhead for the damaging hit (claim 6) | warhead name | none |
| `MultiEngineer` | `[MultiplayerDialogSettings]` | multi-engineer rule; forced off in a skirmish (S-0412 claim 1) | boolean | off |
| `EngineerCaptureLevel` | `[General]` | cursor threshold only (S-0214 table I row I7); never read at a building entry | fraction, single precision | S-0214 |

Initial absent-key defaults have high confidence and `unmeasured` backing here: none of E1–E14
removes the supplied lower rules layers to expose initial defaults. Later-layer omission normally
retains the value already in force (S-0006 claim 1).

## Measurement scenarios

General setup: glossary measurement setup, stock data, director and mods off except the explicitly
named test-map overrides, a tiny purpose-made skirmish map, one human house and one bystander
computer player kept away from the test area, standard mode, game speed index 2, 800 × 600.
The observed area includes the named test buildings and the entrants' approach paths. Apply the
bystander void rule. Record ticks, owners, health, positions, removals and visible trigger effects
with a read-only recorder and tick-labelled video with sound. Repeat each variant twice.
Scripted mouse and keyboard input may time orders but never alter running state. A state or timing
that the recorder cannot resolve makes that variant inconclusive, not a pass. Add the player houses
named by a variant, keeping an undefeated hostile bystander so an allied test does not end the game.
Trigger-counter effects must be calibrated to distinguish two firings in one tick from one firing.

- **E1. Building entry (claim 1).** Order an engineer to capture an enemy capturable building.
  Expected: the capture happens in the tick the engineer finishes its step into a cell of the
  foundation (S-0208 claim 22). In a second run, give the engineer a stop order one cell short.
  Expected: no capture while it stands next to the building. Falsified by a capture without an
  arrival in a foundation cell, by capture before or after the arrival tick, or by no capture
  after an otherwise eligible arrival.
- **E2. Raise counts (claims 2, 8 and 10).** Give the target building a repeating tag whose trigger
  has the entered-by event and an action with a calibrated visible counter. Give the
  engineer a repeating tag with a destroyed-by-anything event and its own counter. Cases:
  - (a) capture an enemy building,
  - (b) repair a damaged allied building,
  - (c) send two engineers to the same damaged allied building, the second a few cells behind the
    first. No order can send an engineer to a full-health allied building (S-0214 row I5 gives an
    action without an order), so the second one meets the building already repaired,
  - (d) use a one-time entered-by trigger to give a damaged enemy building to the engineer's
    house at the general raise. Expected: one entry firing, repair rather than capture, one
    engineer destroyed firing. At full health instead, expect refusal and no destroyed firing.

  Expected entry counts: (a) 2, (b) 1, (c) 2 in total. Engineer destroyed counts: (a) 1, (b) 1,
  (c) 1 in total, and the second engineer of (c) is still on the map.
  Falsified by any other count, or by capture instead of the repair/refusal predicted in (d).
  Other entrants are covered by S-0510 scenarios I6 and I8.
- **E3. Allied repair and refusal (claims 3 to 5).** An allied building at 50 % health, and two
  engineers ordered to it, the second a few cells behind the first. Expected: the building returns
  to full health in the first engineer's arrival tick, the building-repaired sound plays, and the
  first engineer is gone. The second engineer arrives, stays, moves away from the building's centre
  and does not come back. Switch the building's own paid repair on just before arrival: expect
  it off after the engineer repairs it. Record the damaged and healthy animation state, absence
  of an engineer death sequence/effect, and absence of a repeated entry over the next 90 ticks.
  Repeat for an own-house building and a building of an allied player house; use a separate map
  variant with a passive-neutral garrisonable building for arrival-friendliness there.
  Falsified by incomplete healing, continuing paid repair, a missing repair sound when enabled,
  unchanged damaged animations, a killed first engineer, consumption of the second, or re-entry
  without a new order. Scatter position must satisfy S-0208 claims 11 to 13, not a fixed cell.
  Sale variant: order an engineer to an own damaged building, then sell it just before arrival
  using timed input. Only count runs whose entry raise occurs during sale and before removal.
  Expected: repair and engineer consumption, not refusal; refusal while the damaged building
  still exists falsifies the default sale-repair rule. No such timed arrival means inconclusive.
- **E4. Capture at any health (claims 7 and 10).** Enemy capturable buildings at full health, at
  60 % and at 20 %. Expected: all three captured in the arrival tick, the engineer removed, and no
  kill or loss counted for the engineer on the score screen. Falsified by a damaging hit or a
  refusal. In a second set of runs, a named test-map override lowers the capture threshold (S-0214
  F2) so that the cursor offers the *damaging entry* action at full health. Order that action.
  Expected: a capture all the same; any damage-only outcome, surviving engineer, engineer death
  effect, or engineer kill/loss count falsifies the corresponding prediction in either run set.
- **E5. Erasure refusal (claim 3 row 6).** The human house's temporal-weapon infantry starts erasing
  an enemy capturable building when the engineer is one cell away. Expected: the engineer stays and
  scatters, and E2's entry counter shows 1. The sale case of row 6 is `unmeasured`: the bystander
  computer house cannot be made to sell on cue. Falsified by capture, consumption, no entry
  firing, or a scatter inconsistent with S-0208 claims 11 to 13 while erasure is active.
- **E6. Tag carry-over order (claim 8).** Give a tagged enemy building a repeating entered-by
  trigger that changes its carriers to a third, still hostile house and produces a calibrated
  visible counter. The engineer carries a repeating carry-over tag with a destroyed-by-anything
  action that changes all its carriers to a fourth house. Expected: two old-tag entry firings,
  then one engineer destruction firing that also changes the captured building to the fourth
  house. Its final owner is the fourth house, not the engineer's house: destruction acts after
  ownership transfer and through the shared tag. Destroy the building later: the carried tag
  fires again. Repeat without the destruction action to change owners: final owner is the
  engineer's house. Repeat without carry-over: the building keeps its old tag and is not changed
  by the engineer's destruction action. Falsified by any other count, owner or later-tag effect.
  Record carrier owners at each visible action; a repeated entry action before capture must
  be able to act on the building already transferred by the first action. This checks that
  trigger changes, unlike capture's own transfer, can occur between entry raises. These results
  do not distinguish tag carry-over before ownership change from carry-over after it; neither
  order alone is confirmed or falsified by E6.
- **E7. Lost-to-capture mark (claim 11).** A test map with a trigger on event code 3 for house X,
  with a visible action. Capture an X building: expected to fire. Transfer an X building by trigger
  instead: expected not to fire. Use fresh games for these controls so an earlier capture
  cannot leave the mark set. In the capture run, return the building by trigger and evaluate a
  fresh event-code-3 trigger after 90 ticks: expected to fire again. Falsified by no event after
  capture, an event after trigger transfer alone, or the later event failing after return.
- **E8. Planted bomb (claim 13).** Plant a timed bomb on an enemy power plant, then capture it
  before detonation. Expected: no explosion, ever, and health unchanged. Repeat on a garrisonable
  civilian building owned by the enemy: expected the bomb explodes on schedule. Record the
  planting tick and the control detonation tick in an otherwise identical no-capture run; watch
  both capture cases beyond that tick by 90 ticks. Health is compared just after capture and
  before any other damage. Falsified by a later bomb detonation on the ordinary building or by
  loss of the garrisonable building's bomb. An unexplained health loss also falsifies disarming
  without damage; the finite run tests scheduled detonation, not every possible future tick.
- **E9. Contacts (claim 14).** Capture an enemy refinery while its harvester is unloading, an enemy
  repair depot while a vehicle is being repaired on it, and an enemy factory while a new vehicle is
  leaving it. Record positions and whether each named vehicle is stopped at the transfer tick.
  Expected: an unloading harvester and a stopped depot vehicle within F3's distance change owner.
  A still-moving factory contact keeps its owner and is sent away. Compare a vehicle approaching
  the dock only if it is independently known still to be a contact; an unrelated nearby vehicle
  is not a rejection test. Falsified by a docked, stopped near-contact keeping its owner, a moving
  factory contact transferring, or a rejected contact not leaving when a legal route is free.
  These runs do not isolate the exact boundary or the links named in claim 14.
- **E10. Placement reset (claim 15).** Low priority. With a building ready to place and the placement
  cursor active, have a trigger give the human house an enemy construction yard. Expected: the
  placement cursor is cancelled. A control transfer of an ordinary building must not cancel it.
  Falsified by a retained construction-yard placement cursor or cancellation in that control.
- **E11. Computer-only sale (claim 16, original only).** Let a computer house's engineer capture two
  buildings of the human house, one without upgrades and one with an upgrade. Expected: the computer
  house puts both up for sale at once, and its credits rise by the removed upgrade's full refund
  under S-0408 claim 1 before the building sale refund. Predict that integer credit amount using
  the new owner's current factors and the upgrade's refund override, if any. Add a named test-map
  override giving the upgrade a nonzero refund override distinct from its cost: expected the
  override-based full refund, not the cost. Record upgrade removal, sale-start tick and credits
  read-only, keeping the new owner's other income and spending out of that interval. A run with
  unrelated spending is void. Falsified by no immediate sale, retention of the single counted
  upgrade, or a credit amount differing from that full-refund prediction. Recorded only to
  document the original; no AI rule is prescribed by this scenario.
- **E12. Vehicle-like takeover (claim 17).** A test-map override gives a 1 × 1 building type an
  undeploy-into vehicle. Order a vehicle thief into an enemy building of that type. Expected: the
  building changes to the thief's house when the thief reaches its cell, the thief is removed, an
  entered-by trigger on the building fires once, and an event-code-3 trigger for the old owner
  does not fire. Give the thief a repeating destroyed-by-anything counter: expected one firing
  and no death sequence, death effect or kill/loss count. The building's entry action also changes
  its carriers to a third house: final owner must still be the thief's house, showing that the
  entry action precedes takeover. Falsified by different timing, owner, counts, a surviving thief,
  a killing effect or a lost-to-capture event. Repeat on an allied but other-house building:
  expected the same takeover and removal. Carry-over timing and own-house arrivals remain
  unmeasured as claim 17 states.
- **E13. Uncapturable after ordering (claim 3 row 5).** A named test-map override makes a repairable
  building non-capturable. It starts owned by an allied player house and damaged, so a repair
  order is legal. After the engineer begins walking, use a timed trigger transfer to give it to
  a hostile house before arrival. Record the arrival owner's house, health, entry counter and
  engineer destroyed counter. Expected: one entry firing, unchanged health and owner, one engineer
  destroyed firing and immediate removal. Falsified by repair, capture, refusal or survival.
  This tests the default of `quirks.engineer_lost_on_uncapturable`; the off-state is a design
  alternative tested in our engine, not a different oracle setting.

- **E14. Announcement eligibility (claim 12).** In fresh games with the announcement queue idle,
  have the local player's engineer capture an enemy ordinary building and a passive-house tech
  building. Record the capture tick, ownership, announcement requests read-only, and sound.
  Expected: eligible capture feedback, subject to S-0701 claim 39. Transfer the same kinds of
  building to the local player by a map trigger instead: expected no capture/loss announcement
  request. In a further run, occupy a civilian building and then evacuate it; record ownership
  turns and distinguish garrison/abandonment feedback from capture/loss feedback. Expected:
  no capture/loss request from either occupation transfer. Falsified by a capture/loss request
  from either silent transfer cause, or missing eligible capture feedback with all S-0701 gates
  satisfied. If requests cannot be distinguished from queue suppression, that variant is
  inconclusive. Passive-new-owner and two-non-local-owner transfers remain unmeasured.

## Changes for other specs

- S-0408, Edge cases and quirks, original computer-player note: replace the ownership-change
  upgrade-sale description with S-0509 claim 16's initial-owner/passive-house conditions and
  repeated top-upgrade removal with full refunds. It is not universally just one top upgrade.

## Open questions

- Does capture tag carry-over precede the ownership change or follow it? E6 establishes only
  that both follow the entry raises and precede the engineer's destroyed raise. No discriminating
  scenario is yet proposed; the relative order is unmeasured and not prescribed (claim 8).
- Claim 17 rests on one reading of the arrival rule and has no stock case. Low confidence.

## Questions for the project lead

- Confirm the proposed `quirks.engineer_repairs_during_sale` name and off-state refusal. Its
  default preserves the original's gameplay-affecting engineer loss; it is not a display-only bug.
