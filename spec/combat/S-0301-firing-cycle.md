---
id: S-0301
title: Firing cycle (delay between shots, bursts, ammunition and reloading, the firing check)
topic: combat
revision: 17
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-16
measurements: []
requests: [Q-0060, Q-0798]
supersedes: []
---

# S-0301: Firing cycle (delay between shots, bursts, ammunition and reloading, the firing check)

## Summary

When an object that has fired may fire again: the delay between shots and how rate-of-fire
bonuses combine into it, bursts and the short delays inside a burst, ammunition and reloading,
delayed and animation-synchronised shots, and the firing check, the per-attempt test whose answer
says whether a shot can be fired now and, if not, why. Aiming tolerances and turret behaviour are
S-0310; where a shot starts is S-0313; which weapon is used is S-0308; range is S-0307.
Drain attempts replace ordinary shots (S-0608 claim 1; claim 9 here). Disc-laser firing and its
lasting effect are S-0615 claims 12 to 17. Aircraft carrying a first passenger replace ordinary
shots with drop attempts (S-0522 claim 11), outside the ordinary-shot rules below.

## Terms

Glossary terms are used as defined there (*weapon*, *tick*, *tick number*, *turn* in both senses,
*target*, *destination*, *house*, *country*, *vehicle*, *infantry*, *building*, *aircraft*,
*scenario random stream*, *ranged draw*, *round-toward-zero mode*, *importer warning*, *quirk
switch*, *departure*, *AI house*, *speed-normalised delay*). From S-0202: R(*a*, *b*), the ranged
draw. From S-0205: RZ₅₃ and trunc (Formulas there). From S-0203: *turn pass*. In addition:

- **Firer.** An object that fires a weapon: a vehicle, infantry, aircraft or building.
- **Shot.** One firing of one weapon by a firer: one projectile, beam or effect launched, or, for a spawner weapon, one target handed to the spawn controller as a pending target. A shot is *made* when the weapon's firing action succeeds; an attempt that the firing check refuses, or whose projectile launch fails (claim 15), makes no shot.
- **Draining weapon, drain attempt, drainable type, draining, being drained.** As S-0608
  claims 1 to 3 and Terms. A drain attempt replaces an ordinary shot and does not count as a
  shot for this spec's countdown, burst, ammunition or ordinary-shot draw rules (claim 9).
- **Drop attempt.** A carrying aircraft's attempt to release its first passenger instead of an
  ordinary shot (S-0522 claim 11). Neither success nor refusal counts as an ordinary shot here.
- **Void shot.** An attempt that passed the firing check but whose projectile could not be put on
  the map (claim 15). It makes no shot.
- **Rearm countdown.** A per-firer countdown in ticks. It is set by every shot the firer makes, to
  that shot's *delay* (claims 2 to 8 and 10), and it is *running* until that many ticks have passed. While it
  runs, the firer may not fire any of its weapons, except for the sync shot of claim 13 and
  the unchecked shots of claim 9. One countdown serves all weapons of a firer.
  S-0215's *rearm time remaining* is the number of ticks still to run.
- **Rate-of-fire field** *R*. The imported weapon field that sets the base delay between shots, in
  ticks (Import mapping).
- **Burst count** *B*. The imported weapon field that sets how many shots one burst has.
- **Burst counter** *k*. A per-firer integer, 0 when the firer is created. It counts the shots
  made in the current burst, and one counter serves all weapons of a firer.
- **House rate-of-fire factor** *h*. A per-house double, set when the houses are set up (claim 3).
  The neutral and special houses (glossary, *house*) have *h* = 1.0 exactly.
- **Difficulty row.** As S-0302 (Terms): the entry of the glossary's *difficulty skill block* for
  one difficulty, one of the three rules sections *easy*, *normal* and *difficult* (Import
  mapping). Each row has a rate-of-fire factor *ℓ*. The *normal row* is the one the original gives
  a human-controlled house in a skirmish. Which row a house uses is set-up behaviour (claim 3).
- **Veteran, elite.** The two raised veterancy ranks of a unit, as S-0311 (veterancy)
  defines them; S-0311 claim 3 owns the test of whether a unit has an ability. A type's *veteran rate-of-fire ability* and *elite rate-of-fire ability* are two
  imported flags that say whether the raised rank speeds up its firing.
- **Veteran rate-of-fire factor** *v*. A game-wide double from the rules data, holding a decimal
  read as S-0003 claims 6 and 7 state (a single-precision value unless written with a percent
  sign). The difficulty-row and country factors are held the same way.
- **Garrisoned building.** A building whose type can be occupied by infantry and may fire while
  occupied (two imported flags), and which has at least one occupant. Its occupant count is *n*.
- **Occupy factor** *m*, **bunker factor** *b*. Two game-wide single-precision decimals from the
  rules data (Import mapping).
- **Bunkered.** A vehicle that is inside a tank bunker (S-0206, Terms).
- **Berserk.** The state of a unit that a berserk-causing warhead has put out of its owner's
  control (S-0610 claims 4 to 6). *Berserk-causing warhead*, *psionics immunity* and
  *berserk-friendly* have the meanings of S-0610 Terms.
- **Effect weapon.** A weapon whose shot keeps a lasting effect attached to the firer: a sonic
  wave, a spark-particle or fire-particle emitter, or a railgun particle trail (each an imported
  weapon flag). The effect is *alive* from the shot until the effect ends (the combat and rendering
  specs own its duration).
- **Disc laser weapon.** A weapon with the imported disc-laser flag (S-0203, Terms).
- **Magnetic-beam weapon.** A weapon with the imported magnetic-beam flag. Its shot can make a
  *magnetic wave* attached to the firer (S-0607 claim 21). The firer's *held target* is its
  gripped unit (S-0607 claim 5); that grip lasts until an event of S-0607 claim 12, independently
  of the wave's lifetime. S-0607 owns the lift and release.
- **Temporal weapon.** A weapon whose warhead has the imported temporal flag: an erasing weapon
  (glossary, *being erased*). A firer's persistent erasing capability is initially supplied by
  its primary weapon's warhead. Its *temporal beam* is attached to its target from
  the moment it locks on until it lets go. S-0308 (claim 6) and S-0310 (claim 7) also use the term.
- **Holding a target.** A firer *holds* a target with a temporal weapon while its temporal beam is
  attached to that target, and with a magnetic-beam weapon while that target is its held target.
- **Spawner weapon, spawned aircraft.** A *spawner weapon* is a weapon with the imported spawner
  flag. Its shots launch no projectile; they hand the shot's target to the spawn controller as
  a pending target (S-0506 claim 3), rather than launching aircraft immediately. The firer's
  *spawned aircraft* are the slot objects whose life cycle S-0506 owns. A spawned aircraft is
  *alive* while its slot is not empty (S-0506 Terms). A firer is *launching* spawned aircraft
  while S-0506's launching count is above 0 (S-0506 Terms and claim 10).
- **Seeker type.** A vehicle type with the imported hunter-seeker flag (Import mapping). Only its
  effect on the firing check (claim 9, rank 9) is specified here.
- **Playing death frames.** A vehicle whose type has death frames is *playing* them from the tick its
  death counter starts (S-0102, claim 22), until S-0312 removes it.
- **Materialising.** The state of an object that has just teleported and is phasing in at its
  destination. For the teleporting locomotor's own move, this is exactly *phasing in* of
  S-0207 claim 20(f): its delay is Formula F9 there, with that claim's end tick and harvester
  exception. Superweapon and other effect teleports remain owned by their feature specs.
- **Gattling type.** As S-0308 (Terms). **Spray-attack type.** As S-0313 (Terms).
- **Ammunition** *a*. A per-firer integer, in rounds. A type's *maximum ammunition* *A* and
  *initial-ammunition field* *J* are imported integers. An initial field of −1 selects *A*
  (claim 10); ammunition of −1 is unlimited. A maximum of −1 disables vehicle reloading,
  but does not override an explicit initial field.
- **Reload countdown.** A per-firer countdown in ticks, used only by vehicles (claim 11).
- **Firing check.** The test made by attempts to fire a weapon at a target, apart from the
  unchecked shots of claim 9. Its *answer*
  is one of the categories of claim 9: *can fire*, or a reason why not. An attempt either asks for
  the range test ("with range") or does not; a check that does not ask never answers *out of
  range* from the ordinary range test (S-0307). The attacking behaviours say which they ask (S-0310
  claim 9 for vehicles, claim 14 here for a delayed building shot).
- **Firing-frame counter.** A per-vehicle counter that is either *idle* or *running* with a whole
  number value, used by vehicles whose sprite has firing frames (claim 13).
- **Delayed-fire countdown.** A per-building countdown used by buildings whose shots wait for an
  animation (claim 14).
- **Charge turret.** A vehicle type with the imported charge-turret flag; it has several turret
  images (its turret count *N*), and which one is current follows the rearm countdown (claim 16).

## Behaviour

### The delay between shots

1. **Rearm countdown.** When a firer makes a shot in the tick with tick number *t*, its rearm
   countdown is set to that shot's delay *D* (claims 2 to 8 and 10). The firing check then answers
   *rearming* for every weapon of the firer, in every tick with tick number below *t* + *D*, and
   no longer from tick *t* + *D* on. This holds unless a reason of ranks 1 to 4 gives its own answer (claim 9), and
   except in the sync tick of claim 13 and between shots of an aircraft's same-tick full burst
   (claim 4), in which the countdown does not hold the shot back. Apart from those exceptions,
   *D* ticks pass from one shot to the next at the least. With *D* ≤ 0 the countdown is over at once. How often a firer attempts to fire (for a vehicle, once in
   each of its turns) belongs to S-0215 and to claim 9 here. A firer that has never fired, including
   one created during play, has no running countdown: it can fire as soon as it has a target. The
   countdown is not changed by switching weapons or targets. Counting is in ticks: game speed does
   not change *D*, and `quirks.speed_normalised_delays` does not apply to any delay of this spec.
   A carrying aircraft's drop attempt instead leaves the countdown unchanged on refusal and
   expires it immediately on success; it sets no ordinary shot delay (S-0522 claim 11).
   - Backing: `measurable` (scenario F1; S-0522 scenario P7 for drop attempts)
   - Confidence: high

2. **Which delay a shot gets.** Four kinds of delay exist: the 1-tick delay of a building with
   spare rounds (claim 10), the bare rate of an effect weapon (claim 5), the in-burst delay
   (claim 4) and the full delay below. When more than one kind could apply to a shot, the first
   in this order wins and the others are not used, so no draw of a lower kind is taken:

   | Order | Kind | Applies to a shot when |
   |---|---|---|
   | 1 | 1-tick delay (claim 10) | the firer is a building that holds more than 1 round before the shot |
   | 2 | bare rate (claim 5) | the weapon is a sonic weapon, or a spark-particle, fire-particle or railgun weapon: every shot of such a weapon, whether or not an effect of its kind was already alive (the shot starts its emitter or trail when none is) |
   | 3 | in-burst delay (claim 4) | the burst position *j* is below the burst count *B* |
   | 4 | full delay (this claim) | none of the above |

   So a building with spare rounds has no rearm-countdown gap beyond 1 tick, whatever its
   weapon's burst count or kind; other gates, including a live effect, can still prevent a shot.
   An effect weapon with a burst count of 2 or more takes its bare rate after every shot of
   the burst and never draws R(3, 5). Only the full delay takes the factors of claims 3, 6 and 7,
   and the berserk halving of claim 8 applies to whichever kind was chosen. The burst counter
   advances in every case (claim 4).

   Creating the emitter or trail is taken to always succeed. A creation that fails because the host
   machine runs out of memory, which can happen in the original, is not modelled and has no
   scenario.

   The full delay is *D* of Formula F1. Its base is the weapon's rate-of-fire
   field *R* times the firing house's rate-of-fire factor *h* (claim 3) plus one ranged draw
   R(0, 2) from the scenario random stream. Each floating-point operation is rounded according to
   `quirks.chop_rounding` and S-0205; each conversion to a whole number of ticks is truncated toward
   zero. The veteran factor (claim 6), the garrison division (claim 7), the bunker division (claim 7)
   and the berserk halving (claim 8) scale it where their condition holds, nested as Formula F1 nests
   them. The nesting shows in the delays: the same factors nested otherwise can truncate to a
   different whole number of ticks. Formula F1 gives the number domain of each result.
   - Backing: `measurable` (scenarios F1 and F2 for the full delay; F3, F4 and F9 for the order)
   - Confidence: high (the full delay, and 1-tick before every other kind); medium (effect weapon
     before in-burst, since stock data has no effect weapon with a burst count above 1: test data
     must supply the case)

3. **The house rate-of-fire factor.** In a skirmish, *h* of a player house is the rate-of-fire factor
   of the house's difficulty row times the rate-of-fire factor of the house's country, a product of two
   doubles (Formula F2). In a campaign mission *h* is the row's factor alone; the country's factor is
   not used (this records the original; our engine's campaign rule is deferred, see below).
   - **Exception: the neutral and special houses.** The two non-player houses every skirmish creates
     have *h* = 1.0 exactly: no difficulty row and no country factor applies to them, whatever their
     countries' rate-of-fire factors or the rows hold. This is the original's rule, and our engine
     keeps it by decision of the project lead (S-0412 claim 5; S-0413 claim 8). Their delay is
     therefore the rate-of-fire field *R* plus the draw, before the other factors of Formula F1.
   - *Our engine: every player house, human or AI, uses the normal row* (a rule of our engine by
     decision of the project lead; no switch; the glossary's *difficulty skill block*). AI difficulty
     comes from the AI's strategy modules and never from this factor. A house's row can differ only
     through an optional per-player handicap setting, which is off by default and is never tied to
     an AI house's difficulty automatically. A difficulty row that a map sets for a house is
     ignored in a skirmish; only the handicap setting can change a house's row. Campaign
     difficulty is deferred to the future campaign spec (campaigns are out of scope for now).
   - *The original's set-up (a departure, no switch).* The original gives the house of every
     human player in a skirmish the normal row. The house of each computer player gets the row that
     its lobby entry selects (S-0412 claim 4 owns the pairing): the easy computer the difficult row,
     the normal computer the normal row and the hard computer the easy row. The computer-easy-bonus shift is conditional on more than one human participant and the rules' computer-easy-bonus flag being on. A skirmish has at most one human participant, including observers (S-0302, claim 3), so this shift does not occur in a skirmish. Its effect outside this spec's skirmish scope is not specified here. In a campaign mission the original takes the row of the player's chosen difficulty for the player's house and the row of the scenario's computer difficulty for the others. Our engine does not reproduce these assignments; the handicap setting can.
   - Backing: `measurable` (scenario F2); `unmeasured` for the one-step move of the
     computer-easy-bonus flag (it needs a game with more than one human player: a skirmish does not
     have one in our engine, and network games are out of scope). The historical campaign
     rules in this claim are `unmeasured` here: campaigns are outside this spec's
     skirmish scope and have no proposed run
   - Confidence: high (the product, that the campaign uses the row's factor alone, and the factor of
     exactly 1.0 of the two non-player houses, as S-0412 scenario K5 part d measures it); high for
     the row each lobby entry selects (S-0302 scenario D19 part c measures it); medium for the
     one-step move (never observed: it needs a game with more than one human player, which a
     skirmish does not have in our engine and network games are out of scope)

4. **Bursts.** A weapon with burst count *B* fires bursts of *B* shots. A shot made while the
   firer's burst counter is *k* is the *j*-th shot of its burst, *j* = *k* + 1, and leaves the
   counter at *j* mod *B*, so the counter is 0 again after the last shot of a burst. The delay
   after the shot depends on *j*:

   | Burst position | Delay after the shot |
   |---|---|
   | *j* < *B* | the *in-burst delay* below, unless claim 2's order gives a 1-tick delay or a bare rate |
   | *j* ≥ *B* | the full delay of claim 2, unless claim 2's order gives a 1-tick delay or a bare rate |

   The in-burst delay after the *j*-th shot is:
   - for a vehicle with 1 ≤ *j* ≤ 4 whose type gives a burst delay *j* that is not −1: that value,
     in ticks, with no draw,
   - otherwise one ranged draw R(3, 5) from the scenario random stream: 3, 4 or 5 ticks.

   The counter changes in the same way whichever kind of delay the shot gets. No rate-of-fire bonus applies to an in-burst delay, but the berserk halving does (claim 8). The
   burst counter is shared by the firer's weapons, so a firer that changes weapons in mid-burst
   carries the count over and the new weapon's *B* decides. When the firer's target is cleared (it
   has none), its burst counter is set to 0; a change from one target to another keeps it.
   An aircraft's full burst makes all *B* shots in one tick without a firing check between shots
   (S-0501 claim 12). Each shot still advances the burst counter and sets the countdown.
   The in-burst delays do not separate those shots. Each attempt made while a first passenger
   exists is instead a drop attempt: it never advances the burst counter or chooses an ordinary
   full or in-burst delay, including their draws (S-0522 claim 11). The exception ends when no
   first passenger remains; any further attempts in that same burst are not cargo drops.
   - Backing: `measurable` (scenario F3; S-0501 scenarios AC7 to AC9 for aircraft,
     S-0522 scenario P7 for drop attempts)
   - Confidence: high (the counter and both in-burst delays); medium (the reset when the target is
     cleared, and the carry-over between weapons)

5. **Effect weapons fire at their bare rate.** The delay after a shot of a sonic, spark-particle,
   fire-particle or railgun weapon is the rate-of-fire field *R* itself, with no draw, no house
   factor and none of the bonuses of claims 6 and 7 (the berserk halving of claim 8 still applies).
   It outranks the in-burst delay and is outranked by a building's 1-tick delay (claim 2's order).

   A firer has at most one live effect of each kind (one sonic wave, one spark emitter, one fire
   emitter, one railgun trail), whichever of its weapons started it. Each weapon of a firer has an
   *other weapon*: for the weapon in slot 0 it is the weapon in slot 1, and for a weapon in any
   other slot it is the weapon in slot 0 (S-0308 numbers the slots). The firing check (claim 9)
   answers as follows for the asked weapon:

   | The firer has a live effect of | Answer for the asked weapon |
   |---|---|
   | the asked weapon's own kind | *rearming* (rank 5) |
   | the kind of the asked weapon's other weapon, whatever the asked weapon's own kind | *impossible* (rank 4) |

   So an effect weapon fires again after max(*R*′, the effect's lifetime) ticks, where *R*′ is *R*
   (trunc(*R* / 2) for a berserk firer, claim 8), and no draw is taken for its delay. For a
   building that holds more than 1 round, *R*′ is 1 instead (0 if berserk, claim 8), so
   the live-effect gate can still hold it back (claim 10).

   A magnetic wave shares the sonic wave's single slot, so a live magnetic wave also counts as a
   live sonic effect: it blocks a sonic weapon of the same firer (*rearming* for the sonic weapon's
   own kind, *impossible* as the other weapon's effect) (`quirks.shared_wave_slot`,
   default on; the fixed behaviour keeps two slots).

   This does not make a magnetic-beam weapon an effect weapon of this claim: it does not get
   the bare rate or the max(*R*′, lifetime) interval. Its own live wave adds *rearming* only
   when the firer grips a unit and the asked target is neither that unit nor a building
   (S-0607 claim 11; claim 9, rank 4 here). Holding that very target remains a separate early
   refusal. There is no general own-wave firing block for a magnetic-beam weapon, including
   when the shared wave is sonic. A magnetic shot made while a wave is alive creates no new
   wave (S-0607 claim 21). Grip and wave lifetimes are independent.
   - Backing: `measurable` (scenario F4, with its burst and building parts; S-0607 scenarios
     L9 and L10 for magnetic firing, grip independence and wave creation)
   - Confidence: high (the bare rate, the missing draw, *rearming* for an effect weapon's own
     kind, and the magnetic conditional gate and absence of a general own-wave block),
     medium (*impossible* for the other weapon, and that the effect's lifetime can lengthen the
     interval, which depends on the effect specs); low for the shared slot of magnetic and sonic
     waves (a firer needs both weapons, so test data must supply the case)

6. **Veteran factor.** If the firer is veteran and its type has the veteran rate-of-fire ability,
   or it is elite and its type has the veteran or the elite rate-of-fire ability, the delay is
   multiplied once by *v* and truncated (Formula F1). Elite does not apply the factor twice.
   - Backing: `measurable` (scenario F2)
   - Confidence: high

7. **Garrisons and bunkers.** A garrisoned building's delay is the quotient of the delay by its
   occupant count *n* (integer division, truncated toward zero), divided further by the occupy
   factor *m* and truncated when *m* is above 0 (Formula F1, *D*₃ and *D*₄). A bunkered vehicle's delay is
   divided by the bunker factor *b* and truncated when *b* is not 0 (Formula F1, *D*₅). An imported value
   that is not a number never reaches *b*: S-0003 claim 8 makes the importer treat it as absent
   (Edge cases). A building is never treated as
   bunkered.
   - Backing: `measurable` (scenario F5)
   - Confidence: high (formula); medium (the occupant-count division as observed in play, since
     which occupant's weapon fires is S-0308's)

8. **Berserk halving.** A firer that is berserk when it makes a shot gets half the delay chosen by
   claim 2's order (any of its four kinds, with the factors of claims 3, 6 and 7 where they
   apply), truncated toward zero. Exception: the shot of a disc laser weapon is never halved
   (S-0615 claim 12). The
   rule is stated for the firer's state, not for its kind: whether a building can be berserk
   belongs to the warhead specs, and if it can, its 1-tick delay halves to 0.
   - Backing: `measurable` (scenario F6)
   - Confidence: high

9. **The firing check and its answers.** Except for the unchecked shots named below, every
   attempt to fire asks the firing check, with the target and the weapon. Its answer is one of
   these categories:

   | Answer | Meaning |
   |---|---|
   | can fire | the shot can be made now |
   | rearming | the rearm countdown is running, the weapon's own effect is still alive (claim 5), a building's delayed shot is pending (claim 14), a firing-frame gate is closed (claim 13), and a few early cases named in the rank table |
   | out of ammunition | the firer's ammunition *a* is 0, or an aircraft marked as a passenger carrier now carries no passengers (S-0310 claim 5; S-0501 claims 1 and 19). These causes have different ranks below |
   | not facing | the weapon is not pointed at the target (S-0310) |
   | still turning | the turret is still turning (S-0310) |
   | moving | the firer may not fire while it moves (S-0310) |
   | cloaked | the weapon needs the firer uncloaked and it is not |
   | out of range | the target is beyond the weapon's range (S-0307) |
   | illegal | the target is not a legal target for this firer and weapon. The combat specs give the cases: a versus value of exactly zero (S-0304, claim 1), an airborne or non-ground target the projectile cannot hit, or a non-unit target of a projectile that is not anti-ground (S-0306, claim 23), the healing tests (their feature specs), the takeover test (S-0601 claim 3), the berserk refusals (S-0610 claims 3 and 6), the bomb-planting and disarm tests (S-0519 claim 3), the parasite tests (S-0614 claim 5), a held organic firer (S-0614 claim 8), a sinking firer or target (S-0504 claim 5), a falling firer (S-0511 claim 10), or the drain refusals of S-0608 claim 3 (rank 4 below). S-0214, claim 6(e), defers to these for what the check refuses |
   | impossible | the firer cannot fire this weapon now for a reason of its own: no weapon in the slot, the effect of its other weapon still alive (claim 5), the weapon needs power or the firer is disabled, a hidden target fails the test below (S-0516 claim 24), and similar |
   | busy | the firer is launching spawned aircraft (Terms) |
   | must deploy | a vehicle whose type must deploy before it fires, when its own cell fails the deploy-to-fire cell test of S-0517 claim 14 (a slope, a building in the cell, or a land type that does not allow it) and no reason of ranks 1 to 10 below holds. It then outranks *not facing* (S-0310, claim 9), and in the rank table also *moving* and *still turning*, an order that no game state shows (see below). On a cell that passes the test none of this applies: the vehicle gets the aiming answers as usual and S-0310 deploys it on *can fire* or *not facing* |

   **Hidden-target refusal (S-0516 claim 24).** A target hidden from the firer's house gives
   *impossible* when the firer's mean weapon damage is above 0 or the target is not allied with
   the firer. Hidden means fully cloaked without that house's sensor coverage; a building also
   counts at fade levels 11 to 15 without coverage (S-0516 claim 18). Mean weapon damage is the
   target-independent integer of S-0308 claim 15, including its ambient component and
   truncation toward zero. It is not the selected weapon's damage or the shot damage after
   firepower or armour scaling. The refusal applies even to a forced attack on an own object.
   Allied hidden targets with mean weapon damage at or below 0 pass this test only; all other
   firing restrictions still apply.

   Where more than one reason holds, the firer gets the answer of the highest rank below (rank 1
   is the highest):

   | Rank | Answer | Reasons at this rank |
   |---|---|---|
   | 1 | impossible | a vehicle that is playing its death frames (S-0102, claim 22: its death counter has started; vehicles only) |
   | 2 | busy | a vehicle that is launching spawned aircraft (S-0506 claim 10) |
   | 3 | illegal | a vehicle that is melee-linked (S-0214, Terms), whatever its target (vehicles only) |
   | 4 | illegal, impossible, rearming | most *illegal* and *impossible* reasons (those about the firer's state, the other weapon's alive effect and most target tests), the *illegal* answer of an infantry healer (S-0310, claim 11), the *illegal* answer when the firer is sinking (S-0504 claim 5) or falling (S-0511 claim 10), the *illegal* answer when the target is the building the firer drains (with any weapon), or when a draining weapon targets an object already being drained or of a type that is not drainable (S-0608 claim 3; the latter two tests do not apply to cell targets), the *illegal* answer for a berserk-causing warhead against a psionics-immune target, a bunkered vehicle or a tank bunker holding a vehicle (S-0610 claim 3), the *illegal* answer for a berserk firer against a berserk-friendly target (S-0610 claim 6), the *illegal* answer for a firing vehicle that is deploying or undeploying, or for a weapon whose warhead carries its target with a locomotor against a target vehicle that is deploying or undeploying, or is an in-place deploying type with *unload* current (S-0517 claim 11), the *impossible* hidden-target refusal defined above (S-0516 claim 24), the *impossible* answer for a spawner weapon whose mobile firer is under the parasite hold regardless of organic type (S-0506 claim 10; this precedes the organic-only *illegal* test at rank 9), and the early *rearming* cases: the firer is materialising, it already holds this very target with a temporal or magnetic-beam weapon, a magnetic-beam weapon's firer grips a unit and has a live wave while the asked target is neither the gripped unit nor a building (S-0607 claim 11), or the weapon is a spawner weapon none of whose spawned aircraft is alive (S-0506 claim 10). Their precedence among each other is not stated |
   | 5 | rearming | the rearm countdown runs, the firing-frame gate of claim 13 is closed, or the weapon's own effect is alive (claim 5) |
   | 6 | (not used) | an earlier numbering kept the weapon's own live effect here, with the same answer as rank 5; the two are one rank, and the number stays unused so that other specs' pointers to the later ranks stay valid |
   | 7 | out of ammunition | the firer's ammunition *a* is 0 |
   | 8 | cloaked | |
   | 9 | out of range, illegal | the late target tests: a seeker type (Terms) answers *out of range*, whatever the distance; a warhead that cannot affect the target's armour class (the zero-versus test of S-0304 claim 1), a failed takeover test (S-0601 claim 3), a bomb-planting warhead against a target that already carries a planted bomb or a disarm warhead against one that carries none (S-0519 claim 3), the parasite tests (S-0614 claim 5), the target is sinking (S-0504 claim 5), or the firer is held and organic (S-0614 claim 8; a spawner weapon is already refused at rank 4), give *illegal* |
   | 10 | out of range | the attempt asked for the range test and the target is beyond the weapon's range |
   | 11 | must deploy, out of ammunition | a vehicle whose own cell fails S-0517 claim 14's deploy-to-fire cell test gets *must deploy* (S-0310 claim 9); an aircraft marked as a passenger carrier that now carries no passengers gets *out of ammunition*, regardless of its ammunition count (S-0310 claim 5; S-0501 claim 19). The two causes apply to different kinds of firer |
   | 12 | illegal, impossible | the remaining vehicle tests: a vehicle that is tethered (glossary) with a building as its contact gets *impossible*, and a vehicle whose default weapon is a healing weapon gets *illegal* (S-0310, claim 11; S-0314). Which of the two wins when both hold is not stated; stock data has no vehicle that gives both |
   | 13 | moving | |
   | 14 | still turning | |
   | 15 | not facing | |
   | 16 | a refusal of the vehicle's locomotor | a vehicle whose locomotor adds a refusal of its own gets it only when no reason above holds; which answer it gives is open (Open questions) |

   **Falling and the landing turn (S-0511 claim 10).** A falling firer gets *illegal*
   at rank 4. For ordinary infantry and vehicles, the fall and any landing hit precede
   that turn's firing check. A unit still falling after its fall step therefore cannot fire,
   including an infantry unit whose firing sequence began before the drop. A unit that
   lands and survives loses this falling-specific refusal in the same turn: it may fire
   then if it attempts a shot and all other firing restrictions pass. Landing does not
   itself grant a shot or restore a target lost during the fall.

   **Drain attempts (S-0608 claim 1).** Passing the firing check with a draining weapon
   leads to a drain attempt rather than an ordinary shot: no projectile, shot damage or
   warhead effect, no rearm-countdown start, no burst advance, no ammunition use and no
   ordinary-shot draws. S-0608 claims 1, 2 and 5 own the link attempt, target clearing and
   link-formation effects. Clearing the target still resets the burst counter under claim 4.
   Link effects, including animation creation and release of controlled units, retain their
   own possible random draws; their randomness is not an ordinary-shot draw.

   A takeover weapon that passes S-0601 claim 3's test still faces the remaining target tests
   at rank 9, including the zero-versus test, and all later ranks (S-0601 claim 4).
   Passing takeover legality alone therefore does not give *can fire*.

   The aircraft cargo refusal at rank 11 follows every general refusal (ranks 4 to 10)
   and precedes aircraft aiming at rank 15. An empty marked carrier therefore answers
   *rearming* while its countdown runs, *out of range* when the ordinary range test fails,
   and otherwise *out of ammunition* even with rounds remaining and its aim wrong.
   A fresh aircraft placed empty has no passenger-carrier mark and no cargo refusal
   (S-0501 claim 1). This is the cargo-availability rule, not a defect or a quirk:
   passenger shots release cargo (S-0501 claim 20); unloading does not turn a marked
   carrier into an ordinary armed aircraft. No switch applies.

   So a firer whose countdown runs gets *rearming* even when its target is out of range or it is
   out of ammunition. The rank of *rearming* for a building's pending delayed shot (claim 14), and
   of *busy* for a firer that is not a vehicle, is not stated.

   Where reasons of two of ranks 1 to 4 hold at once, which stock data does not give, the answer
   may be that of either if the behaviours that respond to the answer (S-0215, S-0310) respond to
   the two answers alike; otherwise the rank table's order holds.

   Among ranks 11 to 15, S-0310 (claim 9) gives *must deploy*, *moving* and *still turning* no
   reply, so a vehicle with any of them stays as it is, while *not facing* makes it re-aim. So the
   order of those three among themselves shows in no game state, and an implementation may answer
   whichever of the three holds. The order of each of the three against *not facing* does show: a
   vehicle for which one of the three holds (a vehicle with a destination that is also facing the
   wrong way, for example) does not re-aim. *Illegal* and *impossible* of rank 12 draw replies of
   their own (S-0310, claim 11), so their place among the others shows too.

   **Exception (S-0215, claim 32).** The secondary shot of a building whose type has the SAM
   flag is made in the same processing as its primary shot without a firing check of its own, so
   the table above does not apply to it. The same exception applies to the secondary firing
   attempt of a non-SAM building with an installed upgrade when S-0215 claim 32's upgrade
   weapon gate passes: a primary attempt is followed by a secondary attempt in that processing,
   without another firing check. A rearm countdown begun by the primary does not prevent the
   secondary attempt. These are attempts, not a guarantee of two successful shots. If the
   secondary slot resolves to no weapon, that attempt does nothing: no shot, no burst-counter
   change, no rearm-countdown change, no ammunition use and no random draw. It produces no
   firing-check answer, because it asks no firing check. In particular, it leaves the primary
   shot's burst position and rearm delay intact. This requires the resolved slot to be empty,
   not merely the gate-tested upgrade's own secondary: S-0308 claim 12 owns resolution from
   other upgrades, occupants and the host. S-0215 claim 32 owns the gate and paired attempts.
   Claim 14 here specifies their priority over animation-delayed firing. An aircraft's full burst likewise has no firing check between
   its shots (S-0501 claim 12).
   - Backing: `measurable` (scenarios F7, F8 and F15; S-0506 scenarios R2 and R6 for spawner
     handoff and lockouts; S-0607 scenario L9 parts c and d for the gripping lifter's conditional
     rearming; S-0614 scenarios P2 and P9 for parasite restrictions; S-0504 scenario
     N6 for sinking restrictions; S-0511 scenario B3, turn-order variant, for falling
     refusal and the same-turn surviving-landing boundary; S-0517 scenario P4 for transition and magnetic-beam refusals
     and P5 for the deploy-to-fire cell test; S-0501 scenarios AC7 to AC9 for aircraft bursts, and
     scenario F15 part f and S-0516 scenario CS9 part c for hidden targets; S-0519 scenario T3 for bomb-planting and disarm legality,
     S-0601 scenarios MC2, MC3 and MC11 for takeover legality; S-0610 scenario P4 for the
     berserk-friendly refusal; scenario F7 part e and S-0501 scenario AC11 for the
     empty passenger-carrier refusal and its precedence; S-0608 scenarios D1 and D2 for
     drain attempts instead of shots and D3 for the rank-4 drain refusals; also
     S-0215 scenario B36 and its extensions for the upgrade gate and ordered paired attempts
     despite the primary's rearm countdown; scenario F17 for an empty resolved secondary
     slot and animation-delayed upgrade buildings).
     The berserk-weapon refusals of S-0610 claim 3 are `unmeasured` here for that claim's
     reason: stock berserk weapons do not isolate the single-target firing check.
     Exact precedence of the sinking target refusal is `unmeasured` for the reason stated by
     S-0504 claim 5. `unmeasured` for the order among ranks 1 to 4 when two of them hold at once (only
     contrived data gives such a firer, for instance a spawner vehicle that is also playing death
     frames or is melee-linked, and the behaviours that respond to the check's answer (S-0215,
     S-0310) are not known to respond differently to *busy*, *illegal*, *impossible* and the early
     *rearming* cases, so no scenario could tell the orders apart).
     Rank 16 is `unmeasured`: no isolated locomotor refusal and its answer have yet been
     identified, so no scenario here can distinguish it from another refusal.
   - Confidence: high (the empty-secondary no-op and the upgrade-building paired attempts without a second firing check,
     falling refusal and the same-turn surviving-landing boundary,
     the three drain refusals and no ordinary shot from a drain attempt),
     medium (its missing countdown start and ammunition use, as S-0608 claim 1), high (rearming before ammunition and range, facing and moving after them, and
     *must deploy* before *not facing*, and the hidden-target refusal using mean weapon damage),
     medium (the full membership of *illegal* and *impossible*,
     and the early *rearming* cases, which other specs own; only the ones relevant to this spec are
     listed; the tether and death-frame reasons); low (ranks 1 to 4 among themselves; rank 3, whose
     cause (S-0214) is itself low; rank 16). The passenger-carrier refusal and its
     precedence have medium confidence.

### Ammunition and reloading

10. **Ammunition.** A new vehicle, infantry unit, building or aircraft starts with ammunition
    *a*₀ = *A* when its type's initial-ammunition field *J* is −1, and *a*₀ = *J* otherwise
    (Formula F6). Thus an absent initial field fills a finite magazine, including 0 rounds
    for a maximum of 0; it does not grant unlimited ammunition. An explicit initial value is
    used without clamping to the maximum, even above it (`quirks.initial_ammo_above_capacity`,
    default original). An explicit 0 starts empty, including when the maximum is −1.
    The same rule applies to objects placed by the map and objects created for production.
    there is no separate map ammunition value. This is the starting count, before any turns
    that can reload a vehicle (claim 11) or service an aircraft. Each shot
    that a vehicle, infantry or building makes uses one round when *a* > 0; *a* = −1 is never
    changed. With *a* = 0 the firing check answers *out of ammunition*. A disc laser shot uses no
    round (S-0615 claim 12; and it neither advances a garrison's muzzle index, S-0313,
    nor restarts a vehicle's reload, claim 11). A drain attempt likewise uses no round
    (claim 9; S-0608 claim 1). Aircraft spend one round per attack run in which they fired, not per shot
    (S-0501 claim 15). A successful drop attempt additionally spends one round immediately,
    whereas a refused drop has no net immediate ammunition change (S-0522 claims 9 and 11).
    Run-level pending spending still applies even when every drop attempt in the firing pass
    is refused; it is additional to successful drops' immediate spending.
    Servicing on pads belongs to S-0501 claims 26 to 28. Only their starting count is owned here. The 1-tick rule depends on
    a building's ammunition before the shot: a building that holds more than 1 round before a shot
    has a delay of exactly 1 tick after that shot, with no draw, whatever its weapon's burst count
    or kind (claim 2's order; `quirks.building_ammo_fast_rearm`). The shot made with its last round gets the normal delay of
    claim 2, and a building with unlimited ammunition (*a* = −1) is not affected by the rule. A
    building with 3 rounds and no other firing gate therefore fires in ticks *t*, *t* + 1
    and *t* + 2, then answers
    *rearming* until its countdown ends, and only then *out of ammunition*.
    - Backing: `measurable` (scenario F9; the building part of scenario F3 for a burst weapon,
      the building part of scenario F4; S-0522 scenario P7 for drop ammunition)
    - Confidence: high (starting count, no clamp, the same rule for map and production,
      rounds used, out of ammunition); medium (the building rule: it needs a building with
      ammunition above 1, so test data must supply it)

11. **Reloading (vehicles only).** A vehicle whose type has *A* ≥ 0 regains ammunition over time.

    | Vehicle state or event | Reload behaviour |
    |---|---|
    | A projectile-launching shot leaves *a* < *A*; the type lacks the manual-reload flag and *A* ≠ 0 | The reload countdown restarts at the reload delay of Formula F3 for the current *a*. Continuous firing therefore keeps postponing the reload. |
    | The reload countdown has never been started and *a* < *A* | The countdown counts as over. The vehicle regains one round in the first of its turns in which *a* < *A*, whenever that turn comes; the countdown then starts as described in the next row. This includes a manual-reload vehicle until its first regained round. |
    | The reload countdown is over and *a* < *A* | *a* grows by 1 and the countdown restarts at the reload delay of Formula F3 for the new *a*. One round comes back per reload delay. |
    | The vehicle lacks the mobile-fire flag and its locomotor is moving it | Its reload countdown does not advance. |

    The reload delay is the type's empty-reload field when *a* = 0 and that field is not −1; otherwise
    it is the reload field plus the reload-increment field times *q*², where *q* = 1
    when the type's pip-wrap field is 0, else *q* is *a* divided by the pip-wrap field in integer
    division toward zero (Formula F3). Infantry and buildings never regain ammunition over time.
    A vehicle regains at most one round in each of its turns, so a reload delay of 0 or less still
    gives one round per tick.
    - Backing: `measurable` (scenario F10)
    - Confidence: high (formula, restart on each shot, one round per completion); medium (the
      never-started countdown that counts as over, the frozen countdown while moving, and the restart
      by damage of claim 17)

12. **Cloaked firers.** A weapon with the fire-only-visible flag (set unless the data clears it)
    gets the answer *cloaked* while its firer is in any cloaking state other than fully visible
    (cloaking, cloaked or uncloaking); an aircraft gets it only while fully cloaked. The firer's own behaviour then uncloaks it (the cloaking specs; S-0310, claim 9).
    - Backing: `measurable` (scenario F7, part c)
    - Confidence: medium

### Shots that wait

13. **Firing frames (sprite vehicles).** A vehicle whose type has *F* > 0 firing frames has a
    firing-frame counter that changes as follows:

    | Counter | Event | Result |
    |---|---|---|
    | idle or running | the vehicle makes a shot that is not a sync shot (below) | running at 2*F* − 1 |
    | running at a value above 0 | a turn of the vehicle, not travelling through a tunnel | running at one less |
    | running at 0 | a turn of the vehicle, not travelling through a tunnel | idle |
    | running | a turn in which the vehicle travels through a tunnel | unchanged |

    The type may give a firing sync frame for each of the values 0 and 1 of *k* mod *B*, where *k*
    is the burst counter (the data value −1 gives none, Import mapping). The sync frame *s* that
    applies follows the current value of *k* mod *B*:

    | *k* mod *B* | Sync frame *s* |
    |---|---|
    | 0 | the type's sync frame for value 0, if it gives one |
    | 1 | the type's sync frame for value 1, if it gives one |
    | 2 or more | none |

    *k* is below *B* except after a change of weapon has carried a count over (claim 4), so for
    a one-shot weapon the value is 0 and for a burst of 2 it is the burst position already made. Where a sync frame applies, an attempt
    with the primary weapon while the counter runs gets *rearming* until the counter equals *s*. In
    the tick in which it equals *s* the rearm countdown is ignored and the shot can be made; that
    shot is a *sync shot*. With no sync frame given, firing frames change only the picture.
    - Backing: `measurable` (scenario F11, test data only; its tunnel part for the tunnel rows)
    - Confidence: medium

14. **Animation-delayed building shots.** In the ordinary non-SAM attack path, outside
    S-0215 claim 32's active upgrade weapon gate, a building whose type has the
    animation-delayed-fire flag does not fire when its attack behaviour gets *can fire*.
    Instead it
    starts its firing animation and its delayed-fire countdown, set by the type's delayed-fire field
    *d* (ticks). The delayed shot is pending until the building's turn max(*d* − 1, 0) ticks after
    the turn that started it. (A building takes a turn in every tick. The countdown does not depend on how often its attack behaviour is processed,
    which is every other tick while it turns towards its target or rearms, S-0215 claim 32.) In that turn the building makes its scheduled firing check again, with range; that check is not refused merely because this same delayed shot is pending. On *can fire* it fires its weapon at its current target; on any other answer the shot is dropped. Either way the shot is no longer pending. Other firing checks made while the shot is pending answer *rearming*. The rearm countdown starts only with the shot itself.

    **Upgrade gate priority (Q-0798).** For a non-SAM building to which S-0215 claim 32's
    upgrade weapon gate applies, the animation-delayed-fire flag does not defer a passing
    gate's pair: the primary and secondary attempts occur in that processing, with no
    delayed shot started by it. A failed gate with the original quirk setting starts
    neither an attempt nor a delayed shot. With that gate's quirk off, a failed gate uses
    the ordinary selected-weapon path, including this claim's animation delay when enabled.
    A passing gate still makes the immediate pair. An empty resolved secondary slot has
    claim 9's no-op outcome, including when the host has the animation-delayed-fire flag.
    The gate's applicability, including the upgrade-position test, belongs to S-0215
    claim 32; weapon-provider order belongs to S-0308 claim 12 and upgrade placement to
    S-0406 claim 12. A gate bypass leaves the ordinary animation-delay rule in force.

    **Prism-type buildings.** S-0518 claims 1–10 own their support recruitment and supported
    shots. In the ordinary attack path covered there (no SAM flag or installed upgrades), a
    prism-type building recruits supporters before starting its own delayed shot. That shot
    always uses this countdown, regardless of the animation-delayed-fire flag, and both its
    final firing check and its shot use weapon slot 0 (S-0518 claims 4 and 7).
    - Backing: `measurable` (scenarios F12 and F17; S-0518 scenarios PS1 and PS5 for prism shots)
    - Confidence: medium (the countdown arithmetic); high (upgrade gate priority and the prism exceptions)

15. **Failed launches.** If the projectile of a shot cannot be put on the map, the shot is void:
    the rearm countdown, the burst counter, the ammunition and the garrison muzzle position (S-0313)
    stay as they were. A carrying aircraft's drop attempt is not a void shot merely because
    it launches no projectile: successful release and its ammunition spending remain, and its
    rearm countdown expires; refusal retains the countdown (S-0522 claim 11). Neither outcome
    produces ordinary-shot effects: no spray draw, weapon report or muzzle animation.
    - Backing: `measurable` (S-0306, scenario P5 run d: an arcing shot for which the launch pitch has
      no solution is removed before it enters play, S-0306 claim 6 a; observed here by scenario F14,
      void-shot part; S-0522 scenario P7 for the drop exception)
    - Confidence: medium (failed projectile launches); high (the drop exception)

### Related state

16. **Charge turrets.** While its rearm countdown runs, a charge-turret vehicle shows turret
    number *c* = trunc(*N* × *r* / *D*) clamped to 0 … *N* − 1, where *r* is the rearm time remaining and
    *D* the delay last chosen (after any berserk halving); with *D* ≤ 0, or once the countdown is
    over, *c* = 0. This is recomputed in each of its turns. It applies only to a charge-turret type
    with a turret count *N* above 0 that is not a gattling type. The turret number is for drawing
    only: it does not select the weapon. A charge-turret type's weapon is the slot of its weapon
    number, which stays at 0 because the type ignores every request to change it (S-0308, claim 5),
    so every shot of such a vehicle uses its slot-0 weapon, whichever turret image is shown (the one
    shot that can fall inside the countdown, a sync shot of claim 13, is no exception).
    - Backing: `measurable` (scenario F13)
    - Confidence: medium

17. **Damage can empty a magazine.** A type with the imported damage-reduces-readiness flag loses
    ammunition when damaged and restarts its reload countdown (claim 11) at once. The amount
    belongs to S-0302.
    - Backing: `measurable` (scenario F10, part d)
    - Confidence: low

18. **Draws taken by one shot.** These are ordinary-shot draws, not the possible
    link-effect draws of a drain attempt (claim 9; S-0608 claims 1 and 5), nor the
    standing-position and parachute draws of a drop attempt (S-0522 claims 9, 11 and 13).
    A drop attempt takes no ordinary-shot draws, whether successful or refused. A shot takes, from the scenario random stream and in this order:
    the spray pick of S-0313 (only for the first shot of a burst of a spray-attack type), the
    projectile's own draws (S-0306), the draws of creating an effect weapon's particle system
    (S-0202, claim 10, when this shot starts the effect), then the one delay draw (R(0, 2) of claim 2 or R(3, 5) of claim 4) if
    the delay kind chosen by claim 2's order takes one, and after it the draws of the effects the shot creates last: its
    muzzle-flash animation, a sonic wave or a laser (S-0202, claim 10, and the specs of those
    effects). A refused attempt takes no draw at all. A delay without a draw (a burst-delay field of claim 4, the bare
    rate of claim 5, the 1-tick delay of claim 10) takes no delay draw. A void shot as in claim 15 takes the projectile's draws but no delay draw.
    - Backing: `measurable` (scenario F14, with the random-state recorder; the void-shot rule through
      S-0306 scenario P5 run d; S-0608 scenario D1 part a for drain attempts and link effects,
      S-0522 scenario P7 for drop attempts)
    - Confidence: high (one delay draw per shot, or none); medium (its place between the projectile's and the animation's
      draws)

## Formulas

**F1. Full delay** (claims 2, 6, 7, 8). *R* is the rate-of-fire field (integer, ticks), *h* the
house factor (double), *u* = R(0, 2), *v* the veteran factor (double), *n* the occupant count
(integer), *m* and *b* the occupy and bunker factors (singles, widened exactly to doubles). RZ₅₃ is
S-0205's rounding of a double result, which is round toward zero under `quirks.chop_rounding` (the
default) and round to nearest with it off; trunc is the conversion of S-0205 claim 7.

> *D*₁ = trunc(RZ₅₃(RZ₅₃(*R* × *h*) + *u*))
> *D*₂ = trunc(RZ₅₃(*D*₁ × *v*)) if the veteran condition of claim 6 holds, else *D*₁
> *D*₃ = trunc(*D*₂ / *n*) (integer division toward zero) for a garrisoned building, else *D*₂
> *D*₄ = trunc(RZ₅₃(*D*₃ / *m*)) for a garrisoned building when *m* is above 0, else *D*₃
> *D*₅ = trunc(RZ₅₃(*D*₄ / *b*)) for a bunkered vehicle when *b* ≠ 0, else *D*₄
> *D* = trunc(*D*₅ / 2) (toward zero) for a berserk firer, unless the shot is a disc laser shot (claim 8), else *D*₅

Every value is a whole number of ticks. Examples (with *h* = 1, *v* = 0.5 as a double, *u* = 1):
*R* = 30 gives *D*₁ = 31 and a veteran's *D*₂ = 15; a garrison of *n* = 4 occupants with *m* = 2 then
gives *D*₃ = 3 and *D*₄ = 1.

**F2. House factor** (claim 3). With *ℓ* the difficulty row's rate-of-fire factor and *κ* the country's
(doubles): *h* = RZ₅₃(*ℓ* × *κ*) for a player house in a skirmish, *h* = *ℓ* in a campaign mission, and
*h* = 1.0 for the neutral and special houses, whatever *ℓ* and *κ* hold.

**F3. Reload delay** (claim 11). Integers, ticks. With *A* the maximum ammunition, *a* the current
ammunition, *E* the empty-reload field, *L* the reload field, *I* the reload-increment field and *W*
the pip-wrap field:

> delay = *E* when *a* = 0 and *E* ≠ −1,
> otherwise delay = *L* + *I* × *q*², with *q* = 1 if *W* = 0, else *q* = trunc(*a* / *W*).

The product and sum are 32-bit integer arithmetic (S-0205 claim 1).

**F4. In-burst delay** (claim 4; used only when claim 2's order reaches it). After the *j*-th shot of a burst (1 ≤ *j* < *B*): the type's burst
delay *j* (an integer, ticks) for a vehicle when *j* ≤ 4 and that field ≠ −1; else R(3, 5). For a
berserk firer that value is then halved and truncated toward zero, as in the last step of Formula F1
(claim 8, which exempts a disc laser shot). Integer arithmetic throughout.

**F5. Charge-turret number** (claim 16; multi-image, non-gattling types only). *c* = min(*N* − 1, max(0, trunc(*N* × *r* / *D*))) when
*D* > 0 and the countdown runs; else 0. Integer arithmetic.

**F6. Starting ammunition** (claim 10). *a*₀ = *A* when *J* = −1, otherwise *J*.
All three quantities are signed 32-bit integers in rounds; this is an exact selection,
with no rounding. With `quirks.initial_ammo_above_capacity` on (the default), an explicit
*J* above a nonnegative *A* remains *J*. With the switch off, that result is capped at *A*.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Full-delay draw | R(0, 2) | ticks | Claim 2; one draw |
| In-burst delay draw | R(3, 5) | ticks | Claim 4; when no burst-delay field applies |
| Burst-delay fields read | 4 | fields | Claim 4; positions 1 to 4 of a burst, vehicles only |
| Firing sync values | 2 | burst-counter values | Claim 13; burst-counter values 0 and 1 |
| Building multi-round delay | 1 | ticks | Claim 10 |
| Veteran factor, absent-key default | 1.0 | factor | Import mapping |
| Occupy and bunker factors, absent-key default | 1.0 | factor | Import mapping |
| Difficulty-row and country factors, absent-key default | 1.0 | factor | Import mapping |
| Burst count, absent-key default | 1 | shots | Import mapping |
| Rate-of-fire field, absent-key default | 0 | ticks | Import mapping |
| Maximum and initial ammunition, absent-key default | −1 | rounds | Maximum −1 disables vehicle reloading; initial −1 selects the maximum (claim 10) |
| Empty-reload field, absent-key default | −1 | ticks | Import mapping; −1 = not used |
| Reload, reload increment, pip wrap, absent-key default | 0 | ticks, ticks, rounds | Import mapping |

## State and ordering

**A firer's firing state.**

| State | Event | Result |
|---|---|---|
| ready (no countdown running), or countdown running in claim 13's sync tick | an attempt passes the firing check and makes a shot | the shot is the *j*-th of its burst, *j* = *k* + 1; the countdown runs for the shot's delay *D*, of the kind that claim 2's order gives for *j*, with the factors of claims 3, 6 and 7 where that kind takes them and the halving of claim 8 where it applies; the burst counter is *j* mod *B* |
| ready | an attempt passes the firing check with a draining weapon | a drain attempt replaces the ordinary shot (claim 9; S-0608 claim 1); no countdown start, burst advance, ammunition use or ordinary-shot draws; target clearing still resets the burst counter |
| carrying aircraft with a first passenger | firing reaches a drop attempt | no ordinary shot, burst advance, delay draw, spray draw, report or muzzle animation; refusal retains countdown and ammunition, success expires countdown and spends one round immediately; run spending remains separate (S-0522 claim 11) |
| ready | an attempt passes the firing check, but its projectile cannot be put on the map (a void shot, claim 15) | nothing changes |
| ready, animation-delayed non-SAM building on the ordinary path | its attack behaviour gets *can fire*, outside an active upgrade gate | the delayed shot becomes pending (claim 14); no shot is made yet |
| ready, animation-delayed non-SAM building with active upgrade gate | *can fire*, gate passes | immediate primary and secondary attempts, no delayed shot started; an empty secondary does nothing (claims 9 and 14) |
| ready, animation-delayed non-SAM building with active upgrade gate | *can fire*, gate fails with the gate quirk on | no attempt and no delayed shot started (claim 14) |
| countdown running, outside claim 13's sync tick | an attempt | answer *rearming*, unless a reason of ranks 1 to 4 gives its own answer (claim 9); either way no shot and no draw |
| countdown running | *D* ticks have passed since the shot | ready |
| any | the firer's target is cleared | burst counter 0; countdown unchanged |
| building, delayed shot pending | its turn max(*d* − 1, 0) ticks after the turn that started the shot | firing check with range: a shot at the current target on *can fire*, otherwise dropped; no longer pending (claim 14) |

**Ordering consequences.**
- The delay draw of a shot is taken in the firer's turn, after the draws of the projectile it
  launches and before those of its muzzle-flash animation (claim 18); two firers that fire in the same tick take their draws in update order
  (S-0203).
- A shot made in tick *t* with delay *D* ≥ 1 allows the next shot in tick *t* + *D* at the
  earliest (claim 1), unless that next shot is a sync shot (claim 13) or another shot of the
  aircraft's same-tick full burst (claim 4). So measured shot intervals are *D* ticks when the firer attempts every tick
  and makes no sync shot.
- A building with a pending delayed shot reports *rearming*; under S-0215 claim 32 its attack
  behaviour is therefore processed every other tick (it keeps turning towards its target) until the
  shot. The delayed shot itself is timed by the building's own turns, one per tick, so its tick is
  the same whether or not a processing falls in it (claim 14).

## Edge cases and quirks

- **Building rounds** (`quirks.building_ammo_fast_rearm`, default on). A building whose ammunition
  is above 1 rearms in 1 tick after every shot (claim 10), over every other kind of delay. With the
  switch off, the delay is the one that claim 2's order gives without its first row, at every
  ammunition level. Backing: `measurable` (scenarios F9, F3 and F4, building parts); confidence
  medium.
- **Shared wave slot** (`quirks.shared_wave_slot`, default on). Sonic waves and magnetic-beam waves
  share one effect slot per firer (claim 5). With the switch off, each has its own slot and a live
  magnetic wave does not affect a sonic weapon. Backing: `measurable` (scenario F4, shared slot
  part); confidence low.
- **Bare-rate effect weapons and berserk.** A sonic weapon's delay is the bare field and is still
  halved when berserk (claim 8 applies after the choice). This is the original behaviour; no switch. Backing: `measurable` (scenario F6, bare-rate
  part), confidence high.
- **The campaign ignores the country factor** (claim 3). Historical original behaviour only.
  no campaign rule is specified for our engine. Backing: `unmeasured` here because campaigns
  are out of scope and no campaign run is proposed; confidence high.
- **Burst count 0 or less** (departure, no switch). The original fails on a burst count of 0 (a
  division by zero). For a negative count *B* it keeps going: after the shot at position *j* the burst
  counter is the remainder of *j* divided by |*B*|, a value from 0 to |*B*| − 1, but no position is
  below *B*, so every
  shot gets the full delay and never an in-burst delay. Our engine treats any burst count below 1
  as 1, and the importer warns, naming the weapon. Policy: bad data gives a safe deterministic
  result plus an importer warning. Backing: `unmeasured` for the original's negative-count
  behaviour (confidence medium): our engine departs from it, and no real data has a negative
  count, so a measurement would only document a behaviour we do not reproduce.
- **Negative or zero delays.** A rate-of-fire field of 0 or less, a negative house, veteran or
  bunker factor, or a burst-delay field below −1, can give a delay of 0 or below; the countdown is
  then over at once (claim 1). The importer warns for a negative rate-of-fire field, a negative
  factor, or a burst-delay field below −1. A bunker factor or an occupy factor above 0 and below 1
  lengthens the delay; no warning. Backing: `measurable` (scenario F16); confidence high.
- **Occupy factor not above 0, bunker factor 0.** Then that division is skipped (claim 7). An
  occupy or bunker value written as text that is not a number is read as absent (S-0003 claim 8,
  a departure of that spec), so the factor keeps its default of 1.0 and divides by 1 (backing `measurable`, scenario F5, skips
  part, confidence high). The original
  also skips the division for a factor that holds a not-a-number value; no imported text produces
  one, so there is no scenario for it (`unmeasured`, confidence low).
- **Starting ammunition above the maximum** (`quirks.initial_ammo_above_capacity`, default
  original). An explicit initial field above a nonnegative maximum supplies that many rounds,
  not the maximum. With the switch off, the starting count is capped at the maximum.
  Backing: `measurable` (scenario F9, starting-count part); confidence high.
- **Ammunition below −1.** A firer whose *a* is below 0 spends no rounds when firing
  (a round is used only when *a* > 0) and is never *out of ammunition*
  (that answer needs *a* = 0). Without reloading it therefore behaves as unlimited.
  vehicle reloading is still governed by claim 11. Backing: `measurable` (scenario F9, below-minus-one part), with
  confidence medium.
- **Firing sync frame at or above the counter's start.** The counter starts at 2*F* − 1 and only
  falls, so a sync frame *s* ≥ 2*F* is never reached: read literally, claim 13's gate gives the
  primary weapon *rearming* for the whole run of the counter, so a following shot comes only after
  the counter is idle again. Backing: `measurable` (scenario F11, beyond-the-start part, test art
  data); confidence low. The part also settles the first claim-13 open question.
- **Ammunition in reload arithmetic.** Formula F3 is integer arithmetic. A pip-wrap field below 0 gives a *q* of
  the opposite sign to *a*, or 0 when *a* is smaller than the field's magnitude, because the
  division is toward zero; the square of *q* is never negative. No warning. Backing: `measurable` (scenario F10, part f), confidence
  medium.

## Import mapping

| Key (file, section) | Concept | Form | Absent |
|---|---|---|---|
| `ROF` (rules, weapon) | rate-of-fire field *R* | integer, ticks | 0 |
| `Burst` (rules, weapon) | burst count *B* | integer | 1 |
| `IsSonic`, `UseSparkParticles`, `UseFireParticles`, `IsRailgun` (rules, weapon) | effect weapon kinds (claim 5) | boolean | no |
| `IsMagBeam` (rules, weapon) | magnetic-beam weapon (claim 5, Terms) | boolean | no |
| `Temporal` (rules, warhead) | temporal weapon; the primary weapon's warhead initially supplies the firer's temporal beam (Terms) | boolean | no |
| `Spawner` (rules, weapon) | spawner weapon (Terms, claim 9) | boolean | no |
| `HunterSeeker` (rules, vehicle type) | seeker type (Terms, claim 9) | boolean | no |
| `DiskLaser` (rules, weapon) | disc laser weapon (claim 8) | boolean | no |
| `DecloakToFire` (rules, weapon) | needs the firer uncloaked (claim 12) | boolean | yes |
| `BurstDelay` followed by 0, 1, 2 or 3 (rules, vehicle type) | burst delay *j* = 1 … 4 (claim 4) | integer, ticks | −1 |
| `FiringFrames` (art, vehicle image) | firing frames *F* (claim 13) | integer | 0 |
| `FiringSyncFrame` followed by 0 or 1 (art, vehicle image) | firing sync frames for burst-counter values 0 and 1 (claim 13) | integer | −1 |
| `Ammo` (rules, type) | maximum ammunition *A*, also the starting count when *J* = −1 (claim 10) | signed integer, rounds | −1 |
| `InitialAmmo` (rules, type) | initial-ammunition field *J*: −1 selects *A*, every other value is literal, with no cap by default (claim 10, Formula F6) | signed integer, rounds | −1 (selects *A*, not necessarily unlimited) |
| `Reload`, `EmptyReload`, `ReloadIncrement` (rules, type) | *L*, *E*, *I* of Formula F3 | integer, ticks | 0, −1, 0 |
| `PipWrap` (rules, type) | *W* of Formula F3 | integer | 0 |
| `ManualReload` (rules, type) | no reload restart on firing (claim 11) | boolean | no |
| `MobileFire` (rules, type) | reload advances while moving (claim 11); see S-0310 | boolean | yes |
| `DamageReducesReadiness` (rules, type) | claim 17 | boolean | no |
| `VeteranAbilities`, `EliteAbilities` (rules, type) | the entry `ROF` in either list sets the veteran or elite rate-of-fire ability (claim 6) | list | — |
| `VeteranROF` (rules, `General`) | veteran factor *v* | decimal (S-0003, claims 6 and 7), held as a double | 1.0 |
| `OccupyROFMultiplier`, `BunkerROFMultiplier` (rules, the combat-damage section) | *m*, *b* | decimal (S-0003, claims 6 and 7), narrowed to single precision | 1.0 |
| `ROF` (rules, the three difficulty-row sections `Easy`, `Normal`, `Difficult`) | difficulty-row factor *ℓ* (easy, normal, difficult row) | decimal (S-0003, claims 6 and 7), held as a double | 1.0 (see S-0006 for a layer that has the section but not the key) |
| `ROF` (rules, country section) | country factor *κ* | decimal (S-0003, claims 6 and 7), held as a double | 1.0 |
| `CompEasyBonus` (rules, `General`) | the computer-easy-bonus flag of claim 3 (the original's set-up only; read by the optional handicap setting) | boolean | no |
| `CanBeOccupied`, `CanOccupyFire` (rules, building type) | garrisoned building (Terms) | boolean | no |
| `IsAnimDelayedFire`, `DelayedFireDelay` (art, building image) | claim 14's ordinary delayed shot; a passing upgrade gate bypasses both | boolean; integer, ticks | no; 0 |
| `Primary`, `Secondary`, `ElitePrimary`, `EliteSecondary` (rules, type) | weapon references for slots 0 and 1 and their elite replacements; S-0308 claims 1, 2 and 12 own resolution; claim 9 here covers an empty resolved secondary | weapon type name | held value; engine default no weapon |
| `IsChargeTurret`, `TurretCount` (rules, vehicle type) | charge turret and *N* (claim 16) | boolean; integer | no; 0 |

The importer warns for a `Burst` below 1, a negative `ROF`, a negative factor, and a burst-delay
value below −1 (Edge cases).

## Measurement scenarios

Scenario labels F1 to F16 are separate from the formulas F1 to F6; the text says "scenario" or
"Formula" each time. All scenarios use stock data unless a named test rules file is listed, under the measurement setup
(glossary), a small purpose-made skirmish map, one game speed (any; claim 1 says it does not matter),
and the random-state recorder where draws matter. Unless a scenario states otherwise, each distinct
setup is run once; any explicitly stated repetitions or comparisons between runs take precedence.
"Shot ticks" means the tick numbers at which the firer's shots are made, read from projectile creation or the target's health drops.

**F1. Rearm countdown and full delay.**
- Setup: one human-house vehicle with a single-shot weapon (burst count 1, not an effect weapon) and
  a stationary enemy target building with high strength in range; the house uses the normal row; the country factor is 1.0 (test rules file sets it if needed). Run a second time at another
  game speed. In a third run, while a countdown runs, change the target to another building in
  range, and in a fourth give the vehicle an order to use its other weapon (test data: a second
  single-shot weapon in slot 1).
- Observable: shot ticks over 20 shots; the recorder's draw results for R(0, 2).
- Expected: each interval equals *D*₁ of Formula F1 for that shot's draw *u*; the first shot
  comes in the first tick in which the vehicle is facing the target; the intervals in ticks are the
  same at both speeds; the target change and the weapon change do not move the tick of the next shot.
- Falsified if: any interval differs from that value, the intervals change with game speed, or a
  target or weapon change shortens or lengthens the running countdown.

**F2. House and veteran factors.**
- Setup: as F1, with a test rules file that sets the house's normal-row factor to 1.5 and its
  country factor to 0.8 (expected *h* as Formula F2 gives it, about 1.2),
  and optionally computer houses at each lobby entry (easy, normal and hard computer: the original's
  set-up of claim 3 with the pairing of S-0412 claim 4, recorded and not reproduced by our engine; in
  these runs the rows' factors differ, and the expected *h* is the factor of the row that claim 3
  gives the entry times the country factor); a veteran vehicle of a type with the veteran
  rate-of-fire ability, and an elite one of a type with only the veteran ability; the test rules
  file also sets *v* = 0.5.
- Observable: shot intervals and draws.
  A further run sets *v* by test data, once written as 0.6 and once as 60%, for a weapon whose
  *D*₁ is 10.
- Expected: Formulas F1 and F2 exactly; elite gets the factor once. The 0.6 run gives *D*₂ = 6 (the single
  just above 0.6), the 60% run *D*₂ = 5 (a double just below 0.6 under the round-toward-zero mode).
- A further run (the non-player houses): the test rules file gives the three rows the factors 2.0,
  1.5 and 0.5 (easy, normal, difficult) and the neutral country the factor 1.5 (all exact in binary).
  A defence building of the neutral house with a rate-of-fire field of 40, not veteran, fires at a
  stationary target of the special house (S-0412 scenario K5 part d sets this up) over 20 shots.
  Expected: every interval 40, 41 or 42 ticks (*D*₁ with *h* = 1.0 and the draw *u*). Falsified if an
  interval lies outside 40 to 42, in particular at 60 to 62, 80 to 82 or 90 to 92 (a country factor
  or a row applied).
- Falsified if: the country factor does not act in the skirmish run, the
  skirmish intervals do not follow that *h*, the elite interval shows the factor twice, the 0.6 run
  gives a *D*₂ other than 6, or the 60% run gives one other than 5.

**F3. Bursts.**
- Setup: a vehicle whose weapon has burst count 3 (test rules file), then the same with burst delays
  1 and 2 (claim 4) set to 7 and 9; an infantry unit with a burst-3 weapon.
- Observable: shot ticks and draws.
- Expected: without burst-delay fields, intervals inside a burst are 3, 4 or 5 matching the draws,
  and after the third shot the full delay; with the fields, the vehicle's intervals are 7 and 9 with
  no draw; the infantry unit always draws R(3, 5). Clearing the target in mid-burst (stop order) and
  giving a new one starts a fresh burst. Changing the target (not clearing it) in mid-burst keeps
  the burst going.
- Carry-over: a vehicle with a burst-3 weapon in one slot and a burst-2 weapon in the other (test
  rules file), made to fire the first once and then the second. Expected: the second weapon's shot is
  position 2 of 2 of its burst, so its delay is the full delay.
- Building part: a building defence whose weapon has burst count 2 and is not an effect weapon, with
  3 rounds (test rules file). Expected: shots in ticks *t*, *t* + 1 and *t* + 2, and no R(3, 5)
  draw after the first two (claim 2's order puts the 1-tick delay first); the third shot is made
  with the last round and is the first of a new burst (*j* = 1 < *B*), so the delay after it is
  one R(3, 5) draw, not a full delay (the recorder shows it; F9's observation of when the
  building drops its target agrees). With 4 rounds instead, the fourth shot is *j* = 2 = *B* and
  its delay is the full delay, with an R(0, 2) draw. Falsified if the second shot comes 3 to 5
  ticks after the first, if no R(3, 5) draw follows the third shot of the 3-round run, or if an
  R(0, 2) draw follows it.
- Falsified if: in-burst intervals use the full delay, the counter is not reset by the stop, a
  change of target resets it, or the carried count does not decide the second weapon's delay.

**F4. Effect weapons.**
- Setup: a stock vehicle with a sonic weapon; a stock unit with a fire-particle or spark-particle
  weapon; a stock railgun weapon if available, else a test rules file.
- Observable: shot ticks; draws.
- Expected: no delay draw is taken; intervals are max(*R*, effect lifetime).
- Falsified if: a draw is taken for the delay, or an interval differs from
  max(*R*, effect lifetime) when facing, range and all other firing gates remain satisfied.
- Burst part (claim 2's order): with a test rules file, the sonic vehicle's weapon gets burst count
  2, and its *R* is longer than the wave's lifetime. Expected: after every shot, including the
  first of a burst, the interval is *R* and the recorder shows no R(3, 5) draw. Falsified if the
  interval after the first shot is 3 to 5 ticks (a wave still alive would then answer *rearming*
  for the weapon's own kind, so the shot ticks show it) or a draw appears.
- Building part: with a test rules file, a building defence that cannot be garrisoned, with a sonic weapon
  (or a spark-particle weapon), *R* far longer than the effect's lifetime, and 3 rounds. Expected:
  the second shot comes when the first wave ends, not *R* ticks after the first (a 1-tick delay
  with the effect gate holding it); the delay after the shot made with the last round is *R*, so,
  as in scenario F9, the building keeps its target and turns (S-0215, claim 32, second row) until
  *R* ticks after that shot, and drops it at its first processing after that. Falsified if the
  second shot comes *R* ticks after the first, or the building drops its target before *R* ticks
  after its last shot.
- Shared slot: with test data, a vehicle that carries a sonic weapon and a magnetic-beam weapon,
  the magnetic weapon's full delay (its rate-of-fire field plus up to 2) set shorter than the
  wave's lifetime (the effect specs give it), so that the rearm countdown is over while the wave
  is alive and only the wave can hold the sonic weapon back. Fire the magnetic one, then try the
  sonic one while the magnetic wave is alive. Expected: no sonic shot until the wave ends.
  Falsified if the sonic weapon fires while the wave is alive. Control: with
  `quirks.shared_wave_slot` off it fires at once.
- Magnetic control: named test map data supplies a magnetic-beam weapon with an ordinary,
  non-seizing warhead and a full delay shorter than the wave's remaining lifetime. Use a
  stationary firer and durable target building, speed index 2, director and other mods off.
  Keep the target unchanged and all other firing gates satisfied. Record shot ticks, grip
  presence and live wave count with tick-labelled capture and a read-only recorder, for three
  runs. Expected: no grip; repeated shots follow their claim-2 delays while the first wave
  remains alive, with no second wave. Falsified by an own-wave-only firing refusal or a new
  wave on those later shots. S-0607 scenario L9 parts c and d separately tests the conditional
  refusal while gripping and the continued grip after retargeting to a building.
- Other weapon: with test data, a firer with a sonic weapon in slot 0 and an ordinary weapon in slot
  1, and a firer with the two the other way round, the sonic weapon's rate-of-fire field set below
  the wave's lifetime so that the rearm countdown is over while the wave is alive. Fire the sonic
  weapon and ask for the ordinary one while the wave is alive. Expected: the ordinary weapon does
  not fire until the wave ends (the check answers *impossible*), and fires in the first attempt
  after it. Falsified if it fires while the wave is alive.

**F5. Garrisons and bunkers.**
- Setup: a garrisonable building with 1, 2 and 4 occupants of one infantry type attacking a target,
  a vehicle inside a tank bunker; the stock occupy and bunker factors.
- Observable: shot intervals of the building and the bunkered vehicle.
- Expected: Formula F1 with *n* and *m*, and with *b*.
- Skips (test rules file): an occupy factor of 0 and a negative one, and a bunker factor of 0.
  Expected: each division is skipped, so the delay is the one without it. Falsified if a division by
  zero or a changed delay appears.
- Falsified if: the intervals do not scale as Formula F1 says.

**F6. Berserk.**
- Setup: a vehicle made berserk by a stock berserk warhead, firing at a nearby unit.
- Observable: shot intervals and draws.
- Expected: half the claim-2 delay, truncated. A further run, with test data giving the berserk
  vehicle a disc laser weapon: the interval is the full delay.
- In-burst and bare-rate parts (test rules file): a berserk vehicle with a burst-3 weapon and no
  burst-delay fields, and a berserk vehicle with a sonic weapon. Expected: the in-burst intervals
  are 1 or 2 ticks (the R(3, 5) draw halved and truncated, matching the draws); the sonic interval
  is max(trunc(*R* / 2), the wave's lifetime). A bare rate of 1 gives a halved value of 0, so only
  the wave's lifetime holds it.
- Falsified if: the interval is the full delay or rounds up, the disc laser interval is halved,
  or an in-burst or bare-rate delay is not halved.

**F7. Precedence of answers.**
- Setup: (a) a building defence whose countdown runs when its target moves out of range,
  (b) a vehicle with *A* = 1 whose countdown runs when its round is spent; (c) a cloaked unit whose
  weapon needs decloaking, and, with test data, an aircraft with the same weapon and cloak; (d) a deploy-to-fire vehicle on a sloped cell (S-0310, scenario T8), with
  a target outside its aiming tolerance.
- Observable: whether the building keeps its target and turns (S-0215, claim 32, second row) until
  its countdown ends; in (b), the tick at which the combat specs' consequence of *out of
  ammunition* (S-0310 claim 9 and S-0215, for example the vehicle giving up the attack or going
  to reload) starts, which is read from the vehicle's order state and position; when the cloaked
  unit uncloaks; and,
  in (d), the vehicle's turret or body facing.
- Expected: (a) the building keeps the target until the countdown ends and drops it only at the next
  processing after; (b) the consequence of *out of ammunition* starts only after the countdown; (c) the unit
  uncloaks before its first shot, and the aircraft does so only while fully cloaked, not while
  cloaking or uncloaking; (d) the vehicle does not re-aim, as the vehicle gets *must deploy* and not *not
  facing*.
- Falsified if: (a) the building drops its target while its countdown runs; (b) the consequence of
  *out of ammunition* starts while the countdown still runs; (c) the unit fires while in any cloaking state other than
  fully visible; (d) the vehicle re-aims.

- (e) Empty passenger carrier: use the named facing-test override of S-0501 scenario
  AC11 on a small skirmish map, speed index 2, director and other mods off, with a
  bystander computer player outside the test area under the glossary's void rule.
  Place a controllable non-fighter aircraft loaded, then unload every passenger. Give
  it ample ammunition and a durable enemy ground target. Compare a fresh aircraft of
  the same type placed empty. Repeat three times, recording ammunition, passenger count,
  aiming facing, shot ticks and the firing-check answer before the attack behaviour
  responds, using tick-labelled capture and a read-only recorder as in AC11.
  Test the emptied marked carrier with: (i) countdown running and aim wrong, (ii) countdown
  over, target out of range and the attempt asking for range, (iii) countdown over and
  target in range but aim wrong, and (iv) countdown over, in range and correctly aimed.
  Its last successful passenger drop expires its countdown (S-0522 claim 11), so does not
  supply case (i). Test that case only if normal inputs establish an emptied marked carrier
  with a running countdown; otherwise report it inconclusive. Keep all other refusal causes absent. Expected answers are respectively *rearming*,
  *out of range*, *out of ammunition* and *out of ammunition*, with no ordinary projectile
  shot in any case. The fresh empty control answers *not facing* in case iii and may fire
  in case iv. A different recorded answer, or ordinary projectile fire from the emptied
  marked carrier, falsifies the corresponding part. A state or range-request combination
  not reached through normal orders is inconclusive, not a passing trial.

**F8. Facing only when ready.**
- Setup: S-0206's turret scenario with a target that moves round the vehicle during its rearm period.
- Observable: the turret facing every tick.
- Expected: the facing-error re-aim (the facing-error row of S-0206 claim 9, fed by the replies of S-0310 claim 9) acts only in ticks when the weapon is ready.
- Falsified if: the re-aim acts during the rearm period.

**F9. Ammunition.**
- Setup: test rules file: a building defence whose weapon has burst count 1 and is not an effect
  weapon, with maximum ammunition 3 and initial ammunition 3, and an infantry unit with maximum and initial ammunition 2 (both keys set, so the result does not
  depend on how a new object's ammunition is derived).
- Observable: shot ticks, the number of shots, and when the building drops its target.
- Expected: the building fires in ticks *t*, *t* + 1 and *t* + 2 and never again. It keeps its
  target and turns (S-0215, claim 32, second row) until the full delay after the third shot has
  run, and drops it at its first processing after that. The infantry unit fires twice and never
  again.
- Falsified if: the building fires a fourth time, its intervals differ, it drops its target while
  its countdown runs, or either regains rounds.
- Starting-count part (Q-0060 question 1): a named test rules file sets a buildable vehicle,
  infantry and ungarrisoned defence building to maximum ammunition 4. Remove the initial field
  from every rules layer for those types. Use a small isolated skirmish test area, speed index 2,
  director off and other mods off, with one bystander computer player kept outside the area
  under the glossary's void rule. Repeat with an explicit initial field of −1, 0 and 6.
  also test maximum 0 with the initial field absent, and maximum −1 with initial field 0.
  Supply a single-shot ordinary weapon and a durable stationary enemy target in range.
  Set the vehicle's ordinary and empty reload delays longer than the observation window,
  leave manual reload off, and exclude damage, servicing and ammo-changing effects.
- Procedure: in each case compare one map-placed object with one produced and released by the
  human house. Map-placed vehicles and infantry belong to the human house; map-placed
  buildings belong to the neutral house, since the original skips human-house buildings
  placed by a skirmish map (S-0009). Give each a target hostile to its own house.
  Order sustained fire where orders apply, and let the neutral building acquire its target.
  Record projectile creation ticks by
  tick-labelled video, counting shots until no shot occurs for several full delays, before
  the vehicle's reload can finish. Run each case once for each creation method and kind.
  For the aircraft starting count, compare map-placed and produced aircraft of a test type
  with the same field pairs, using its ammunition pips before takeoff or servicing; if pips
  cannot distinguish an overfull count, report that observation as inconclusive rather than
  interpreting a full display as a cap.
- Expected: absent or explicit −1 with maximum 4 gives four shots. Initial 0 starts empty:
  infantry and buildings make no shot; the vehicle regains one round in its first turn
  (claim 11's never-started countdown), so makes one shot before the long reload wait.
  Initial 6 gives six, not four. Maximum 0 with the initial field absent gives none.
  Maximum −1 with initial 0 also gives none. Map placement and production agree.
  Aircraft start with the same counts under Formula F6. Falsified by another count,
  an unlimited result for absent initial ammunition with a finite maximum, a cap at four
  in the six-round case, or a difference between creation methods. An aircraft count not
  distinguishable through pips remains an unconfirmed part of the proposed measurement.
- Below-minus-one part: test rules file, an infantry unit with maximum and initial ammunition −2
  (both keys set), firing at a durable target for 20 shots. Expected: it never answers *out of
  ammunition*, fires every full delay as an unlimited unit does, and shows no ammunition change.
  Falsified if it stops firing, or its ammunition changes.

**F10. Reloading.**
- Setup: test rules file: a vehicle with maximum and initial ammunition 4 (both keys set), reload
  field 50, reload increment 10, pip wrap 2 and empty-reload field 120. Parts: (a) fire all rounds, then wait. (b) Fire one round
  every 30 ticks. (c) As (a), with the mobile-fire flag off, while driving. (d) A type with the
  damage-reduces-readiness flag that is shot. (e) A type with the manual-reload flag, maximum
  and initial ammunition 4 (both keys set): it fires once. (f) As (a), with pip wrap −2.
- Observable: the ammunition pips over time.
- Expected: (a) the first round comes back 120 ticks after the last shot (*a* = 0 uses the
  empty-reload field), the second 50 ticks later (*a* = 1: *q* = 0), the third and fourth 60 ticks
  apart (*a* = 2 and 3: *q* = 1). (b) No round comes back while the vehicle fires every 30 ticks, for as long as it has rounds.
  (c) No progress while it drives. (d) Rounds are lost and the countdown restarts. (e) The round comes back in the first turn after the shot, not a reload delay later, since its countdown was never started (it held all 4 rounds, so nothing started it before the shot). (f) The same delays as (a): 120, 50, 60 and 60 ticks.
- Falsified if: any delay differs from Formula F3, a round comes back while the vehicle fires every 30
  ticks, the countdown advances while it drives (part c), the damaged vehicle in part (d) keeps
  all its rounds or does not restart its countdown, or the manual-reload vehicle of part (e) does
  not regain its round in the first turn after its shot (never regains it, or only after the
  reload delay), or part (f) differs from part (a).

**F11. Firing frames and sync.**
- Setup: test art file: a sprite vehicle with 4 firing frames and firing sync frame 3 for
  burst-counter value 0, a weapon with burst count 1 and a long rate-of-fire field.
- Observable: shot ticks.
- Expected: the first shot starts the counter at 7; the next shot comes in the vehicle's turn in
  which the counter equals 3 (4 turns later), not at the rearm delay; the shot after that follows
  the rearm delay again.
- Falsified if: the shot ticks follow the rearm delay only.
- Beyond-the-start part: test art file, the same vehicle with firing sync frame 8 (2*F*) for
  burst-counter value 0, and a rate-of-fire field of 2. Expected, under the literal gate of
  claim 13: after a shot the counter runs from 7 to idle, the primary weapon answers *rearming*
  throughout, and the next shot comes only after the counter is idle, so intervals are at least 8
  ticks although *D* is about 2. Falsified if intervals of about *D* ticks appear while the counter
  runs (the gate does not hold the shot back at all).
- Tunnel part: test map with a tunnel pair; the vehicle of the main part makes a shot (counter
  7), and in the next tick is ordered through the tunnel (transit of *T* ticks), then ordered to
  attack again. Expected: the counter does not fall during the transit, so the next shot is made
  *T* ticks later than it would be without the tunnel. Falsified if the transit turns count
  (the shot comes as without the tunnel).

**F12. Animation-delayed building.**
- Setup: a stock building with the animation-delayed-fire flag (or a test art file giving it a
  delayed-fire field of 10) attacking a target.
- Observable: tick of the attack decision (turret stops on target) and tick of the shot.
- Expected: the shot comes max(*d* − 1, 0) ticks after the turn that started the animation.
- Falsified if: it comes at another tick.
- Pending and dropped parts: (a) while the shot is pending, the building keeps its target and turns
  to it, and no second animation starts. (b) The target is moved out of range, or killed, between
  the start and the shot (a test unit ordered away, or a second attacker). Expected: no shot is
  made, the delayed shot is no longer pending, and no rearm countdown has started, so the
  building's next *can fire* starts a new animation at once. Falsified if a shot is made at the
  old target, a countdown runs after the dropped shot, or the building stays pending.

**F13. Charge turret.**
- Setup: a stock charge-turret vehicle firing repeatedly.
- Observable: the turret image index each tick, and which weapon (the projectile type) each shot uses.
- Expected: Formula F5 for the image; every shot uses the slot-0 weapon.
- Falsified if: the index does not follow the remaining countdown, or any shot uses a weapon other
  than the slot-0 weapon.

**F14. Draw order.**
- Setup: the recorder; a vehicle whose projectile takes a known number of draws (S-0306) firing
  alone on the map.
- Observable: the recorded raw values taken in the firing tick.
- Expected: the projectile's draws, then one R(0, 2) (or R(3, 5) inside a burst), then the
  muzzle-flash animation's draws (S-0202, claim 10).
- Falsified if: a second delay draw appears, or the delay draw comes before the projectile's.
- Spray and particle part: a spray-attack type's first shot of a burst (test data), and a
  fire-particle weapon's first shot. Expected: the spray pick comes before the projectile's draws
  and its delay draw follows them. For the fire-particle weapon, the particle system's draws
  come after the projectile's draws, with no delay draw at all (claim 5). Falsified if either
  ordering differs, if a delay draw appears for the fire-particle shot, or if the spray pick
  appears for any shot other than the first of a burst.
- Refusal part: an attempt refused by the firing check while the countdown runs, in each of the
  ticks of the countdown. Expected: no draw of the stream in those ticks from the firer. Falsified
  if any appears.
- Void-shot part: in S-0306's scenario P5 run d the recorder shows the projectile's draws, if the
  projectile takes any, and no delay draw; the countdown, burst counter and ammunition are
  unchanged. Falsified if a delay draw is taken or the countdown starts.

**F15. The named reasons of the rank table.** Each part checks that a shot is not made while
one reason holds, with every other reason absent (the rearm countdown over, the target in range
and in the aiming tolerance, ammunition above 0), and that the same firer shoots when the reason
is absent (the control).
- Observable: in each part, whether and in which tick the firer makes a shot (projectile creation
  or the target's health drops); in (d)(ii), whether a spawned aircraft is launched; in (e), also
  the vehicle's behaviour.
- (a) Death frames (rank 1): test art file giving a sprite vehicle a death frame count of 20 or
  more and a start frame. The vehicle is ready with a target in range; a strong enemy hit destroys
  it. Expected: no shot is made from the tick its death counter starts. Falsified if a shot is
  made while the death frames play. If the vehicle takes no further turns once destroyed, the
  record says so and the part stays open.
- (b) Melee link (rank 3): S-0214's scenario part C, second case (an enemy shooter with a
  link-flag weapon links itself to an own vehicle), with a second enemy in range of the vehicle's
  ready weapon. Expected: the vehicle makes no shot while the link holds; the same vehicle fires
  at the second enemy before the link forms. Falsified if it fires while linked.
- (c) Tether (rank 12): an armed vehicle that docks at a refinery (a stock armed harvester if the
  data has one, else test data giving a harvester a weapon), with an enemy in range while it
  unloads. Expected: no shot from the tick the tether begins until it ends. Falsified if a shot is
  made while it is tethered.
- (d) Early *rearming* (rank 4): (i) use the armed test vehicle and distance/delay controls of
  S-0207 scenario D22 runs 1 and 5, teleporting by its own locomotor, with a stationary enemy
  target in range at the landing point. Record the jump tick, the end of phasing in and shot
  ticks. Expected: no shot while materialising, which ends exactly as S-0207 claim 20(f) and
  Formula F9 specify, including the next-tick end for a nonpositive delay. Once that state ends,
  it adds no firing restriction of its own; other firing checks and the timing of attempts still
  apply. Falsified by a shot during phasing in or by materialising ending at a different tick.
  (ii) a spawner weapon none of whose spawned aircraft is
  alive: no launch until one is alive; (iii) a temporal weapon whose beam holds the target is
  covered by S-0310's scenario T6. Falsified if a shot or launch is made while the condition
  holds.
- (e) Seeker type (rank 9): a seeker type (stock if the data has one, else test data) with its
  target well inside the weapon's range and everything else passing. Expected: the attempt is
  refused although the target is in range, so the vehicle does not make an ordinary shot and
  shows the behaviour S-0215 gives to *out of range*. Falsified if it makes an ordinary shot.

- (f) Hidden targets (rank 4): a named test map, speed index 2, director and other mods off,
  one allied and one enemy bystander computer player outside the observed area under the
  glossary's void rule. Its override sections supply a self-cloaking target and an armed vehicle
  with two ordinary weapons whose signed damage-plus-ambient sums give positive, zero and
  negative mean weapon damage in separate runs (S-0308 claim 15). Keep the weapon being
  attempted unchanged while changing only the other weapon's contribution. Attempt forced
  attacks on an own hidden target, and attacks on allied and enemy targets, with and without
  the firer's own sensor coverage; repeat each case three times. Where target selection is
  unavailable, assign the target before cloaking and record target loss separately rather than
  treating it as a firing refusal.
  Expected: without sensing, no shot at the hidden own or allied target when the mean is
  positive, and no shot at a hidden enemy whatever its sign. A nonpositive mean permits the
  own or allied attempt if no other restriction holds. Sensing removes this refusal. Record
  shot ticks, target retention and health changes by tick-labelled capture and a read-only
  recorder. Falsified by a shot in a refused case or by a refusal that follows the unchanged
  attempted weapon rather than the changed mean. A lost target makes that trial inconclusive.
  Repeat with a building cloaking inside its own field: the refusal starts at fade level 11,
  not only at level 15 (S-0516 claim 18).

**F16. Delays at the edges.**
- Setup: test rules file; a single-shot vehicle weapon, a target of high strength in range.
- Observable: shot ticks and the recorder's draw results for R(0, 2).
- (a) Rate-of-fire field 0, and −5. Expected: with 0, the interval is the draw *u* of R(0, 2),
  but never below 1 tick since the vehicle attempts once per tick (intervals 1, 1 and 2 for draws
  0, 1 and 2); with −5, a shot in every tick the vehicle can attempt. Falsified if a gap of 2 ticks or
  more appears with −5, or the 0 run's intervals do not follow the draws.
- (b) A bunkered vehicle (S-0206) with a bunker factor of 0.5 and of 0: Formula F1's *D*₅ is double
  *D*₄ for 0.5 (the delay lengthens) and equals *D*₄ for 0 (the division is skipped). Falsified
  if the 0.5 run shortens the delay.

**F17. Upgrade pair with an empty secondary and animation delay (Q-0798).**
- Setup: tiny skirmish test map, speed index 2, 800 × 600, director and other mods off,
  one bystander computer player outside the test area under the glossary's void rule.
  Named map overrides supply a non-SAM, ungarrisoned host and an upgrade in position 1
  with a primary ordinary projectile weapon of burst count 2 and no secondary. Remove
  every host and upgrade secondary, including elite replacements, and keep ammunition
  unlimited. Use a durable stationary enemy target in range and correct aim. Set the
  host's animation-delay field to 20 ticks; compare its animation-delay flag off and on.
  Add a passing control with a distinct, single-shot secondary and a failing-gate control
  whose upgrade primary burst count is 1, with no secondary.
- Procedure: install the upgrade through normal placement and order sustained attack.
  Run each fixture three times. With tick-labelled capture and a read-only recorder,
  record projectile creation, burst counter, rearm delay, ammunition, delayed-shot pending
  state and random-stream draws at each attack processing. Stop before other objects
  affect the target. No injection or state writes are permitted.
- Expected: with the passing gate and empty secondary, each processing that gets *can fire*
  makes only the primary shot. Its first shot leaves burst counter 1 and one in-burst
  delay draw; its next shot finishes that burst and gets the full delay. There is no
  extra burst step, delay draw, countdown replacement or ammunition change from the
  missing secondary. Enabling animation delay neither postpones these shots nor starts
  a pending shot. With the populated secondary both attempts occur in the same processing,
  not 19 ticks later. With the failing gate and original quirk setting neither shots nor
  a pending delay appear; on our engine with the gate quirk off, the failing control
  instead starts the ordinary delayed single shot. A no-upgrade ordinary control uses
  scenario F12's delay. The original cannot test our engine's quirk-off alternative.
- Falsified by any extra shot or state/draw change attributable to the empty secondary,
  an immediate-pair fixture becoming delayed, or a failed original gate starting a delay.
  Record unreachable fixtures as inconclusive rather than inferred confirmations.

## Open questions

- Claim 9 lists only the members of *illegal* and *impossible* that matter here; the combat specs
  named there and the feature specs own the rest. The precedence among those is not stated.
- The amount of ammunition lost to damage (claim 17) is S-0302's.
- Aircraft ammunition spending and servicing at landing pads are specified by S-0501 claims
  15 and 26 to 28; starting ammunition is answered by claim 10 (Q-0060 question 1).
- Whether a temporal weapon's firing side restarts the rearm countdown when its beam lets go
  remains open. Beam release itself does not establish such a restart, and
  no release-time rearm rule is claimed here.
- Claim 13: once a running firing-frame counter has passed the sync frame *s* (after the sync shot,
  or when no shot was made in that tick), does an attempt with the primary weapon still get
  *rearming* from the gate until the counter is idle, or does the rearm countdown alone decide?
  Scenario F11's main part cannot tell the two apart while the rate-of-fire field is long; its
  beyond-the-start part, with a short field, can for a sync frame that is never reached.
- Claim 13: does the counter first fall in the turn of the shot that starts it or in the next
  one? The sync shot of F11 comes at the same tick either way, but the tick at which the counter
  goes idle differs by one.
- Claim 9, rank 16: which of the answers of claim 9 the locomotor's own refusal gives (S-0310,
  claim 7).
