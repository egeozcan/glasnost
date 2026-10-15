---
id: S-0405
title: Power (output and drain, damaged power plants, low power, blackouts, which buildings stop)
topic: economy
revision: 15
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: [Q-0180, Q-0224, Q-0276]
supersedes: []
---

# S-0405: Power (output and drain, damaged power plants, low power, blackouts, which buildings stop)

## Summary

How much power each building makes and needs, how a house's totals and its power fraction are
formed and when they are worked out again, the blackouts that cut a house's power to nothing, the
power switch, and what low power changes: which buildings stop working, the units that depend on
a control centre, and the pointers to the production slowdown and to the radar rule (S-0410), in a
skirmish. Campaigns and network games are out of scope.

## Terms

Shared terms used here are defined in the glossary: *house*, *building*, *type*, *field*, *tick*,
*tick number*, *health*, *strength*, *health ratio*, *in play*, *on the map*, *removed*,
*behaviour*, *firing check*, *being erased*, *AI house*, *computer-controlled*,
*human-controlled*, *importer warning*, *quirk switch*, *absent-key default*. S-0205's
notation applies: trunc, RZ₅₃, RN₅₃,
RZ₂₄, *single* and *double*. *Electromagnetic pulse* is S-0203's (Terms), *radar* and *radar
building* are S-0410's, *gap generator* and *gap field* are S-0409's (Terms), *spy satellite* is
S-0409's (Terms), and *construction* and *selling* are the behaviours of S-0215. *Turn* is used
here in S-0203's sense, an object's share of one tick, as in "its own turn" and "the turn pass"
(glossary, *turn*); no rule here speaks of rotation. *House update* is S-0203's (claim 8): a
house's own update in a tick, which comes after the turn pass and before orders are processed
(S-0203, claims 7 and 8). In addition:

- **Switched on, switched off.** The state of a building's power switch, which the power orders of
  S-0213 (claim 13) change. S-0213 calls the two states *online* and *offline*; this spec writes
  "switched on" and "switched off" for the same states. A building is switched on when it is created
  in play. A map's request to start it switched off is subject to claim 18; S-0009 owns the
  map field's encoding.
- **Base output, base drain.** Two non-negative integers of a building type, both read from one
  signed integer field, the *power field*: a value of 0 or more is the base output and gives a base
  drain of 0; a negative value *v* gives a base drain of −*v* and a base output of 0 (Import
  mapping). The same split applies to the type's *extra power field*, giving the *extra output* and
  the *extra drain*.
- **Building output, building drain.** What one building makes and needs at a moment (claims 1 and
  2), in power units (integers).
- **Counted building.** A building of the house that is on the map (claim 3).
- **House output *O*, house drain *D*.** The sums of the counted buildings' outputs and drains, as
  the house last worked them out (claims 4 and 7), except that *O* is 0 when the house works it out
  during a blackout (claims 5 and 6).
- **Low power.** A house has low power when *D* > 0 and *O* < *D*. Otherwise it has full power.
- **Power fraction *p*.** A double from 0 to 1 (Formulas F3). S-0403 (claim 5) uses it for the
  production slowdown.
- **Blackout.** A house state in which its output counts as 0 (claim 5). A *timed blackout* is
  *running* from its start until the end given in claim 6; a *drain blackout* lasts while one of its
  counted buildings that has output is being drained.
- **Being drained.** The state of a building to which an enemy unit with a draining weapon is
  attached (special-abilities specs own the attaching and which building types can be drained).
- **Charger.** An infantry unit registered as a building's charger by a charge shot.
  S-0518 claims 11–12 own registration and removal, including registrations retained by
  non-overpowerable buildings under `quirks.stale_chargers_kept` (default original).
  *n*<sub>c</sub> is the building's current registered-charger count, the same count as
  S-0518 claims 11–12 use, not a fresh count of units currently aiming at it.
- **Absorbing building.** A building whose type takes in infantry, vehicles or both as passengers
  that do not come out on their own (S-0312, Terms, *absorbed passengers*). *n*<sub>a</sub> is the
  number of passengers it holds.
- **Upgrade, upgrade positions.** A building type that is added to an existing building instead of
  standing on its own cells (S-0406, Terms, *upgrade type*). A building has three *upgrade
  positions*, and S-0406 (claim 12) owns how an upgrade is recorded in them, the cap on a building's
  upgrade count and what happens to a wall tower's count past three (*wall tower* and the game-wide
  wall tower type are S-0406's terms). In this spec the upgrades a
  building *holds* are the upgrade types recorded in its three positions, and only they count in its
  output and drain (claims 1 and 2) while the building's upgrade count is positive; when that count
  is 0, none of the recorded upgrades contributes output or drain. An upgrade of a wall tower that
  S-0406 does not record in one of the three positions adds no output and no drain. While the host
  remains in play, an upgrade is lost only by being sold or stripped. S-0408 (claim 5) owns the
  count-based selection: a sale selects the position numbered by the current count, not the
  highest occupied position. For contiguous upgrades this is the most recently added one.
  Selling a turret upgrade resets the count to 0 but clears only the sold position; lower recorded
  upgrades remain, contribute no power, and do not prevent the next sale from selling the host
  (S-0408 claim 5; *turret upgrade* here means an upgrade whose type has that claim's turret flag).
- **Needs-power flag, special-power flag, needs-engineer flag.** Boolean fields of a building type
  (Import mapping). The needs-power flag marks a building that stops working under low power
  (claim 8); the special-power flag marks one that stops working during a blackout; the
  needs-engineer flag marks one that works only when it is *manned* (glossary; S-0515 claim 2).
  Manned is permanent from the building's first ownership change of any cause, not physical
  engineer presence. Placement for a house does not man a building.
- **In operation.** Whether a building currently works as a powered building: claim 8 gives the test.
  It is derived each time it is asked; it is not the power switch. Rules that ask the question
  (firing, claim 9) read the answer as it stands at that moment; the one-off effects of claim 9 and the
  count of active control centres follow it in the building's own turn, except for the immediate
  power-switch effects of claim 10.
- **Under a pulse.** The state of an object between the start of an electromagnetic pulse that
  disables it and the moment the pulse's time runs out (S-0203, row 14). The combat and superweapon
  specs own when a pulse starts and how long it lasts. It is one state; this spec and the radar rule
  (S-0410) use this term for it. It is not *being erased*, which is a separate state of the same
  kind of object.
- **Super-gap.** The second, larger gap of a building whose type's super-gap radius is not 0, which
  the building's deploy behaviour switches on and off in place of its normal gap (claim 16). The
  normal gap and the gap field are S-0409's (claim 11); the power consequence of the super-gap is
  this spec's.
- **Controlled unit, held unit.** A unit that a building has taken over with a mind-control weapon
  is one of the building's *controlled units*. Its *held unit* is its *gripped unit* under
  S-0607 (Terms), not every unit that remains seized after release. S-0607 claims 12–13 own
  the end of the grip and the resulting drop or set-down.
- **Control-centre type, powered-unit type.** A building type can name one vehicle type whose units
  depend on it (its *powered unit*); a vehicle type can carry the powered-unit flag (Import
  mapping). A building of a control-centre type is an *active control centre* from the first
  turn of its own in which it is in operation until the first turn of its own in which it is not,
  or until it is lost (removed, destroyed or changing owner). So between a building's loss of
  operation (claim 8) and its next turn it still counts as active, and claim 13 counts exactly
  these buildings.

## Behaviour

### Output and drain

1. **Building output.** A building's output is 0 when it is switched off or being erased.
   Otherwise its output is formed from its *raw output*: the type's base output, plus, for an
   absorbing building whose type's extra output is above 0, the extra output times
   *n*<sub>a</sub>, plus the base output of each upgrade it holds when its upgrade count is positive
   (Terms, *upgrade positions*). When its upgrade count is 0, no recorded upgrade adds output.
   With a positive count, every recorded position contributes, including positions numbered above
   the count; the count is a gate, not a limit on the positions counted (Q-0224). The
   building makes trunc(raw output × health ratio) (Formulas F1), so a raw output of 0 gives 0. A
   damaged power plant therefore makes less power in proportion to its health, rounded down, and a
   power plant at 0 health makes none. The extra output of a building that is not absorbing adds
   nothing; nothing in the original adds it. A pulse does not change the output: a power plant
   under a pulse is not in operation (claim 8) but keeps making its full output.
   - Backing: `measurable` (scenario W1, parts a to d, f and g; W7 for switched-off output,
     W12 for upgrades, parts f and g for Q-0224); `unmeasured` for output at health 0 before removal (W1 tests health 1,
     and destruction normally removes the plant before a separate total can be read).
   - Confidence: high (the formula, the switch, no effect of a pulse), medium (upgrades, absorbed
     passengers, the extra output of other buildings, being erased)

2. **Building drain.** A building's drain is 0 when it is switched off or being erased. Otherwise
   it is the type's base drain, plus the base drain of each upgrade it holds when its upgrade count
   is positive (Terms, *upgrade positions*), plus the type's extra drain while the building's
   super-gap is active (claim 16). When its upgrade count is 0, no recorded upgrade adds drain.
   With a positive count, every recorded position contributes, including positions numbered above
   the count (Q-0224).
   Health does not change the drain: a damaged building needs as much power as an undamaged one. A pulse does not change it
   either.
   - Backing: `measurable` (scenario W1, parts e, f and g; W7 for switching; W12 for upgrades, parts f and g for Q-0224).
     The super-gap extra drain is `unmeasured` for claim 16's reason.
   - Confidence: high (base drain, no health scaling, the switch), medium (upgrades, being erased)

3. **Which buildings count.** Every building of the house that is on the map is counted, whatever
   its state: during its construction state (both build-up and selling), at any health, and whether
   or not it is in operation. A power plant therefore adds its output from the moment it is placed,
   before its build-up has finished, and keeps adding it while it is being sold, until it is
   removed (S-0408). A building off the map (for example one not yet placed) is not counted. A building that
   leaves or enters the map while it exists counts as *removed* or *created on the map* for the
   event list of claim 7; no skirmish case of this is known, so that part has no scenario. In a
   campaign mission, power counting is outside this spec's scope.
   - Backing: `measurable` (scenario W2); `unmeasured` for a building that leaves or enters the map
     while it exists (no way to cause it in a skirmish is known)
   - Confidence: high (on the map), medium (build-up and selling counted), low (leaving or
     entering the map)

4. **House totals, low power and the power fraction.** When a house works out its power (claim 7),
   it sums the outputs and drains of its counted buildings into *O* and *D*, applies claim 5, and
   from then until the next time uses these values for every rule that asks for its power. The
   house has low power when *D* > 0 and *O* < *D*; its power fraction is *O* / *D* then (0 when
   *O* = 0), and 1 otherwise (Formulas F3). A house with no drain has full power whatever its
   output, including 0.
   - Backing: `measurable` (scenario W3)
   - Confidence: high

5. **Blackouts.** While a house's timed blackout runs (claim 6), or while one of its counted
   buildings whose output (claim 1) is above 0 is being drained, its output *O* is 0 when it works
   out its power: if the house has any drain, it then has low power and a power fraction of 0. One
   drained power plant therefore cuts the whole house's power, not only its own share. The drain
   is unchanged during a blackout.
   - Backing: `measurable` (scenario W4, parts a and b)
   - Confidence: high (timed blackout), medium (drain blackout)

6. **Timed blackout length.** A timed blackout is started for a house with a length of *d* ticks:
   - when a spy of another house enters one of its buildings and the entry selects the blackout
     effect of S-0510 claim 3, row E2, *d* is the game-wide spy blackout length (Import mapping).
     In the default single-effect rule, the type must have base output above 0 and no radar flag:
     a radar-flag type selects re-cover instead, even if it also produces power. Alliance is not
     tested for the effect: an allied spy that reaches entry has the same effect. S-0510 claims
     1 and 3 own entry, its conditional allied case and single-effect precedence, including the
     quirk switch's alternative,
   - when the house launches a force shield, *d* is the force-shield blackout length
     (S-0513 claim 30; that spec owns its import).

   Starting a blackout, whatever its length, makes the house due (claim 7). Let *t*₀ be the tick of
   the start.
   - **Length above 0.** The blackout is *running* from its start until the beginning of the house
     update of tick *t*₀ + *d* − 1, where it ends and the house works out its power again. If the
     start came after that update (possible only for *d* = 1, where the update in question is tick
     *t*₀'s own), it runs to the end of tick *t*₀. A rule that reads a running blackout live (claim 8,
     the no-blackout row) sees it throughout.
   - **Output.** The house's output counts as 0 in every house update that takes place while the
     blackout is running: those of the ticks from *t*₀ (if the start came before the house's update
     in that tick) or *t*₀ + 1 (if after) to *t*₀ + *d* − 2. A length of 1 therefore gives no house
     update with a blackout in force.
   - **Length 0 or less.** No blackout runs: the start only makes the house due, and the live row of
     claim 8 never sees one. A negative length behaves as 0.
   - **A start while another blackout runs.** The new start replaces the running one: the end is
     computed from the new start tick and length, and the lengths do not add. With a length of 0 or
     less the replacement ends the running blackout at once, but the house's totals follow in its
     next update. A spy always starts with the game-wide length, so only a superweapon effect (whose
     length can differ) can start a different length during a running blackout.
   - Backing: `measurable` (scenario W4, parts a, c, d and e; S-0510 scenarios I2 and I5 for
     effect precedence and conditional allied entry); `unmeasured` for the live
     in-operation effect of a length of 1 (W4 records house totals only; no scenario here separates
     the start from the building's next turn), and for a negative length (W4 tests absence, not a
     negative value), for a start after the house update (no spy-entry input path is established
     at that point of a tick), and for replacement by a length of 0 or less (no skirmish setup
     with two different blackout lengths is established here; the spy length is game-wide and
     S-0513 scenario SW11 part c tests a single force-shield launch, not a zero-length
     replacement). S-0513 scenario SW11 part c also makes the force-shield start and its
     blackout length `measurable`.
   - Confidence: medium (the end at *t*₀ + *d* − 1, the restart, the length of 1), low (the exact
     tick of the first house update in which the output counts as 0, the live reading for *d* = 1,
     the replacement by a length of 0 or less)

7. **When power is worked out again.** A house works out its power only in its own house update
   (S-0203, claim 8), and only when an event since its last working-out has made it due. Its values
   are otherwise kept, so every rule in between sees the old totals. These events make it due:
   - a building of the house is placed, created on the map, removed or destroyed, changes owner,
     gains or loses an upgrade, or is switched on or off,
   - the health of one of the house's buildings changes: the building notices the change in its own
     turn, and the house works out its power in its update of the same tick,
   - a passenger enters or is voluntarily released from an absorbing building of the house
     whose type's extra output is above 0 (S-0605 claims 7 and 13),
   - a draining unit attaches to or lets go of one of its buildings,
   - an erasing weapon successfully locks on to one of its buildings, or the last erasing
     attacker releases it before completion (S-0514 claims 4 and 10). Release restores the
     building's contributions according to its resulting switch state (claims 1, 2 and 8).
     The house totals wait for the house update, rather than changing inside the hit or release,
   - a timed blackout starts or ends (claim 6),
   - a super-gap is switched on or off (claim 16).

   Because house updates come after the turn pass and orders after the house updates (S-0203,
   claims 7 and 8), a change that happens during a turn takes effect for rules that ask during the
   next tick's turns, and a power order given in tick *n* changes the totals in the house update of
   tick *n* + 1. When the house works out its power, each
   building's output uses the building's health as it is at that moment, not the health the
   building last noticed (F1). Damage that drops a power plant's health in tick *n* therefore
   lowers the house's output in its update of tick *n* if the house is due then for any reason: the
   plant took its turn after the damage in tick *n*, or some other event of the list made the house
   due. Otherwise the plant notices the drop in its turn of tick *n* + 1 and the output falls in the
   update of tick *n* + 1. Surrender or defeat destruction during the house's own update
   happens after that update's power evaluation: it makes power due for the next house update,
   without changing the totals already published in the current one (S-0414 claim 16).
   A later evaluation still counts buildings by claim 3, including those awaiting removal.
   Whether removal of a passenger while it remains inside a surviving absorbing building
   makes power due is unresolved (S-0605 claim 15); it is not an established event of this list.
   - Backing: `measurable` (S-0414 scenario V12 for house-update destruction,
     scenario W5, parts a to c for health and switch timing; W1, W2, W4
     and W12 for the resulting totals after erasure, passengers, placement, removal, blackout,
     draining and upgrades); `unmeasured` for owner-change timing and the exact recheck timing
     after passengers, erasure, draining and upgrades (those scenarios read settled totals, and
     no tick-level scenario for these causes is specified here), and for super-gap rechecks
     (no skirmish input path is established, claim 16).
   - Confidence: high (the house update, health changes, live health in the sum), medium (the full
     event list; the lag of at most one tick)

### Which buildings stop

8. **The in-operation test.** A building is in operation when all of these hold:

   | Condition | Fails when |
   |---|---|
   | switched on, or charged | it is switched off and *n*<sub>c</sub> < 2 |
   | not under a pulse | it is under a pulse |
   | alive | its health is 0 |
   | powered | its type has the needs-power flag and a base drain above 0, its house has low power, and *n*<sub>c</sub> < 2 |
   | no blackout | its type has the special-power flag and its house has a blackout in force (a timed blackout running, or a drain blackout at its last working-out) |
   | manned when required | its type has the needs-engineer flag and it is not manned (S-0515 claim 2) |
   | not building or selling | its current behaviour is *construction* or *selling* |

   A needs-engineer building placed for a house is never in operation until its first ownership
   change; own-engineer entry does not man it (S-0515 claim 2).
   The last row reads the building's current behaviour only, not its queued behaviour. A building
   whose sale has been ordered is in operation until the selling behaviour becomes its current
   behaviour (at the sale's first processing, S-0408 claim 6) and is not in operation from then on,
   including while the build-down waits for docked units. The state of the build-up or build-down
   sprite does not enter the test. An erasing lock switches the building off (S-0514 claim 4),
   so it fails the switch condition unless it has two or more chargers; with those chargers it
   must still pass every other condition. Being erased is not a separate condition of this
   test; its output and drain are nevertheless 0 (claims 1 and 2). When the last attacker lets
   go before completion, the building is switched on regardless of its pre-lock state
   (S-0514 claim 10). S-0514 owns `quirks.erasure_release_switches_on`, default original.
   With it off, release restores the pre-lock switch state. The power condition reads the
   house's totals as they stand (claim 7). Two consequences: a
   building of a type with the needs-power flag but no base drain never stops for low power; and a
   building with two or more chargers passes the switch and power conditions even under low
   power or when switched off, but must still pass every other condition. The test is the same for every house; no rule of it depends on whether the owner is human-
   or computer-controlled.
   - Backing: `measurable` (scenario W6, parts a to f and j; S-0515 scenarios TB3 and TB8 for
     manned state; scenario W4, part a for the blackout row; S-0514 scenario E7 for erasure's
     switch changes); `unmeasured` for operation with two or more chargers while being erased
     (erasure has disabling effects of its own, so no scenario here separates the operation
     result from those effects); `unmeasured` for the health-0 row (no scenario here keeps a destroyed building
     available to test its operation separately), and for the distinction between queued and
     current construction or selling behaviour (W6 tests current behaviour only).
   - Confidence: high (switch, low power, construction and selling behaviours), medium (chargers,
     pulse, blackout, manned state, the charger exception during erasure); high (erasure's
     switch changes)

9. **What stops when a building is not in operation.** A building that is not in operation:
   - cannot fire: its firing check (S-0301, claim 9) answers *impossible* or *illegal*, as T1
     below gives,
   - stops being an active control centre (claim 13), stops its gap (claim 16), and produces no
     credits from a cash-producing type (S-0515 claims 5 to 7),
   - releases every controlled unit it holds (S-0601 claims 9–10 own the release effects and
     release order) and lets go of its held unit (S-0607 claims 12–13): at 1 lepton or more
     above the surface the unit is dropped; at height 0 or less it is set down,
   - stops the effects of sensor types (visibility specs), and its lights and working animations
     (rendering specs).

   **T1, a building's own answers in the firing check.** The three rows do not overlap, so a
   building has exactly one own answer at any moment.

   | Answer | When |
   |---|---|
   | *illegal* | the building's type lets infantry occupy it (S-0215) and either it has no occupant or its type does not let occupants fire (glossary, *garrisoned building*); or it is being drained; or its current behaviour is *construction* or *selling* and its type is not a pulse cannon (S-0214's type flag) |
   | *impossible* | no condition of the *illegal* row holds, and its type is a pulse cannon or it is not in operation (claim 8) |
   | none of these (S-0301's other answers apply) | neither row above applies |

   An occupiable building that has occupants who can fire is judged by the other conditions. A
   building in its *construction* or *selling* behaviour answers *illegal*, not *impossible*,
   although it is also not in operation (claim 8), unless its type is a pulse cannon. A pulse cannon
   answers *impossible* in every state except when it is being drained or is occupiable without
   occupants who can fire. S-0301 (claim 9) gives these answers its rank 4, leaves open which of them
   applies when several of their conditions hold, and does not list the building cases; T1 settles
   the building cases and is owned here.

   Factories do not stop for low power: production only slows (S-0403, claim 5). What switching a
   factory off does to production is S-0404's (claim 8). The original's being in operation also
   gates further building functions that the specs of those functions own.

   Firing reads the test whenever it is asked. The control centre, gap, release of controlled
   and held units, lights and animations follow the in-operation result in the building's own
   turn (S-0203), except for the immediate switch effects in claim 10. The first such turn after
   the result changes applies the new operation state. This also applies when build-up ends or
   selling begins. These effects can be reconciled on a turn even without an in-operation
   transition: for example, a charged gap building whose switch-off stopped its field but left
   it in operation can generate again in its next turn. A building that loses power by the house
   update of tick *n* releases its controlled units in its turn of tick *n* + 1. The charged-gap
   switch case is `unmeasured` for claim 10's reason.
   - Backing: `measurable` (scenario W6, parts a, h, i, j and k, which cover firing, the release of a
     controlled unit, the working animation, the effects at the end of the build-up and the gap),
     `unmeasured` for T1's distinction between *illegal* and *impossible*, including where both
     sets of conditions hold (W6 observes firing refusal, not an answer label; no differing
     building response to these labels is known, and S-0301 claim 9 makes the same argument).
     `measurable` for cash production (S-0515 scenario TB3, including switched-off and unmanned
     cases). Held-unit release inherits S-0607 claims 12–13's backing (scenario L8 for a
     building losing operation). `unmeasured` for the lights and the sensor types (each needs the
     observable of the spec that owns the effect, and those specs' scenarios should add a variant
     in which the building is switched off)
   - Confidence: high (firing under low power, the one-off effects' timing for the animation and the
     controlled unit), medium (T1 where only one of its conditions holds, the gap, the other items),
     low (T1's answer where the conditions of both its rows hold)

10. **The power switch.** A power order applies only to a live building on the map whose type
    does not have the firestorm-wall flag (S-0213, claim 13 and its Edge cases define the flag).
    An applicable power-off order switches an on building off only when its type has a base drain
    above 0 or the needs-power flag; otherwise it changes nothing. An applicable power-on order
    switches an off building on only when it is not under a pulse. Repeating an order for the
    state already set changes nothing. A switched-off building makes no output and needs no drain
    (claims 1 and 2) and is not in operation unless charged and the other conditions hold (claim 8).
    Which buildings the player may click in power mode is S-0214's (claim 13).

    An accepted switch-off stops the building's lights and power-dependent working animations
    immediately, without waiting for its next turn; an accepted switch-on starts them immediately.
    An accepted switch-off also ends its gap field immediately; switching on lets the field
    start again in the next building turn that finds it in operation.
    These switch effects do not wait for the in-operation transition of claim 9 and do not test
    its other conditions. A later in-operation transition can change them again.
    - Backing: `measurable` (scenario W7, parts a to d for switching and animations; S-0213
      scenario O12 for the firestorm-wall exclusion); `unmeasured` for orders to buildings dead
      or off the map (input cannot target them), an order repeating the existing switch state
      (input produces the opposite order), a power-off order to a type with neither drain nor
      needs-power flag (input produces no order for it), and the immediate light effect (W7
      specifies an animation, not a light-equipped test type), and for the gap's switch-off
      timing and its restoration while charged (W6 tests a house-power change, not these
      switch-only cases). S-0409 claim 11 already states the immediate switch-off exception and the next-building-turn restoration rule; this claim is consistent with that rule and does not change the gap's shape or cell effects.
    - Confidence: high (the drain-or-flag condition and order eligibility), medium (the pulse
      condition, immediate lights and animations)

11. **Production under low power.** Low power slows every production line of the house by the
    factor that S-0403 (claim 5) derives from the power fraction *p*. When a house works out its
    power, the step period of each of its lines is worked out again at once (S-0403, claim 8).
    - Backing: `measurable` (S-0403 scenarios B4 for slowdown and B7 for re-evaluation)
    - Confidence: high

12. **Radar.** Low power is one input of the radar rule: before a house's defeat, and unless the
    map grants free radar, radar requires full power (claim 4). S-0410 (claim 1 and Edge cases and
    quirks) owns the whole rule and `quirks.radar_first_building_decides` (default: the original),
    including the radar outage, free radar, which radar building decides, what makes it qualify,
    the switch's alternative, and radar after defeat. S-0410 (claim 2) owns when availability is
    evaluated again. The radar rule does not use the in-operation test of claim 8.
    - Backing: `measurable` (S-0410 scenario R1)
    - Confidence: high (low power as an input)

13. **Units that depend on a control centre.** A house's active control centres are counted
    together, whatever their types. When the count goes from 1 to 0 (the last active control centre
    stops being in operation, or is lost), every vehicle of the house whose type is the powered unit
    of the building type that just stopped, and that is not off the map, is disabled; when the count
    goes from 0 to 1, every vehicle of the house whose type is the powered unit of the building that
    just came into operation is enabled again. A vehicle whose type has the powered-unit flag and
    that enters a cell while its house has no active control centre is disabled too, unless the cell
    holds a building (a vehicle in a factory or on a repair depot stays enabled). A disabled vehicle
    is inactive until enabled, but disabling does not imply immediate rest: a hovering vehicle can
    still spin, settle and retain residual travel, and over water can fall and be destroyed.
    S-0503 claims 8 and 10–11 own these hover effects and their conditions, including restoration
    of lift on re-enabling. Because one count serves all control-centre types, a
    house with two control-centre types that power different vehicle types disables neither kind
    while either centre runs (`quirks.shared_control_centre_count`, Edge cases).
    The cell-entry rule reads the vehicle type's powered-unit flag, while enabling and disabling at
    the count's transitions read only the control centre's powered unit; the two need not agree
    (Edge cases).
    - Backing: `measurable` (scenario W8, parts a to i; part h for off-map vehicles at disabling).
      Hover effects inherit the backing and confidence of S-0503 claims 8 and 10–11, with
      scenario H7 there testing ordinary disabling, settling, spin, residual travel and water loss.
      `unmeasured` for off-map vehicles at enabling and for loss or owner-change transitions
      (W8 tests disabling off-map and switch transitions only; no scenario part covers these
      other cases).
    - Confidence: medium (the count and its two transitions), low (the cell-entry rule and its
      building exception)

14. **Spy satellite.** The map reveal of a spy satellite (S-0409 owns it) does not depend on
    power: neither low power, nor a blackout, nor switching the building off stops it, whether or
    not its type has the needs-power flag. S-0409 (claim 15) owns the rule.
    - Backing: `measurable` (scenario W9, parts a to d)
    - Confidence: medium

15. **Superweapon charge.** S-0512 claims 4 and 6 own suspension and resumption. At its
    availability checks, an ordinary building-granted, power-dependent superweapon that still
    has a giving building is suspended when the house has low power (using claim 7's last totals)
    or none of its giving buildings
    is switched on. Normally its charge clock stops, keeping the remaining time, and resumes
    from that time when the house has full power and at least one giving building is switched on.
    Giving buildings are those of S-0512 claim 6; their being in operation (claim 8) is not the test.

    Initial grants differ: under low power a building-granted record starts suspended even when
    its type is not power-dependent. At full power a power-dependent record granted with all
    giving buildings switched off starts charging until the next availability check. S-0512
    claim 4 owns these exceptions and their quirk switch. S-0512 claims 10 and 14 own the
    second-stage firing and charge-setting exceptions that can restart a clock while suspension
    remains: time then runs through suspension, but clock completion cannot make the record
    ready until suspension ends. Their quirk switch and the one-time and permanent-trigger grant
    exceptions are owned by S-0512 claims 10, 12–14.
    - Backing: `measurable` (W10; S-0512 scenarios X2 and X3 for suspension, remaining time
      and initial grants; running-clock exceptions inherit S-0512 claims 10 and 14's backing)
    - Confidence: high (ordinary suspension and initial grants); the running-clock exceptions
      inherit S-0512 claims 10 and 14's confidence

16. **The super-gap and its extra drain.** A building whose type's super-gap radius is not 0 (Import
    mapping) can be switched between its normal gap and its super-gap by its deploy behaviour. It
    switches to the super-gap only when it is in operation. While the super-gap is active, the gap
    field has the super-gap radius (S-0409 owns the field's shape) and the type's extra drain is
    added to the building's drain (claim 2); switching back, or the building ceasing to be in
    operation, ends the super-gap and removes the extra drain, and the house's power is due again
    (claim 7). Whether input can give that deploy behaviour to a building is not established (Open
    questions).
    - Backing: `unmeasured` (no skirmish input path to the building's deploy behaviour is
      established; W11 is a conditional proposal, not backing). W6 part k measures the normal
      gap stopping, not a deployed super-gap.
    - Confidence: low

17. **All houses alike.** No rule of this spec depends on whether a player house is human- or
    computer-controlled, so AI houses follow every claim as written. The neutral and special houses
    follow them too. The original's computer player has no power rule of its own in a skirmish.
    - Backing: `unmeasured` (a statement that no claim above has an exception by owner; each claim's
      scenario uses a human house, and the absence of an exception cannot be shown by one scenario)
    - Confidence: medium

18. **Map-requested power state (Q-0180).** A pre-placed building starts switched on when the
    map's switched-on flag is set or absent. A flag of 0 starts it switched off only when its
    type has a base drain above 0 or the needs-power flag; otherwise it stays switched on.
    Thus a pure power-producing type with neither condition still makes power despite a flag
    of 0. Unlike power orders, this initial-state rule does not exclude the firestorm-wall
    flag: a firestorm-wall type meeting either condition starts switched off with a map flag
    of 0, and one meeting neither starts switched on. The power-mode flag does not decide
    this initial state. This rule is consistent with S-0009 claim 40, which gives the same conditional startup rule.
    S-0009 owns the map field's encoding.

    Later power orders follow claim 10 without an exception for map placement. A non-firestorm
    building that started switched off can be switched on by an applicable power-on order
    when it is not under a pulse. A firestorm-wall building cannot be switched on by such an
    order. Input must still offer the order under S-0214 claim 13. A pure producer that stayed
    on at placement has no off state to restore.
    - Backing: `measurable` (W13 for start states and subsequent input; W7 for the pulse gate)
    - Confidence: high

19. **Meaning of the power meter (Q-0276).** The local player's meter represents the
    house totals *O* and *D* of claim 4, not the power fraction alone. Its occupied extent
    represents max(*O*, *D*) when that maximum is positive. Within that extent, the supplied part represents *O*, the
    near-shortage part represents the first 100 power units of unmet drain, and the
    further-shortage part represents any remaining unmet drain (F4). With full power
    only the supplied part remains; unused output is not a separate shortage part.
    With both totals 0, the meter uses only the supplied part as an empty-house fallback
    (F4); this does not mean that the house generates positive power.
    Thus the distinction is an absolute shortage of 100 power units, not a percentage
    threshold. These are semantic parts, not required colours or artwork.

    The meter expands as the base grows. Its scale uses the sum *B* of base output plus
    base drain of the house's buildings, without health scaling, upgrade contributions,
    passenger contributions, switching or blackout suppression. A damaged or switched-off
    plant therefore changes the totals represented but does not itself shrink *B*.
    F4 gives the scale and the proportions. This display calculation changes no game state.

    Layout, orientation, colours, artwork, display resolution and transition animation
    remain implementer design under S-0411's interface scope. F4 specifies settled
    meaning, not a pixel reproduction or an animation deadline. In particular, do not
    replace this meter with a drain/output fraction that discards the absolute shortage
    threshold and base-dependent occupied extent.
    - Backing: `measurable` (W14)
    - Confidence: high (shortage split and scale), medium (settled display interpretation)

## Formulas

All results in power units are integers (S-0205, claim 1). The formulas display RZ₅₃ for
`quirks.chop_rounding` on (the original); with it off, replace every RZ₅₃ below by RN₅₃
(S-0205). Trunc is S-0205's conversion toward zero.

**F1, building output** (claim 1). *P* is the raw output (integer), *H* the building's health at the moment the
house works out its power (not the health it last noticed, claim 7) and *S* the strength
(integers, *S* > 0):

> output = trunc(RZ₅₃(RZ₅₃(*H* / *S*) × *P*)) when the building is switched on and not being
> erased; 0 otherwise. *P* ≥ 0, so *P* = 0 gives 0. Unit: power units.

The health ratio is formed as one double division and the product as one double multiplication,
in floating point in the original. Example (invented values): *P* = 120, *H* = 333, *S* = 500:
*H* / *S* = 0.666 (RZ₅₃ of it is just below 0.666), times 120 is just below 79.92, so the output is 79.

> *P* = base output + passenger contribution + Σ base output of the upgrades recorded in the building's three upgrade positions when its upgrade count is positive (Terms, *upgrade positions*). When its upgrade count is 0, the upgrade sum is 0. Integer arithmetic, exact. The passenger contribution is extra output × *n*<sub>a</sub> when the building is absorbing and its extra output is above 0; otherwise it is 0.

**F2, building drain** (claim 2):

> When the building is switched on and not being erased, drain = base drain + upgrade contribution + super-gap contribution; otherwise drain = 0. The upgrade contribution is Σ base drain of the upgrades in the three upgrade positions when the upgrade count is positive; it is 0 when the count is 0. Integer arithmetic, exact. The super-gap contribution is extra drain while the super-gap is active; otherwise it is 0.

**F3, power fraction** (claim 4):

> *p* = 1 when *D* = 0 or *O* ≥ *D*; *p* = 0 when *O* = 0 < *D*; otherwise *p* = RZ₅₃(*O* / *D*).
> Unit: none (a double from 0 to 1).

The division is one double division in floating point in the original. S-0403 narrows *p* to single
precision before it uses it.

**F4, settled meter scale and proportions** (claim 19). *N* > 1 is the chosen display
capacity in abstract bands, an integer design parameter, not a prescribed screen size.
*B* is the non-negative integer sum of the house's buildings' base output and base drain.
The occupied capacity *A*, in bands, is:

> *q* = trunc(RZ₅₃(RZ₅₃(400 / RZ₅₃(*B* + 400)) × *N*))
>
> *A* = *N* − min(*N* − 1, max(0, *q*))

The constant 400 has units of power units. The addition, division and multiplication in
*q* are floating-point operations in the original, with conversion toward zero at the
end. The clamps and subtraction defining *A* are integer operations. There is at least
one band of capacity even for *B* = 0.

Let *U* = max(0, *D* − *O*), *V* = min(*U*, 100), *W* = *U* − *V*, and
*T* = *O* + *V* + *W* = max(*O*, *D*), all exact integers in power units.
For *T* > 0, the three parts have unquantised target extents:

> supplied = RZ₅₃(*A* × RZ₅₃(*O* / *T*))
>
> near shortage = RZ₅₃(*A* × RZ₅₃(*V* / *T*))
>
> further shortage = RZ₅₃(*A* × RZ₅₃(*W* / *T*))

Division and multiplication are double floating-point operations. Extents are in bands.
With *T* = 0 the supplied extent is *A* and both shortage extents are 0, an empty-house
display fallback rather than a change to *O*. These are proportions before artwork quantisation:
rounding these extents to drawn marks, including distribution of residual fractions,
is implementer design, not a further game-state rule. No formula here prescribes pixels.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Meter scale constant | 400 | power units | F4 |
| Near-shortage interval | 100 | power units | Claim 19, F4 |
| Chargers that keep a building in operation | 2 | infantry units | Claim 8 |
| Upgrade positions whose upgrades count in power | 3 | positions | Terms; the positions and the cap on the upgrade count are S-0406's (claim 12) |
| Blackout end, ticks before *t*₀ + *d* | 1 | ticks | Claim 6: the blackout ends in the house update of tick *t*₀ + *d* − 1 |
| Spy blackout length, absent-key default | 0 | ticks | Claim 6; a length of 0 or less starts no blackout |
| Power field, absent-key default | 0 | power units | Terms |
| Needs-power, special-power, needs-engineer flags, absent-key default | no | boolean | Claim 8 |

## State and ordering

| State | Event | Result |
|---|---|---|
| house power not due | building placed, created on the map, removed, destroyed or changes owner; a building's health changes (damage or repair, noticed in its own turn); building switched on or off; upgrade gained or lost; passenger enters or is voluntarily released from an absorbing building whose type's extra output is above 0; draining unit attaches or lets go; erasing weapon successfully locks on or its last attacker releases before completion; timed blackout starts or ends; super-gap switched on or off (claim 7) | power due |
| power due | the house's update | *O*, *D* worked out (claims 4, 5); production periods worked out again (claim 11); a change of low-power state checks superweapon suspension and resumption under S-0512 claim 6, with claim 15's exceptions |
| full power | worked out with *D* > 0 and *O* < *D* | low power: needs-power buildings with a base drain above 0 and fewer than 2 chargers stop being in operation (claim 8) |
| low power | worked out with *O* ≥ *D* or *D* = 0 | full power: the power condition no longer prevents operation; claim 8's other conditions still apply |
| in operation | the building's current behaviour becomes *selling* (sale's first processing) | not in operation at once (claim 8); output and drain unchanged (claim 3) |
| no blackout | spy entry selects claim 6's blackout effect, at tick *t*₀, with *d* > 0 | timed blackout running until the house update of tick *t*₀ + *d* − 1 (claim 6) |
| no blackout | the same, with *d* ≤ 0 | house due; no blackout runs (claim 6) |
| not an active control centre | the building's first turn in which it is in operation | active control centre (Terms; claim 13) |
| active control centre | the building's first turn in which it is not in operation, or its loss | not active (Terms; claim 13) |
| switched on | applicable power-off order (live building on the map, type without the firestorm-wall flag), type with a base drain above 0 or the needs-power flag | switched off; building output and drain 0; house totals follow claim 7 |
| switched off | applicable power-on order (live building on the map, type without the firestorm-wall flag), no pulse | switched on |
| active control centres 1 | the last one stops | its powered-unit vehicles disabled (claim 13) |
| active control centres 0 | one comes into operation | its powered-unit vehicles enabled |

Observable ordering: a building's in-operation state uses the house totals as last worked out
(claim 7), so a defence that loses power by a building's death in tick *n* still fires in the turns
of tick *n* that come before the house's update, and stops in tick *n* + 1. Destruction
by that house update's surrender or defeat is later than its power evaluation: the old totals
remain through tick *n*, and the re-evaluation is in tick *n* + 1 (claim 7, S-0414 claim 16).

## Edge cases and quirks

- **The extra output of a building that does not absorb.** It adds nothing in the original (claim
  1), and a positive extra output on a type that does not absorb has no effect in our engine
  either. This is the original's rule, not a quirk, so it has no switch.
- **Negative extra power on an absorbing building.** It is an extra drain, which counts only during
  a super-gap (claim 16); the per-passenger rule needs an extra output above 0. Reproduced.
- **One count for all control centres** (`quirks.shared_control_centre_count`, default: the
  original). The original keeps one count of active control centres per house for all
  control-centre types, and acts only on the powered-unit type of the building whose change moves
  the count between 0 and 1 (claim 13). With two control-centre types that power different vehicle
  types, losing the only centre of one type while a centre of the other type runs disables nothing,
  and when the count later reaches 0 only the type of the last centre to stop is disabled. With
  the switch off, the count is kept per powered-unit type: a vehicle type is disabled when the
  house has no active control centre whose powered unit is that type, and enabled when it has one.
- **A powered-unit flag that no control centre names.** A vehicle type with the powered-unit flag
  that is no control-centre type's powered unit is disabled on entering a cell without a building
  while its house has no active control centre, and no centre coming into operation enables it
  again; a vehicle type that a control centre names but that lacks the flag is disabled and enabled
  only at the count's transitions. Reproduced; the importer warns for either mismatch. With
  `quirks.shared_control_centre_count` off, the cell-entry rule uses the per-type count of the
  vehicle's own type.
- **A needs-power type with no drain.** It never stops for low power (claim 8). Reproduced as the
  original's rule; the importer warns for a building type with the needs-power flag and no base
  drain, because the flag then does nothing.
- **Blackout length of 1 or less.** No house update sees the blackout (claim 6), and with a length
  of 0 or less none runs at all. A negative length behaves as 0. Reproduced; the importer warns for
  a negative spy blackout length.
- **A blackout one house update short** (`quirks.blackout_one_short`, default: the original). A
  blackout of length *d* counts as 0 output in *d* − 1 house updates at most (claim 6), so *d* = 1
  has none. With the switch off, a blackout of length *d* > 0 counts as 0 output in exactly the
  *d* house updates from the first one after its start, and a length of 0 or less still has none.
- **More than three upgrades.** S-0406 (claim 12) lets the upgrade count of a wall tower type grow
  past 3, but a building has three upgrade positions and the original's power reads only those
  three (Terms). A fourth or later upgrade therefore adds no output and no drain. This is the
  original's rule, not a quirk, so it has no switch. Any other effect of a fourth or later upgrade
  is S-0406's (Open questions).
- **Campaigns and network games.** Out of scope; no counting or timing rule here specifies them.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, building section, `Power` | base output (value ≥ 0) or base drain (−value, value < 0), for the host and each upgrade type; upgrade contributions use claims 1 and 2's count gate, not a position limit (Q-0224) | integer | 0; a layer without the key keeps the value in force |
| Rules, building section, `ExtraPower` | extra output (value ≥ 0) or extra drain (−value, value < 0) (Terms, claims 1, 2, 16) | integer | 0; keeps |
| Rules, building section, `InfantryAbsorb`, `UnitAbsorb` | absorbing building (claim 1; the passenger rules are S-0312's) | boolean | no |
| Rules, building section, `Powered` | needs-power flag (claim 8) | boolean | no |
| Rules, building section, `PoweredSpecial` | special-power flag (claim 8) | boolean | no |
| Rules, building section, `NeedsEngineer` | needs-engineer flag (claim 8) | boolean | no |
| Rules, building section, `TogglePower` | whether input offers power mode on the building (S-0214, claim 13) | boolean | yes |
| Rules, building section, `PowersUnit` | the control centre's powered unit (claim 13) | vehicle type name | none |
| Rules, vehicle section, `PoweredUnit` | powered-unit flag (claim 13) | boolean | no |
| Rules, building section, `Drainable` | whether a draining unit can attach (special-abilities specs) | boolean | no |
| Rules, building section, `SuperGapRadiusInCells` | super-gap radius (claim 16; this spec owns the key) | integer, cells | 0 |
| Rules, `[General]`, `SpyPowerBlackout` | spy blackout length *d* (claim 6) | integer, ticks | 0; keeps |
| Rules, `[General]`, `LowPowerPenaltyModifier`, `MinLowPowerProductionSpeed`, `MaxLowPowerProductionSpeed` | production slowdown (claim 11) | owned by S-0403 | S-0403 |
| Map, `[Structures]`, switched-on field of a building entry | request for initial power state, subject to claim 18, not an unconditional off state | comma-separated integer field, position and parsing owned by S-0009; 0 requests off, nonzero requests on | on |
| Rules, building section, `IsFirestormWall` | exclusion of later power orders (claim 10), not of map-requested off state (claim 18) | boolean, owned by S-0213 | no |

The radar flag and the free-radar map flag are mapped in S-0410, the spy satellite flag and the normal
gap radius in S-0409.

## Measurement scenarios

All scenarios run under the measurement setup, in a skirmish on a flat purpose-made test map, with
named test files where a scenario sets fields or defines test types. Power output and drain are
read from a recorder of the house's power values, or from the sidebar's power meter where it shows
them; health and credits from a recorder or the interface; ticks with a recorder of the tick
number. Any game speed, held fixed within a comparison. Director and unrelated mods are off.
Only the named test data for the scenario are enabled. One human player owns the test buildings.
One bystander computer player is kept outside the observed area (the test buildings and all
units acting on them), under the glossary's void rule. Enemy spies and other acting test units
are map-placed and execute the stated behaviours; scripted input controls the human player's
orders. The read-only recorder and scripted-input apparatus never inject orders or write state.
All numbers assigned to test types below are invented test inputs, not stock rule values.

**Runs.** Each scenario states its number of runs. A part that a tick-level timing decides is run
many times with the triggering event landing at different points of the tick, which the test map
arranges by varying the delay before the event (a unit's walking distance, a timer trigger); a
part that is a plain state read is run three times. A part passes only if every run agrees with the
expected result. A timing that is "±1" admits one tick either way and nothing more.

**Power states.** Where a scenario needs low power or full power, it uses a spare consumer: with
the spare consumer switched on the house has a drain above its output (low power), and with it
switched off the house has full power (W3's second pair).

**W1: building output and drain (claims 1 and 2).**
- *Setup:* test power plant type of base output 120 and strength 500; test consumer type of base
  drain 40 and strength 500. One of each, own, no other building with power except, in part (f),
  the spare consumer (Power states, above). For part (g), a
  weapon of the erasing kind that the combat specs own (S-0203, Terms).
- *Procedure:* (a) read *O* at full health. (b) Damage the plant to health 333 and read *O*.
  (c) Damage it to health 1 and read *O*.
  (d) In one run, only an absorbing test type with base output 0, extra output 30 and two
  infantry passengers inside; in a second run, only a non-absorbing test type with base output 100
  and extra output 30 (both read alone, without the plant of (a)). (e) Damage the consumer to
  health 100 and read *D*.
  (f) Disable the undamaged plant and consumer with a pulse; then, while both are under the
  pulse, switch a spare consumer off (which makes the house's power due, claim 7) and read *O*
  and *D*.
  (g) In one run the undamaged plant, in a second run the undamaged consumer, is locked on by the
  erasing weapon; read *O* (first run) or *D* (second run) while it is locked on, then after the
  weapon lets go.
- *Runs:* three per part, except (g): three per run kind.
- *Observable:* *O* and *D* after each step.
- *Expected:* (a) 120; (b) 79 (trunc of just below 79.92); (c) 0 (trunc of 0.24); (d) 60, and 100
  for the non-absorbing type; (e) *D* stays 40; (f) *O* stays 120 and the pulsed consumer's
  drain stays 40; (g) 0 while locked on, and 120
  (first run) or 40 (second run) after the release.
- *Falsified if:* any recorded output or drain differs from the expected value in its part,
  including a changed drain after damage, changed output under a pulse, or nonzero output or
  drain while locked on.

**W2: which buildings count (claim 3).**
- *Setup:* a test power plant type with a long build-up time and a test consumer, own.
- *Procedure:* place it and read *O* during its build-up; later sell it and read *O* during the
  selling animation and after it is removed.
- *Runs:* three.
- *Observable:* *O* per tick.
- *Expected:* its output counts from placement and through the sale, and is gone after removal.
- *Falsified if:* output is absent during build-up or sale, or remains after the removal's
  next house update.

**W3: low power and the power fraction (claim 4).**
- *Setup:* test plants and test consumers giving (*O*, *D*) = (100, 100), (99, 100), (50, 100) and
  (0, 100), the drain of 100 including a test defence with the needs-power flag and a drain of 10.
  Two further runs without the defence or any consumer: (*O*, *D*) = (100, 0) and (0, 0).
- *Procedure:* an enemy unit stands in the defence's range; in every run one item is produced.
- *Runs:* three per pair.
- *Observable:* whether the defence fires; the build time of the item (S-0403 scenario B4).
  Also record the power fraction as a double with the read-only recorder, without narrowing it.
- *Expected:* the defence fires at (100, 100) and not at the three others; the build time is the
  full-power time at (100, 100), (100, 0) and (0, 0). The recorded fractions are respectively
  1, RZ₅₃(99 / 100), 0.5, 0, 1 and 1, formed as F3 specifies.
- *Falsified if:* firing, the recorded fraction or production time differs from its expected result; for the
  low-power pairs, the production time must match S-0403 scenario B4's computed period.

**W4: blackouts (claims 5, 6 and 8).**
- *Setup:* own house with two test plants (base output 100 each, neither with the radar flag), a consumer (drain 100) and the
  defence of W3; spy blackout length 300 in a named test file (parts d and e change it as stated),
  an enemy house with spies and a draining unit; a test type C with the special-power flag and a
  working animation, and no needs-power flag.
- *Procedure:* (a) an enemy spy enters one plant at tick *t*₀ (recorded). (b) In a second run a
  draining unit attaches to one plant, and later lets go. (c) In a third run a second spy enters
  the other plant 100 ticks after the first. (d) With the spy blackout length 1, a spy enters a
  plant. (e) With the spy blackout length absent, a spy enters a plant.
- *Runs:* (a) and (c) twenty each, varying walking distance and recording the entry tick.
  Parts (b), (d) and (e): three each. Do not assume these inputs can cause an entry after the house
  update; that case remains unmeasured.
- *Observable:* end-of-tick *O* and *D*, and shot ticks separately (shots in a tick read the
  previous house totals). In (a), also C's working animation. A C-type building without the
  special-power flag provides the animation control in a separate run.
- *Expected:* (a) end-of-tick *O* is 0 from *t*₀ or *t*₀ + 1 through *t*₀ + 298, and is 200
  again at *t*₀ + 299; fewer than 300 house updates publish output 0. C's animation stops
  during the blackout, but the unflagged control's does not. (b) *O* is 0 after the attachment's
  recheck, and 200 again after the release's recheck. *D* stays at its pre-blackout value in
  both (a) and (b). (c) Output returns at *t*₀ + 399, not *t*₀ + 599. (d) and (e) No house
  update publishes output 0.
- *Falsified if:* any recorded total, restoration tick or animation result differs from its
  expected result. In particular, an output of 100 during (a) or (b) would mean only one plant
  lost output; any zero-output house update in (d) or (e) falsifies the short-length rule.

**W5: when power is worked out (claim 7).**
- *Setup:* own defence with the needs-power flag and a weapon that is ready to fire (target in
  range, reload period short against the time since its last shot), a consumer, a spare consumer
  of drain 1, and a plant whose output just covers the total drain; an enemy target in the
  defence's range that it is attacking. The damage in (a) and (c) is large enough to take the
  plant's output well below the remaining drain.
- *Procedure:* (a) damage the plant at a recorded tick *n*. (b) In a second run start at low power
  and switch the consumer off with a power order at tick *n*, so that full power returns. (c) Repeat
  (a) with the hit landing at different points of the tick, and in every repeat also
  switch the spare consumer off with a power order at tick *n* − 1 (which makes the house due in
  the update of tick *n*).
- *Runs:* (a) twenty, with the hit landing at different points of the tick; (b) ten; (c) twenty.
- *Observable:* end-of-tick *O* and *D*, the defence's shot ticks and its readiness to fire.
  Timing of the totals is decisive even when rearming conceals the first restored shot.
- *Expected:* (a) output falls in the update of *n* or *n* + 1, never later; no shot at
  *n* + 2 or later. (b) Totals remain low-power at the end of *n*, become full-power in the
  update of *n* + 1. No shot occurs before *n* + 2; it fires at *n* + 2 if ready then,
  otherwise on the first later tick when all firing conditions allow it. (c) Output falls in
  the update of *n* in every repeat; no shot at *n* + 1.
- *Falsified if:* any recorded total changes outside these updates, or a shot occurs outside
  the expected firing interval. In (c), unchanged output at the end of *n* would mean that
  the forced recheck did not read live health.

**W6: the in-operation test (claims 8 and 9).**
- *Setup:* a named test file defines these types:
  - defence A with the needs-power flag and a drain of 10; defence B with the needs-power flag and
    no drain,
  - a type C with the special-power flag and a working animation, and a type D with the animation
    and without the flag; neither has the needs-power flag,
  - a defence E placed for the human house, never having changed owner, with the needs-engineer
    flag and no needs-power flag; a defence F with the
    needs-power flag, a drain and a mind-control weapon,
  - a refinery-type R with a working animation, no needs-power flag, and a harvester docked in it
    with its way out blocked,
  - a control-centre type K with powered unit X, no needs-power flag and no drain, with a long
    build-up time (at least 300 ticks), and a vehicle type X with the powered-unit flag,
  - a gap type G with the needs-power flag and a drain of 10 (S-0409 owns the gap),
  - charging infantry of a type that can charge A, a pulse weapon, a factory
    producing an item, and enemy targets in range of every defence.
  - The power state of each part is as that part says (spare consumer, above).
- *Procedure:*
  - (a) at low power, watch A and B.
  - (b) at low power, two chargers charge A; then only one.
  - (c) at full power with A switched off, two chargers charge A.
  - (d) at full power, a pulse hits A; watch A during the pulse and after it ends.
  - (e) at full power, a timed blackout (W4) while C and D are switched on.
  - (f) at full power, observe E before any ownership change; a trigger transfers it to another
    house and a second trigger returns it to the human house, then observe it again. Keep full
    power and a legal enemy target in range after its return.
  - (g) at low power, watch the factory.
  - (h) F takes over an enemy vehicle at full power; then the house is brought to low power.
  - (i) at full power, a sell order on R.
  - (j) at full power with no other control centre, place K; during its build-up order an X vehicle
    to a cell with no building; read whether the vehicle stops, and read it again when the build-up
    ends. In the same runs, place a second defence A together with K and watch whether it fires
    during its own build-up and after it.
  - (k) G at full power: read, from the enemy's side, whether cells inside the gap field are hidden,
    then bring the house to low power and read again.
- *Runs:* three per part, except (h) and (i), whose tick-level timing is run twenty times with the
  trigger at different points of the tick.
- *Observable:* whether each defence fires; whether C, D and R show their working animation; whether
  the item's progress advances; whether the X vehicle obeys orders; whether the gap hides cells.
- *Expected:* (a) A does not fire and B fires. (b) A fires with two chargers and not with one.
  (c) A fires; (d) A does not fire while under the pulse and fires again after it ends; (e) C stops
  its animation, D keeps it; (f) E does not fire before the ownership changes and fires after
  its return, remaining manned without an engineer inside; (g) progress advances,
  more slowly; (h) the vehicle returns to its owner's control 1 tick after the tick in which the
  recorder first reads low power; (i) the animation stops at the tick of the sale's first
  processing (S-0408 claim 6) and the building then stays on the map, sprite unchanged, until the
  harvester has left; (j) the X vehicle is disabled during the build-up and enabled again in the
  tick after the build-up ends (the building's first turn in operation); the second A does not fire
  during its build-up and does after it; (k) the gap hides cells at full power and stops hiding
  them one tick after the house's low-power update.
- *Falsified if:* any state, animation or firing result differs from the expectation, or any
  timed result is outside its stated tick interval.

**W7: switching power (claim 10).**
- *Setup:* own consumer (drain 40) and needs-power test type with output 10 and no drain.
  A named test file
  gives each the power-mode flag and a power-dependent working animation. A plant maintains
  full power throughout. A pulse weapon. Part (d) uses two additional consumer variants with
  the firestorm-wall flag: one map-placed switched on, the other map-placed switched off.
- *Procedure:* (a) click each test consumer in power mode to switch it off, record the tick *n*
  when its switch changes, then click again to switch it on. (b) Switch the consumer off, hit
  it with the pulse and click it in power mode while under the pulse. (c) Wait for the pulse
  to end without another click; read *D*, then click to switch on. (d) Click the two
  firestorm-wall variants in power mode and read their switch state and drain after the next
  house update. Use normal scripted input only, not a trigger that might follow a different
  application rule.
- *Runs:* three per part, except (a): twenty with tick-level recording.
- *Observable:* switch state, *O*, *D* and whether the working animation is running or paused,
  sampled by the recorder after orders each tick. Also record its displayed image frame,
  comparing with a control run in which no click occurs. The pause state can change after
  the animation already advanced in its turn that tick; the displayed image alone does not
  distinguish that from pausing in the next turn.
- *Expected:* (a) both test buildings switch off; the drain of 40 leaves *D* and the output
  of 10 leaves *O* at the next house update. Both animations are paused by the end of tick *n*,
  not first at their next turn.
  Clicking again switches them on, restores their contributions at the next house update,
  and makes their animation running in that order's tick. (b) The switch stays
  off, the consumer contributes no drain, and its animation remains stopped. (c) It still
  contributes no drain until the new click after the pulse; that click switches it on and
  restores its drain at the next house update. (d) Neither variant changes its switch state
  or drain contribution.
- *Falsified if:* any switch, output, drain or animation result differs from the stated expectation,
  in particular, a firestorm-wall variant changing state falsifies the exclusion, and an
  animation changing only on the next turn falsifies the immediate switch effect.

**W8: control centres (claim 13).**
- *Setup:* a named test file defines these types. Control-centre type K with the needs-power flag,
  a drain of 10 and powered unit X; a plant that covers every drain (full power throughout, so low
  power never decides the result); vehicle type X with the powered-unit flag, and vehicle type
  X' with no powered-unit flag, used only in part (f); vehicle type Z with the powered-unit flag
  that no centre names; an own repair depot; two vehicles of each of X and Z, and one of X'. For part
  c the file adds a second control-centre type K2 (same flags and drain) with powered unit Y,
  one of each centre, and vehicles of X and Y. For part h the file adds an own test transport type
  that can carry the X' vehicle. The centres are switched off by power orders (the type's drain
  makes the order effective, claim 10).
- *Procedure:*
  - (a) switch the centre K off, read the state of both X vehicles and give them orders. (b) switch
    it on again and repeat.
  - (c) with K and K2 both on, switch off only K.
  - (d) with K off and no other centre active, order one Z vehicle to a cell that holds no building.
  - (e) in the same state, place the second Z vehicle next to the depot and order it onto the depot,
    so that the depot's cell is the first cell it enters, then order it on to a cell with no
    building.
  - (f) repeat (a) with a variant of K whose powered unit is X', and the X' vehicle.
  - (g) after (d), switch K on again and read the state of the Z vehicle.
  - (h) with the variant of K of (f) switched on as the only active control centre (K and K2
    absent or switched off), load the X' vehicle, enabled, into the transport; switch that centre
    off while the vehicle is inside; then unload it onto a cell with no building and give it
    orders.
  - (i) continue (c) by switching K2 off too, then switch K on again with K2 still off.
- *Runs:* three per part.
- *Observable:* whether the vehicles move and obey orders.
- *Expected:* (a) both X vehicles are disabled and refuse orders; a hovering vehicle can still
  settle, spin and travel residually as S-0503 claims 8 and 10–11 specify (scenario H7 there tests
  those effects and water destruction). (b) Both work again; (c) the X vehicles
  keep working (quirk); (d) the Z vehicle is disabled on entering the cell; (e) it stays enabled on
  the depot's cell and is disabled on the first cell without a building after it; (f) the X'
  vehicle is disabled and enabled like X in (a) and (b); (g) the Z vehicle stays disabled; (h) the
  X' vehicle obeys orders after it is unloaded (it was off the map at the transition, and the
  cell-entry rule does not apply to it because its type lacks the powered-unit flag). (i) On
  the count's 1-to-0 transition only Y is disabled; X keeps working. Bringing K back enables
  X only, so Y stays disabled until K2 itself causes a later 0-to-1 transition.
- *Falsified if:* (a) or (b) differ, or in (c) the X vehicles stop, or in (d) the Z vehicle keeps
  working, or in (e) it is disabled on the depot's cell or stays enabled afterwards, or in (f) the
  X' vehicle is unaffected, or in (g) the Z vehicle works again, or in (h) the X' vehicle is
  disabled after it is unloaded, or in (i) either vehicle type differs from the expected state.

**W9: spy satellite (claim 14).**
- *Setup:* a named test file gives a test type with the spy satellite flag (S-0409) the needs-power
  flag, a drain of 10 and the power-mode flag; the same house has plants and a spare consumer.
  Its reveal has taken effect (S-0409 gives when) before each part.
- *Procedure:* (a) bring the house to low power. (b) At full power, switch the test type off.
  (c) At full power, start a timed blackout (W4). (d) At low power, switch the test type off too.
- *Runs:* three per part.
- *Observable:* whether the shroud returns over unexplored areas (S-0409's observable).
- *Expected:* the map stays revealed in every part.
- *Falsified if:* the shroud returns in any part.

**W10: superweapon charge (claim 15; S-0512 claims 4 and 6).**
- *Setup:* a named test file defines a power-dependent superweapon test type, own and charging,
  and a second variant with that dependence off, using S-0512's import mapping. Both use an
  ordinary running charge clock and have a giving building switched on, plants and a spare
  consumer. Charge time is long enough that neither becomes ready during the test interval.
  No trigger sets charge and no second-stage order restarts it.
- *Procedure:* (a) after the grant under full power, bring the house to low power for 600 ticks,
  then back to full power. (b) The same with the second variant; low power must start after the
  grant, not at it. (c) Grant the second variant under low power, restoring full power 600 ticks
  later. (d) With full house power throughout, switch off every giving building of the
  power-dependent type for 600 ticks, then switch one on. Each is compared with a control
  granted at the same tick with full power and its giving building on throughout.
- *Runs:* five per part and per control.
- *Observable:* remaining charge time during the interval and the tick of readiness.
- *Expected:* (a), (c) and (d) keep their remaining time during suspension and become ready
  600 ticks (±2) later than their controls; (b) keeps charging and becomes ready on its
  control's tick. S-0512 X2 and X3 supply additional switch and initial-grant variants.
- *Falsified if:* a suspended clock counts down, any delay in (a), (c) or (d) is outside
  598 to 602 ticks, or (b)'s readiness tick differs from its control.

**W11: super-gap (claim 16).**
- *Setup:* only if input or a test map can give the deploy behaviour to a building: a gap test type
  with a super-gap radius and an extra drain of 20, own, at full power.
- *Procedure:* deploy it twice.
- *Runs:* three.
- *Observable:* *D* after each deploy.
- *Expected:* *D* rises by 20 after the first and falls back after the second.
- *Falsified if:* either recorded drain differs from the expected drain. This proposal becomes
  usable backing only once the input path in Setup is demonstrated.

**W12: upgrades (claims 1 and 2).**
- *Setup:* a named test file defines these types. A test plant P of base output 100, strength 500,
  upgrade count 3; upgrade types for P with output 10, output 20 and drain 5; a wall tower test type
  WT (named as the game-wide wall tower type in the test file) of base output 50, upgrade count 4,
  four upgrade types for WT with outputs 10, 20, 30 and 40. Part (f) uses full-health P with two
  level-less upgrades: a lower upgrade of output 10 and an upper upgrade of output 20 whose type
  has the turret flag of S-0408 claim 5. A second variant gives the lower upgrade drain 5 instead
  of output 10. No other building contributes output or drain in either variant.
  Part (g) uses a fresh full-health P with three level-less upgrades: lower output 10,
  middle output 20, and upper output 0 with the turret flag. A second variant gives the middle
  upgrade drain 5 instead of output 20. A replacement level-less upgrade has output 30 and
  no turret flag. No other building contributes power in these variants.
- *Procedure:* (a) add the three upgrade types to P one at a time (S-0406 claim 12 gives how),
  reading *O* and *D* after each. (b) Add the four upgrade types to WT one at a time, reading *O*
  after each. (c) In a fresh run, give P all three upgrades and damage it to health 333.
  (d) In a fresh full-health run, give P all three upgrades and sell its upgrades one at a time,
  reading the settled totals after each sale. (e) In a fresh run, give the output-10 upgrade
  a level field of 3 and add it to unupgraded P; the lower positions stay empty (S-0406).
  (f) In each turret variant, read totals with both upgrades attached, sell the upper upgrade,
  then read settled totals, the upgrade count and the recorded positions. Give another sell
  order and observe whether the host begins its ordinary sale rather than selling the lower upgrade.
  (g) In each three-upgrade variant, sell the upper turret upgrade, read settled totals, then
  add the replacement upgrade and read settled totals, the count and recorded positions.
- *Runs:* three per part and per variant.
- *Observable:* *O* and *D* after each addition, damage or upgrade sale; in (f) and (g), also the count,
  recorded upgrades and the host's ordinary sale, using the read-only recorder and scripted input.
- *Expected:* (a) *O* 110, then 130; *D* 5 after the third, *O* still 130. (b) *O* 60, 80, 110 and
  110 again after the fourth. (c) *O* 86 and *D* 5: health scales all raw output, not drain.
  (d) *D* 0 with *O* 130 after the first sale, then *O* 110 and 100 after the next two.
  (e) *O* 110 and *D* 0 despite the empty lower positions.
  (f) Before the upgrade sale, the output variant has *O* 130 and *D* 0; the drain variant has
  *O* 120 and *D* 5. After it, both have *O* 100 and *D* 0, count 0, the lower position still
  recorded and the upper position empty. The next sale begins the host's ordinary sale in both
  variants (S-0408 claim 6).
  (g) After the turret sale both variants have count 0, positions 1 and 2 retained, position 3
  empty, *O* 100 and *D* 0. After the replacement, both have count 1 and the replacement
  in position 1, with position 2 unchanged: the output variant has *O* 150 and *D* 0,
  the drain variant has *O* 130 and *D* 5. Thus the position above the positive count contributes.
- *Falsified if:* any sum differs from the expected value in its part, or in (f) the lower
  position is cleared, the count stays positive, its upgrade still contributes power, or the next
  sale sells the lower upgrade instead of the host, or in (g) position 2 is cleared or fails to
  contribute after the count becomes 1.

**W13: map-requested power state (claim 18, Q-0180).**
- *Setup:* a named test map defines full-health test building types: P with base output 100,
  no base drain and no needs-power flag; C with base drain 40; N with base output 10,
  no base drain and the needs-power flag. Give all three the power-mode flag. Add two
  firestorm-wall variants, F with C's drain and G with P's output and no needs-power flag.
  Pre-place each with switched-on field 0. Control runs use field 1 and an absent field.
  Use the map-owner setup of S-0009 to give the human player the test objects. Keep enough
  other generation for full power; the observed area is these buildings. Game speed 2,
  director and unrelated mods off, one distant bystander under the common void rule.
- *Procedure:* record switch states and totals from the first visible tick. Click each
  building in power mode after setup, then record its switch and the next house totals.
- *Runs:* three for each field variant.
- *Observable:* switch state per building, output and drain, and whether input gives a power
  order; use read-only recording and ordinary scripted input, never state writes.
- *Expected:* with field 0, P and G start on, C, N and F start off. P and G each contribute
  output 100; C, N and F contribute neither output nor drain. Clicks switch C and N on,
  restoring drain 40 and output 10 at the next house update. Neither firestorm-wall variant
  changes through a power order; P remains on because input offers no power order for it.
  With field 1 or absent, all start on. A separate run without the power-mode flag has
  the same initial states but offers no power-mode order.
- *Falsified if:* any initial state differs, a flag-0 pure producer loses output, a later
  applicable power-on fails without a pulse, or a firestorm-wall power order changes state.

**W14: meaningful meter scale (claim 19, Q-0276).**
- *Setup:* a named test map defines own test plants and consumers with base output 500
  and base drain 300 in total, hence *B* = 800, and no upgrades or passengers. A second
  variant has base output 500 and base drain 500, hence *B* = 1000. Use game speed 2,
  stock data except these named overrides, director and unrelated mods off, and the
  distant bystander under the common void rule. The observed area is these buildings
  and the local player's meter. A third variant has no buildings and both totals 0.
- *Procedure:* capture the settled meter at full health. In the first variant damage
  plants until total output is 200, then switch them off. In the second variant damage
  plants until output is 300, then switch them off. Wait until the meter stops moving
  after each change; record *O*, *D* and building health with the read-only recorder.
- *Runs:* three per variant.
- *Observable:* occupied extent and the lengths of the three visibly distinct meter
  parts in screenshots, compared with the recorded totals. Infer the original's band
  capacity from its repeated drawn marks rather than fixing any screen position here.
- *Expected:* in the first variant, the full-power occupied extent is *A* for *B* = 800.
  At (200, 300), the supplied and near-shortage target proportions are 2/3 and 1/3,
  with no further-shortage part. At (0, 300), near and further shortage have proportions
  1/3 and 2/3. In the second variant, at (300, 500), supplied, near and further shortage
  have proportions 3/5, 1/5 and 1/5. At (0, 500), the shortage proportions are 1/5
  and 4/5. The settled occupied capacity follows F4 for the respective *B* and does not
  shrink merely because plants are damaged or switched off. The empty-house variant
  has one supplied band and no shortage bands. Allow a single drawn
  band of quantisation discrepancy per part, not a changed semantic threshold.
- *Falsified if:* a settled part differs by more than one band from these proportions,
  shortage classification follows a percentage rather than 100 power units, or the
  occupied extent follows damage-scaled output instead of the base-dependent scale.
  No transition duration or colour choice is tested by this scenario.

## Open questions

- Whether any input or scripted path gives a gap building its deploy behaviour, so that claim 16 can
  occur in a skirmish.
- The exact first tick of a timed blackout (claim 6), which depends on where in the tick the spy's
  entry falls (W4 parts a and c vary it).
- Whether a blackout started with a length of 0 or less during a running one occurs in any
  skirmish (claim 6); it needs a superweapon effect with its own length.
- Whether removal of a passenger inside a surviving absorbing building makes the house's
  power due (S-0605 claim 15).
- The full list of building functions that the in-operation test gates beyond those of claim 9.
- What else a fourth or later upgrade of a wall tower changes in the original (S-0406 claim 12).
  This spec does not depend on it: power reads only the three positions (W12 confirms that a fourth
  adds nothing).
