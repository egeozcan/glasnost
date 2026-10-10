---
id: S-0304
title: Special versus values (what zero and small versus values mean for firing, retaliation and automatic targeting)
topic: combat
revision: 2
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
measurements: []
requests: []
supersedes: []
---

# S-0304: Special versus values (what zero and small versus values mean for firing, retaliation and automatic targeting)

## Summary

Besides scaling damage (S-0302), a warhead's versus value for the target's armour class decides
three targeting questions in the original: whether a weapon may fire at an object at all, whether
a unit hit by an attacker fights back, and whether an object is picked automatically as a target.
Each question compares the versus value with a fixed threshold: exactly zero, a threshold just
below 1 %, and a threshold just below 2 %. This spec owns those thresholds and the three rules. It
also states the one use in weapon choice and the use in target preference, which S-0308 and S-0309
own in full, and lists the one further use that S-0309 owns, in the choice of search classes.

## Terms

Shared terms used here are defined in the glossary: *armour class*, *warhead*, *weapon*,
*target, destination*, *behaviour*, *house*, *human-controlled*, *importer warning*. In addition:

- **Versus value.** The factor *v* that a warhead gives one armour class, a double-precision number
  read from the warhead's versus list (S-0003 claim 13; the class order is S-0305's). It is the
  same value that S-0302 multiplies damage by. "The versus value of a weapon against an object"
  means the versus value of the weapon's warhead for the object's armour class.
- **Weapon it would use.** Of an actor with two weapons, the one the weapon choice of S-0308
  picks against the given object; for an actor with one weapon, that weapon. Every rule below
  looks at this weapon only.
- **Retaliation.** An actor's automatic attack on the object that has just damaged it, as opposed
  to an attack it was ordered to make or one it chose by itself in an automatic search.
  S-0309 (claim 18) gives the other conditions on the retaliating object and the attacker, and
  S-0215 (claims 18 and 19) owns what retaliation does and its behaviour, owner and distance
  conditions.
- **Automatic search.** Any search in which an actor picks a target by itself, without an
  order naming that target: the range, area and map-wide searches of guarding, area guarding,
  hunting and idle behaviours and of firing on the move (S-0215 for when they run, S-0309 for
  the searches).
- **Illegal target.** An object at which a weapon may not fire at all, as opposed to one it cannot
  fire at for the moment (out of range, not facing it, reloading). S-0214 claim 6(e) uses this
  test for the attack cursor.
- **Force-fire, attack cursor.** S-0214's terms. S-0214 owns what holding force-fire changes when
  an attack order is given, and when the attack cursor appears on an object (claim 6(e)).
- **τ<sub>r</sub>, τ<sub>a</sub>.** The retaliation threshold and the acquisition threshold
  (Constants).

## Behaviour

1. **A versus value of exactly zero makes the target illegal.** A weapon whose versus value against
   an object is exactly zero (+0 or −0) may never fire at that object, whatever the order, whether
   force-fire is held, and whatever the owner of either side. The test concerns infantry units,
   vehicles, aircraft and buildings only: a weapon fired at a cell (a force-fire order on the
   ground) or at another kind of object (a terrain object such as a tree) is not affected by any
   versus value. Every other versus value, positive or negative, however small, leaves the target
   legal for an ordered attack.
   - Backing: `measurable` (scenario V1, runs 1 and 4)
   - Confidence: high (medium for other kinds of object, which no scenario here covers)

2. **A force-fire order on an illegal target is accepted but never fires.** S-0214 (claim 6(e))
   lets force-fire bypass the cursor's legality test, so with force-fire held the attack cursor
   appears on an object that claim 1 makes illegal, and the order is accepted. The actor then never
   fires at it. It drops the object as its target, and stops the attack, at the first check of its
   attack behaviour whose answer is *illegal*. In the precedence of the firing check's answers
   (S-0301, claim 9), *rearming*, *out of ammunition* and *cloaked* come before *illegal*: while
   the actor's rearm countdown is running, or it is out of ammunition or cloaked, the answer is
   *rearming*, *out of ammunition* or *cloaked* and not *illegal*. So the target is dropped at the
   first check of the attack when none of those answers holds then, and otherwise at the first
   check after they have ended (for the countdown, after it has run out). Until then the actor
   keeps the target, and it never fires. Without force-fire the
   attack cursor does not appear on such an object (S-0214).
   - Backing: `measurable` (scenario V1, run 4)
   - Confidence: medium (the illegality is certain; that the target is dropped at the first
     check that answers *illegal*, rather than kept, follows the attack behaviour's handling of an
     illegal target and has not been observed; the order of answers is S-0301 claim 9's)

3. **Retaliation needs a versus value above τ<sub>r</sub>.** An actor that the other rules of
   S-0309 would let retaliate does so only when the versus value of the weapon it would use against
   the attacker is greater than τ<sub>r</sub>, a number just below 1 % (Constants). A value of 1 %
   (written `1%` or `0.01`, S-0003) is above it, so a weapon of 1 % does retaliate. Zero, negative
   values and positive values up to τ<sub>r</sub> never retaliate.
   - Backing: `measurable` (scenario V1, run 2)
   - Confidence: medium (the comparison is clear, but a reading in which 1 % prevents retaliation
     is also plausible from play, so the scenario must decide)

4. **Automatic searches need a versus value above τ<sub>a</sub>.** In every automatic search, an
   object is a candidate only when the versus value of the weapon the searching actor would use
   against it is greater than τ<sub>a</sub>, a number just below 2 % (Constants). Objects at or
   below it are never picked automatically, however close they are and whatever else the search's
   scoring gives them. A value of 2 % (written `2%` or `0.02`) is above it and is picked like any
   other. This is the only versus test of a candidate; the candidate tests are S-0309's (claim 12, table
   C) and how candidates are scored is S-0309's (claims 13 and 14, Formulas F4 to F6). A separate
   rule that uses the versus values of two armour classes to choose which classes of object an
   infantry unit searches for at all is listed in claim 8.
   - Backing: `measurable` (scenario V1, run 3)
   - Confidence: high (the comparison); medium (that every automatic search of S-0309 applies it:
     the test belongs to the shared candidate check that the searches use)

5. **The resulting meaning of each versus value.** Together, claims 1, 3 and 4 give these bands
   for a versus value *v* of the weapon an actor would use against an object:

   | *v* | Ordered attack | Retaliation | Automatic searches | Damage per hit (S-0302) |
   |---|---|---|---|---|
   | exactly 0 | illegal (claims 1, 2) | no | no | 0 (but 1 on an undemolishable building, S-0302 claim 7) |
   | below 0 | yes | no | no | negative: the hit heals |
   | 0 < *v* ≤ τ<sub>r</sub> | yes | no | no | as S-0302, usually 0 |
   | τ<sub>r</sub> < *v* ≤ τ<sub>a</sub> (for example 1 %, or 0.015) | yes | yes | no | as S-0302 |
   | *v* > τ<sub>a</sub> (for example 2 %) | yes | yes | yes | as S-0302 |

   Because S-0003 (claim 13, `quirks.percent_list_integer_prefix` on, the original) reads a versus
   item written with a percent sign by its whole-number part only, an item written `1.5%` gives
   1 % and falls in the fourth row, and one written `0.5%` gives 0 and falls in the first. With that
   switch off, `1.5%` still falls in the fourth row and `0.5%` falls in the third. Written without a
   percent sign, `0.015` gives the fourth row and `0.005` the third.
   - Backing: `measurable` (scenario V1)
   - Confidence: as claims 1, 3 and 4

6. **Weapon choice avoids a zero versus value.** When an actor with two weapons chooses between
   them against an object, and S-0308's earlier rules have not already decided, a weapon whose
   versus value against the object is exactly zero is not chosen if the other's is not. The outcome
   depends only on which of the two values are exactly zero:

   | Primary's *v* | Secondary's *v* | Weapon chosen |
   |---|---|---|
   | ≠ 0 | ≠ 0 | not decided by this rule (S-0308) |
   | ≠ 0 | 0 | primary |
   | 0 | ≠ 0 | secondary |
   | 0 | 0 | primary, which is then illegal by claim 1 |

   Small non-zero values play no part in the choice. S-0308 owns the rest of the weapon choice and
   the precedence of this rule within it.
   - Backing: `measurable` (scenario V2)
   - Confidence: high (the zero test and its outcome); medium (its place among S-0308's rules)

7. **Versus values weigh target preference.** In the scoring of an automatic search the
   versus value enters as a factor of two terms: the versus value of the searching actor's weapon
   against the candidate, and the versus value of the candidate's
   weapon against the searching actor. The second term is present for every candidate that is an
   infantry unit, vehicle, aircraft or building, in every kind of search, and has its sign reversed
   when the candidate is already targeting the searching actor. The weights, whether a higher score
   is preferred and the rest of the score are S-0309's. This claim only fixes that the values used
   are the versus values defined here, unchanged (not the damage of S-0302), each of the weapon
   that its owner would use against the other (S-0308).
   - Backing: `measurable` (through S-0309's scenarios)
   - Confidence: medium

8. **One more use, and no other.** Apart from the damage of S-0302, claims 1 to 7 and the
   following, the original makes no decision on a versus value. The exception is S-0309's rule about
   the classes an armed infantry unit searches for, claim 5 of S-0309: when the versus values
   of the primary weapon's warhead against both armour classes 4 and 6 (S-0305's numbering) are
   exactly zero, the unit searches for infantry only. It is a test of two fixed armour classes
   of the unit's own primary weapon, not of a weapon against a given object, it uses the same
   exact-zero comparison as claim 1, and S-0309 owns it and its place among the search rules.
   - Backing: `unmeasured` for the absence of other uses (a statement about every rule of the
     original; each owning spec shows its own rules); the exception is `measurable` through
     S-0309's scenario A2
   - Confidence: medium (every use of the versus list found in the program is listed here, in
     S-0309 claim 5 or in S-0302)

## Formulas

> legal(*v*) ⇔ *v* ≠ 0. Exact comparison (S-0205, claim 4); −0 counts as 0.
>
> may retaliate(*v*) ⇔ *v* > τ<sub>r</sub>. Exact comparison of the double *v* with the
> constant.
>
> automatic candidate(*v*) ⇔ *v* > τ<sub>a</sub>. Exact comparison of the double *v* with the
> constant.

No arithmetic is done, so no rounding applies and the results do not depend on
`quirks.chop_rounding` (S-0205). The thresholds are single-precision numbers, so an engine that
compared single-precision values with 0.01 and 0.02 written as doubles would decide `1%` and `2%`
differently from the original (see Edge cases).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| τ<sub>r</sub>, retaliation threshold | 5368709 / 2²⁹ = 0.00999999977648258209228515625 | versus factor (1.0 = 100 %) | The single-precision number nearest to 0.01; claim 3 |
| τ<sub>a</sub>, acquisition threshold | 5368709 / 2²⁸ = 0.0199999995529651641845703125 | versus factor (1.0 = 100 %) | The single-precision number nearest to 0.02; claim 4 |
| Illegal value | 0 | versus factor | Claim 1 |

For reference, `1%` gives the double 0.0100000000000000002081668…, and `2%` gives
0.0200000000000000004163336… (S-0003 claim 13), each just above its threshold.

## State and ordering

| Situation | *v* of the weapon it would use | Result |
|---|---|---|
| Actor given an attack order on an object, no force-fire | 0 | No attack cursor (S-0214); no order |
| Actor given an attack order on an object, force-fire held | 0 | Order accepted; never fires; target dropped at the first check that answers *illegal* (claim 2) |
| Actor given an attack order on an object | ≠ 0 | Attacks normally |
| Actor damaged by an attacker, other rules of S-0309 allow retaliation | > τ<sub>r</sub> | Retaliates |
| Same | ≤ τ<sub>r</sub> | Does not retaliate |
| Actor in an automatic search | > τ<sub>a</sub> | Object is a candidate |
| Same | ≤ τ<sub>a</sub> | Object is never picked |

Observable ordering: none of its own. The value tested is always that of the weapon the actor
would use, as S-0308 chooses it, so the bands apply to that weapon, not to its best weapon.

## Edge cases and quirks

- **The thresholds are single-precision numbers just below 1 % and 2 %.** A value of exactly 1 % or
  2 % as the importer produces it passes the test, so "1 %" means "retaliate but never pick
  automatically", and "2 %" behaves like any larger value. Our engine compares the double versus
  value with the exact constants of the Constants table. This is the original's behaviour and
  needs no switch: a mod can put any armour class in any band of claim 5 by the value it writes
  (for example 0.005 for the third band, or 0.015 for the fourth).
- **The weapon it would use decides.** The bands look at the weapon the actor would use, so an
  actor whose primary weapon is 1 % against an object and whose secondary is 0 % uses the primary
  (claim 6) and retaliates with it but never picks the object in an automatic search.
- **Negative versus values** are legal for ordered attacks but never lead to retaliation or to a
  pick in an automatic search, and their hits heal (S-0302). The importer gives no warning for them, since they are a known modding
  device.
- **Healing weapons.** A weapon whose damage is negative (a medic's, for example) uses the same
  tests: its targets need versus values above τ<sub>a</sub> to be picked automatically. Which
  objects a healer seeks out is owned by S-0314.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, warhead section, key `Verses` | the versus value *v* of each armour class (claims 1 to 7) | S-0003, claim 13 and row C5 (eleven items in S-0305's class order) | S-0003: eleven items of 100 %, which are above both thresholds |

## Measurement scenarios

**V1: the bands (claims 1 to 5).** All runs under the measurement setup, with named test files: a
test map with override sections for a test warhead *H*, a test weapon *W* using it, and the test
types below.
- *Setup:* flat open ground. Defender *D*: a stock vehicle type with a single weapon, replaced by
  *W* in the test map (same range, rate and projectile as its stock weapon, warhead *H*), owned by
  the human house, standing in *guard* (S-0215). The value of *H* against the armour class of
  *X*'s type in the run is *v*, and 100 % against every other class. Each run uses one *v* from the
  list 0 %, 1 % and 2 % (written with a percent sign) and −0.5, 0.005, 0.0099999999, 0.015,
  0.0199999999 and 0.025 (written without one): nine sub-runs per run kind. 0.005 lies in the band
  0 < *v* ≤ τ<sub>r</sub>, and 0.0099999999 and 0.0199999999 lie just above τ<sub>r</sub> and
  τ<sub>a</sub> but below 1 % and 2 %. Any game speed.
- *Run 1 (ordered attack):* an enemy vehicle *X* of a stock armed type, owned by the computer
  house, stands 3 cells from *D*, with its weapon replaced by one with 0 % against every class (so
  it never fires). Order *D* to attack *X* without force-fire. *Observable:* whether the attack
  cursor appears and *D* fires.
- *Run 2 (retaliation):* *X* has its stock weapon and stands 3 cells from *D*, within the range of
  both. *X* attacks *D* by itself. For *v* at or below τ<sub>a</sub>, *D* cannot have picked *X* in
  an automatic search (claim 4), so a shot by *D* at *X* after the first hit is retaliation. *Observable:*
  whether *D* fires at *X* after the first hit.
- *Run 3 (automatic search):* *X* is an enemy vehicle of a stock unarmed type within *D*'s weapon
  range, never attacking. *Observable:* whether *D* fires at *X* within 300 ticks.
- *Run 4 (force-fire on an illegal target):* as run 1 with *v* = 0 %, ordered with force-fire held.
  *Observable:* whether the cursor appears, whether *D* fires, and whether *D* still holds *X* as
  its target (it stays turned towards it) 100 ticks later. *D* must not have fired in the run
  before the order, so that its rearm countdown is 0 and the first check answers *illegal*. A
  second part, run 4b, has *D* first fire at a second enemy vehicle *Y* (an armed-and-legal
  target) and then, while the countdown from that shot is still running, be ordered to
  force-fire on *X*. *Observable:* the tick at which *D* stops holding *X* as its target, against
  the tick at which the countdown ends (the weapon's rate-of-fire delay in ticks, S-0301 claim 2).
- *Runs:* 3 of each sub-run (the outcomes are yes/no; differences between runs would only come from
  timing).
- *Expected:*

  | *v* | Run 1 fires | Run 2 retaliates | Run 3 fires |
  |---|---|---|---|
  | −0.5 | yes | no | no |
  | 0 % | no cursor, no fire | no | no |
  | 0.005 | yes | no | no |
  | 0.0099999999 | yes | yes | no |
  | 1 % | yes | yes | no |
  | 0.015 | yes | yes | no |
  | 0.0199999999 | yes | yes | yes |
  | 2 % | yes | yes | yes |
  | 0.025 | yes | yes | yes |

  Run 4: the cursor appears, *D* never fires, and *D* has dropped *X* as its target. Run 4b: *D*
  never fires at *X*, and keeps *X* as its target until the countdown ends and drops it at the
  first check after that.
- *Falsified if:* any cell of the table differs. In particular, run 2 with 1 % not retaliating
  falsifies claim 3 as written (it would mean the threshold is at or above 1 %), and run 3 with
  2 % not firing falsifies claim 4. Run 2 with 0.005 retaliating falsifies claim 3 (the threshold
  would be below that value), and run 2 with −0.5 retaliating falsifies it too (negative values
  would retaliate). Run 3 with 0.005, −0.5, 0.0099999999, 1 % or 0.015 firing falsifies claim 4.
  Run 2 with 0.0099999999 not retaliating, or run 3 with 0.0199999999 not firing, falsifies either
  the threshold (τ<sub>r</sub> or τ<sub>a</sub>) or the reading of the item as the double nearest
  to the written decimal (Terms; S-0003 claim 13): both values lie less than half a
  single-precision step above their thresholds, so an item read through single precision would
  fall exactly on the threshold. Run 4 with *D* firing falsifies claim 1; *D* keeping *X* as its
  target without firing (run 4) falsifies only the second half of claim 2, and in run 4b *D*
  dropping *X* before the countdown ends falsifies only its timing.

**V2: weapon choice (claim 6).** As V1 run 1, with a stock defender type that has two weapons whose
stock choice against *X*'s type is the primary. In the test map the primary's warhead is 0 %
against *X*'s armour class and the secondary's is 100 %. *Expected:* *D* fires its secondary at
*X*. Second run: the primary 100 %, the secondary 0 %: *D* fires its primary. *Falsified if* *D*
does not fire in the first run, or fires the zero-value weapon in either run.

## Open questions

- Whether every automatic search of S-0309 uses the shared candidate check of claim 4. The
  original's searches found so far all do; S-0309 should confirm it for each search it lists.
- Claim 2's handling after the force-fire order (dropping the target at the first check that
  answers *illegal*) should be checked against S-0215's attack behaviour, for vehicles and
  infantry as well as buildings.
