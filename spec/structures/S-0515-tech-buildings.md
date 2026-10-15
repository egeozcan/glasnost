---
id: S-0515
title: Tech buildings (manned state, acquisition bonus, periodic income, ownership changes)
topic: structures
revision: 2
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# S-0515: Tech buildings (manned state, acquisition bonus, periodic income, ownership changes)

## Summary

Tech buildings are neutral buildings that a player takes over to use their services. This spec owns
what is particular to them: the *manned* state that an ownership change gives a building for good, the
one-off credit bonus for taking a cash-producing building from a passive house, the periodic income
of a cash-producing building and how ownership changes and erasure affect it, and what an ownership change
does and does not do to a secret lab's secret type. Every other service of a tech building is the
general rule for its type's fields, owned by the specs listed in claim 1. This spec covers
skirmish only; campaigns and network games are out of scope.

## Terms

- **Tech building.** A building whose type has the needs-engineer flag (S-0405, Terms). The stock
  ones are the neutral service buildings that maps place for the neutral house: the oil derrick, the
  hospitals, the machine shop, the airport, the outpost, the secret lab and the civilian power
  plant. "Tech building" is a name for this spec's subject; no rule below depends on the name, only
  on the type's fields.
- **Ownership change.** S-0509's *building ownership change* (S-0509, Terms), written short: any change of a
  building on the map from one owning house to another, whatever causes it: an engineer capture
  (S-0509 claim 8), an occupation or its end (S-0508), a trigger action (S-0011 claim 24). A
  building put on the map for a house, by placement, a deploy, a trigger action that creates it or
  the scenario start, has no ownership change. An event that would give the building to the house that
  already owns it is not an ownership change and has none of the effects below.
- **Manned.** A state of a building: it is *manned* from its first ownership change on, for good
  (claim 2). This is the meaning of "an engineer is inside" in the engineer row of S-0405 claim 8.
- **Cash-producing type.** A building type with a nonzero acquisition bonus or a positive income
  delay (Import mapping). The *acquisition bonus* *B*, the *income amount*
  *a* and the *income delay* *D* are whole numbers; *B* and *a* are in credits, *D* in ticks.
- **Income countdown.** A countdown in ticks that every building has (claim 4). It is *restarted*
  with a length; its *remaining time* is the length less the ticks since the restart, and never
  below 0. A countdown that has reached 0 has *run out* and stays so until the next restart.
- **Income cycle.** The moment, in a building's own turn (glossary, *turn*, second sense), at which
  its income countdown is restarted by claim 5 and a payment may be made.
- **Passive house** (S-0009, Terms), **credit balance** (glossary), **in operation** (S-0405, claim
  8), **being erased** (glossary) are used as defined there.

## Behaviour

1. **What a tech building is made of.** A tech building has no rule of its own beyond its type's
   fields, its manned state (claim 2) and, for a cash-producing type, claims 3 to 7. Its other
   services are the general rules of the fields its type sets, owned elsewhere:

   | Stock building | Service | Owner |
   |---|---|---|
   | hospital, machine shop | house-wide self-healing of infantry or vehicles, totals moved by ownership changes | S-0314 claim 10 |
   | (any type with the hospital flag) | healing infantry inside | S-0314 claim 7 |
   | outpost | repair of vehicles (S-0314 claim 6) and its weapon (firing gated by S-0405 claim 9, table T1) | S-0314, S-0405 |
   | airport | gives its superweapon (the paradrop) to its owner | S-0512 claims 2, 3 and 7; S-0513 claim 42; S-0522 |
   | secret lab | offers its secret type to its owner | S-0407 claims 4, 14 and 15; claim 8 below |
   | civilian power plant | power output to its owner | S-0405 claim 1 |
   | (armory, no stock tech building) | veterancy training | S-0311 claim 12 |

   Being taken over (S-0509), not being sellable (S-0408 claim 3), not counting for defeat
   (S-0414 claims 2 and 3) and the capture announcements (S-0701 claim 39) are likewise those
   specs' rules. The airport needs nothing further: S-0512 claim 3 grants its superweapon to a new
   owner, and claim 7 there removes it from the old one, because a giving building does not
   depend on being in operation (S-0512, Terms) and therefore not on being manned.
   - Backing: `measurable` (scenarios TB6 and TB8 for the airport and the outpost).
     The dependency index is `unmeasured` here: it is not independent confirmation of the
     owning specs' claims, which retain their own backings and scenarios.
   - Confidence: high

2. **The manned state.** A building becomes manned at its first ownership change, whatever the cause
   and whatever the old and new owners, and stays manned for the rest of its life: later
   ownership changes, damage, power, a pulse and being erased do not end it. A building is not manned when it
   is put on the map, by any means. Through the engineer row of S-0405 claim 8, a tech building that
   has never changed owner is never in operation. In particular:
   - a tech building that a map gives to a player house from the start, or that a trigger action
     creates for one, produces no income (claim 5) and cannot fire (S-0405 claim 9) until its first
     ownership change,
   - its power output (S-0405 claim 1), its self-heal counts (S-0314 claim 10), its superweapon
     (S-0512) and its secret type (S-0407 claim 4) do not depend on being in operation, so these
     work for an unmanned tech building as for any other.

   The state is the same for every building; only the needs-engineer flag makes S-0405's engineer
   row read it.
   - Backing: `measurable` (scenario TB3 parts c to f; TB8). The services delegated to other
     specs retain those claims' backings rather than independent confirmation here.
     A first change between two passive houses is `unmeasured`: no scenario here exposes its
     manned state without a later ownership change that would itself man the building.
   - Confidence: high (set by every ownership change, never cleared); medium (no other way to set it)

3. **Acquisition bonus.** At an ownership change of a building of a type whose acquisition bonus *B*
   is not 0, when the **old** owner is a passive house, the new owner's credit balance changes by
   *B* at once (Formulas F3). The bonus does not depend on the new owner (a passive new owner is
   paid too), on the cause of the ownership change, on the building being in operation, or on whether
   the change is announced. An ownership change from a house that is not passive pays nothing. A
   building that returns to a passive house pays the bonus again at its next ownership change away
   from it. The bonus is paid straight into the balance: no income factor, country bonus, handicap
   or difficulty factor applies to it.
   - Backing: `measurable` (scenarios TB2 and TB5)
   - Confidence: high

4. **When the income countdown restarts.** A building's income countdown is restarted with the
   length *D*:
   - (a) when the building is put on the map for the first time,
   - (b) at every ownership change,
   - (c) at each of its income cycles (claim 5).

   Nothing else restarts it. A building created and not yet restarted has a countdown that has run
   out, and so has a countdown restarted with *D* ≤ 0. Because it restarts at an ownership change,
   the countdown of a building taken from another player starts again from the full length: the
   new owner does not inherit the old owner's phase.
   - Backing: `measurable` (scenarios TB2 and TB9). The exhausted countdown before placement
     is `unmeasured`: an off-map building has no income turn, and placement replaces its phase
     before an income payment could expose the initial value.
   - Confidence: high

5. **Income cycles and payments.** In a building's own turn, when the remaining time of its income
   countdown is exactly 1 tick, the building has an income cycle: the countdown is restarted with
   *D* (claim 4 c) and then, if the building's **current** owner is not a passive house and the
   building is in operation (S-0405 claim 8), the owner is paid the income amount *a* (Formulas F2).
   A cycle without payment still restarts the countdown, so a payment missed for want of
   operation (switched off, under a pulse, unmanned) is not made up and does not shift later
   cycles. A countdown that has run out has no cycle. So, for *D* ≥ 2 and a building whose turns
   are not skipped, the cycles fall *D* − 1 ticks after the restart and every *D* − 1 ticks after
   that (Formulas F1, `quirks.cash_cycle_one_short`). A building with *D* = 1 has at most one cycle
   per restart, in the restart's own tick when the building's turn in that tick comes after the
   restart, otherwise none; with *D* ≤ 0 it has no cycle at all. With *a* = 0 a payment changes
   nothing unless the balance is negative (F2).
   Income does not depend on the game speed except through the tick rate (S-0201).
   - Backing: `measurable` (scenarios TB1, TB3, TB5 and TB9)
   - Confidence: high (the cycle at remaining time 1, the period *D* − 1, the conditions,
     restart without payment and the delay boundaries)

6. **Income during erasure.** A building that is being erased has no income cycle in its turns,
   while its countdown keeps running in ticks. If the tick in which the
   remaining time is 1 falls in that time, the countdown runs out, and the building has no further
   income cycle until a restart of claim 4 (in practice, its next ownership change), even after the
   erasure has ended (`quirks.cash_cycle_lapse`). An erasure that ends before that tick changes
   nothing.
   - Backing: `measurable` (scenario TB4)
   - Confidence: medium

7. **Income and ownership changes.** Each payment goes to the house that owns the building at that
   cycle. After an ownership change the old owner receives nothing more from the building, and the
   new owner's first payment comes at the first cycle of the countdown restarted by the ownership change
   (claim 4 b), that is, *D* − 1 ticks after the ownership change for *D* ≥ 2. A building owned by a
   passive house, such as a derrick still held by the neutral house, keeps cycling and pays nobody.
   - Backing: `measurable` (scenario TB2) for recipients and ownership-change phase.
     Continued cycling under a passive owner is `unmeasured`: it pays nobody, and the next
     ownership change restarts the countdown, hiding its earlier phase.
   - Confidence: high

8. **Secret labs on an ownership change.** An ownership change does not change a secret lab's secret type
   and takes no random draw for secret-type selection: the type it was given at game start
   (S-0407 claim 14) stays with it
   through any number of ownership changes, and passes from the old owner to the new owner by S-0407
   claims 4 and 15.
   - Backing: `measurable` (scenario TB7)
   - Confidence: high

## Formulas

> **F1. Income cycle ticks** (claim 5). For a restart in tick *t*₀ with *D* ≥ 2 and no skipped
> turns, the cycles fall in ticks *t*₀ + *k*(*D* − 1), *k* = 1, 2, … With
> `quirks.cash_cycle_one_short` off: *t*₀ + *k*·*D*. Integers, in ticks; no rounding.

> **F2. Payment** (claim 5). Let *Y* be the owner's credit balance before the payment and *Y'*
> after it. All values are signed 32-bit integers in credits, with wrapping as in S-0205
> claim 1; no rounding. Write wrap(*x*) for that signed wrapping.
> - *a* > 0: *Y'* = wrap(*Y* + *a*).
> - *a* ≤ 0: write *q* = wrap(−*a*). With no stored resources, *Y'* = wrap(*Y* − *q*)
>   when *Y* ≥ *q*; otherwise *Y'* = 0. In particular *a* = 0 with a negative balance
>   sets the balance to 0. Skirmish has no stored resources (S-0404 claim 4).
>
> Only modified data reaches the second case; stock income amounts are positive.

> **F3. Acquisition bonus** (claim 3). *Y'* = *Y* + *B*, signed integers in credits, for any sign
> of *B*, with signed 32-bit wrapping (S-0205 claim 1), no rounding and no floor: a negative
> bonus can take the balance below 0. Only modified data has a negative bonus.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Cycle trigger | 1 | ticks | An income cycle happens when the countdown's remaining time is exactly this (claim 5). |
| Absent-key values of *B*, *a*, *D* | 0 | credits, credits, ticks | Import mapping. |

## State and ordering

| State | Event | Result |
|---|---|---|
| unmanned | first ownership change | manned for good (claim 2) |
| countdown running, remaining > 1 | building's turn | nothing |
| countdown running, remaining = 1 | building's turn, not being erased | restart with *D*; pay *a* to the current owner if it is not passive and the building is in operation (claim 5) |
| countdown running, remaining = 1 | building's turn while being erased | nothing; the countdown runs out in the next tick (claim 6) |
| countdown run out | building's turn | nothing (claim 5) |
| any | ownership change | bonus *B* to the new owner if the old owner is passive and *B* ≠ 0; restart with *D* (claims 3, 4) |

Observable ordering:
- The bonus and the countdown restart take effect within the ownership change, so the new owner's
  balance shows the bonus in the tick of the ownership change. A later cycle belongs to the new
  owner; an earlier cycle in that tick belonged to the old owner.
- The payment of a cycle is made in the building's own turn of that tick (S-0203), so it is in the
  balance by the end of the turn pass of the cycle's tick.

## Edge cases and quirks

- `quirks.cash_cycle_one_short` (default on = original; claim 5). The cycle comes when one tick
  is left, so the period is *D* − 1 ticks, not *D*. Off: a cycle comes when the full *D* ticks
  have passed since the restart (F1), so the period is *D*, and *D* = 1 pays every tick. The
  number of payments in a long window differs by a factor of *D* / (*D* − 1) for *D* ≥ 2
  (a real-valued ratio, without rounding).
- `quirks.cash_cycle_lapse` (default on = original; claim 6). A cycle that falls while the building
  is being erased is lost together with all later cycles until the next ownership change. Off: for a
  countdown last restarted with *D* ≥ 1, the building's first turn at or after its due tick
  has the cycle (restart and, if the conditions hold, payment), and cycles continue from there.
  The due tick is when one tick remains with `quirks.cash_cycle_one_short` on, and when the
  full *D* ticks have passed with it off.
- An ownership change from a passive house to a passive house (for example by a trigger action) pays
  the bonus to the new passive owner, where it has no effect on play.
- A tech building that a map gives to a player from the start is unmanned (claim 2): it gives
  power, self-healing, a superweapon and a secret type, but no income, and a tech outpost so given
  cannot fire until its first ownership change.
- Negative bonuses and non-positive income amounts follow F2 and F3; stock data never has them.

## Import mapping

| Key (file, section) | Concept | Parse form | Absent-key default |
|---|---|---|---|
| `ProduceCashStartup` (rules, building type) | acquisition bonus *B* (claim 3) | integer (S-0003), credits | 0 |
| `ProduceCashAmount` (rules, building type) | income amount *a* (claim 5) | integer, credits | 0 |
| `ProduceCashDelay` (rules, building type) | income delay *D* (claims 4, 5) | integer, ticks | 0 |
| `NeedsEngineer` (rules, building type) | needs-engineer flag (S-0405 owns) | boolean | no |
| `MultiplayPassive` (rules, country) | passive house (S-0009 owns) | boolean | no |
| `SuperWeapon` (rules, building type) | the airport's superweapon (S-0512 owns) | type name | none |
| `SecretLab` (rules, building type) | secret-lab flag (S-0407 owns) | boolean | no |
| `InfantryGainSelfHeal`, `UnitsGainSelfHeal` (rules, building type) | self-heal counts (S-0314 owns) | integer | 0 |
| `CaptureEvaEvent` (rules, building type) | capture announcement (S-0701 owns) | announcement entry | none |

A later section that omits one of the three cash keys keeps the value in force (S-0003's
read-with-current-default rule). No other key feeds this spec's claims.

## Measurement scenarios

Common setup: a small purpose-made test map, stock rules plus the named test file that the
scenario gives, the bench (exact tick numbers, scripted input), with the director and mods off,
and game speed index 2 at 800 × 600. Each scenario's named test file is
*tech-buildings-test.map*, whose override sections supply only the test fields described there.
Remove it after the run. Run each deterministic variant once and repeat any failed run. The
human house H has no harvester, no production, no sale and no other income, so its credit balance
changes only through the derrick. A read-only recorder logs tick numbers, ownership and the
credit balances of H, E and the neutral and special houses. A reading "at tick *t*" means after
the turn pass of that tick, without changing ownership to stop income. This avoids both the
sidebar's display lag and a change-owner trigger suppressing a payment in the observed tick.
Ownership changes "by trigger" use
the change-owner trigger action (S-0011 claim 24) on the derrick's tag. The bystander rules of the
glossary (*measurement setup*) apply; the bystander E takes part only through named objects and
test triggers.

**TB1. Income period** (claim 5, F1, `quirks.cash_cycle_one_short`).
- Setup: a test derrick type with *B* = 0, a test income amount *a* and *D* = 11, owned by the
  neutral house; an own engineer next to it.
- Procedure: capture at tick *t*₀ (recorded); read H's balance at *t*₀ + 1000. Repeat with
  *D* = 2 and *D* = 3.
- Expected: a rise of *a* × ⌊1000 / (*D* − 1)⌋ (100 payments for *D* = 11). Falsified by
  *a* × ⌊1000 / *D*⌋ (90 payments) or any other count.

**TB2. Bonus and ownership changes** (claims 3, 4, 7).
- Setup: a test derrick type with test *B* and *a* and *D* = 51, held by the neutral house; an own
  engineer.
- Procedure: H captures at *t*₀ (recorded); by trigger the derrick goes to E at
  *t*₁ = *t*₀ + 50*m* + 20 (20 ticks after H's *m*-th payment), and back to H by trigger at
  *t*₂ = *t*₁ + 200; read at *t*₂ + 45.
- Expected: H's balance has risen by *B* + *m* × *a*: the bonus once, no payment while E held
  the derrick, no bonus on the return from E, and no payment in the 45 ticks after the return
  because the countdown restarted there (first payment due at *t*₂ + 50).
- Falsified by: one more *a* (the return kept H's old phase, which would pay at *t*₂ + 30), one
  more *B* (bonus from a house that is not passive), or fewer *a*.
- Part (b): as above, but the trigger at *t*₁ gives the derrick to the neutral house and H's
  engineer captures it again at *t*₂: expect *B* twice. Part (c): the derrick starts with the
  neutral house and a trigger gives it to H directly: expect *B* (the cause plays no part).
- Part (d): transfer from the neutral house to the special house by trigger and record both
  balances immediately around the transfer. Expect the special house to gain *B*, and no gain
  to the neutral house. Any different credit delta falsifies the passive-new-owner rule.
- Part (e): pulse the neutral derrick before transferring it to H by trigger. Expect the full
  *B* immediately, with no periodic payment while the pulse lasts. Repeat with a different
  country: *B* must remain unchanged. A missing or scaled bonus falsifies
  claim 3.
- Part (f): ask for H to receive its already-owned derrick by trigger 20 ticks after a payment.
  Expect no bonus and the next payment 30 ticks later. A bonus or a restarted phase falsifies
  the same-owner rule.

**TB3. Payment conditions** (claims 2, 5).
- (a) As TB1 with *D* = 51 (payments at *c*, *c* + 50, …): a pulse on the derrick from *c* + 10
  to *c* + 110; read at *c* + 175. Expect payments at *c*, *c* + 150 only (2 × *a* from *c*
  on). Falsified by 4 payments (missed payments made up or not missed), or by 3 with a later
  phase.
- (b) The same with a test derrick type that can be switched off (S-0405 claim 10), switched off
  over the same window. Same expectation.
- (c) A test map that gives the derrick to H from the start (*B* = 0): read at tick 1000. Expect
  no rise at all.
- (d) Continue (c): at *t*₀ a trigger gives the derrick to the neutral house and a second trigger
  gives it back to H at *t*₀ + 1; read at *t*₀ + 1 + 1000. Expect *a* × ⌊1000 / 50⌋: the owner
  changes have manned it. Falsified by no rise.
- (e) Repeat (c) with a trigger-created derrick for H rather than one placed at game start.
  Expect no income until it changes owner; any earlier payment falsifies the claim.
- (f) After manning the derrick, apply nonlethal damage, a pulse and a power-switch interruption
  after a successful payment. Once each interruption ends, expect payments on the original
  phase without another ownership change. Failure to resume falsifies persistence of the
  manned state. TB4's short-lock control tests persistence through erasure without an
  ownership change.

**TB4. Erasure lapse** (claim 6, `quirks.cash_cycle_lapse`).
- Setup: as TB1 with *D* = 51 and a high strength for the test derrick type; a named E unit with
  an erasing weapon, which a test trigger orders to attack the derrick 10 ticks after one of
  H's payments, and which H's scripted fire kills about 100 ticks later, before the derrick is
  erased.
- Procedure: read at 2000 ticks after the lock ends.
- Expected: no payment after the lock begins. Falsified if payments resume after the lock ends.
  A second run in which a trigger then gives the derrick to the neutral house and back to H:
  expect payments again from 50 ticks after the return.
- Control: a lock that begins 5 ticks after a payment and ends 30 ticks later changes nothing.

**TB5. Non-positive amounts** (F2, F3). Test data only, low priority. (a) *B* = −100 captured
with a balance of 50, and a positive *a* of 100 with *D* = 51: expect the shown counter at 0
after the capture and at 50 after the first payment (−50 + 100), not 150; the recorder must
show −50 immediately after capture. (b) *a* = −5 with a balance of 3: expect 0 after the first
cycle, not −2. (c) With that same negative bonus, set *a* = 0: expect the negative balance to
become 0 at the first cycle, not to remain negative. (d) Test signed wrapping with a positive
*a* that takes the balance above the largest signed 32-bit integer: expect the wrapped value,
not saturation. Repeat with a positive *B* that crosses that boundary at capture. (e) With *a* equal to the smallest signed 32-bit integer and balance 0,
expect that same smallest integer after the cycle, not 0. Use *D* = 51 for each variant and
record the balance immediately before and after the cycle. Any different value falsifies F2
or F3.

**TB6. Airport** (claims 1 and 2). A test map gives H an airport from the start (unmanned) and the
neutral house a second one. Expect H's paradrop icon from the start (S-0512 claim 3) although the
airport is unmanned. A test trigger then gives H's airport to E: expect H to lose the icon in the
same tick's house update (S-0512 claims 2 and 7). H's engineer then captures the neutral airport:
expect the icon back. Falsified if the unmanned airport gives nothing.

**TB7. Secret lab** (claim 8). A test map with one secret lab and test secret lists of length 3
(S-0407 scenario T9), held by the neutral house. H's engineer captures the lab; then, by trigger,
it goes to the neutral house and back to H, five times, in quiet ticks. Expect the same secret type
offered to H every time it owns the lab. Run five games, logging the seed without setting it.
The random-state recorder checks each ownership change against a control with identical
objects, visual effects and ownership changes but the secret-lab flag disabled. Expect no extra
selection draw in the lab run; any extra draw or any changed offered type falsifies claim 8.
An unchanged type alone does not establish absence of a draw.

**TB8. Unmanned outpost** (claims 1, 2). A test map gives H an outpost from the start and puts an
E vehicle in its range: expect no shot in 600 ticks. Then a test trigger gives it to the neutral
house and back to H (two ownership changes): expect it to fire at the E vehicle from then on.
A shot before the transfer, or continued silence after it despite an otherwise legal target,
falsifies the manned-state firing gate.

**TB9. Delay boundaries and initial phase** (claims 4 and 5). Use the named test map to
supply derricks with *D* = 1, 0 and −3, positive *a*, and *B* = 0. Give each to H by a trigger
before the turn pass of a recorded tick, and log balances for 100 ticks. Expect exactly one
payment in the transfer tick for *D* = 1, then none; expect none for the other delays.
A later payment falsifies the boundary rule. Repeat with an engineer capture after the
building's turn: for *D* = 1 expect no payment. For initial placement, use a test derrick
without the needs-engineer flag, owned by H from game start, and *D* = 51. Log placement tick
*p* and expect the first cycle at *p* + 50, then every 50 ticks; an immediate payment or a
different phase falsifies claim 4 (a).

## Open questions

- Whether a service depot's repair (S-0314 claim 6) needs its building to be in operation, and so
  whether an unmanned outpost repairs vehicles, is not established here; TB8 can add a variant.
- Whether a passive house holds superweapon records for the airports it owns has no known
  observable effect and is left to S-0512.
