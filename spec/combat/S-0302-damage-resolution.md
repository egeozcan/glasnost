---
id: S-0302
title: Damage resolution (shot damage, defences, versus values, rounding, healing, health thresholds)
topic: combat
revision: 43
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-05
measurements: []
requests: [Q-0061, Q-0075, Q-0081, Q-0170, Q-0648, Q-2626]
supersedes: []
---

# S-0302: Damage resolution (shot damage, defences, versus values, rounding, healing, health thresholds)

## Summary

How much health one hit takes from one object, and what result it reports: the shot damage built on
the firing side, the target's defences, the versus value and the rules that stop a hit, and the
health thresholds, for infantry, vehicles, aircraft, buildings and terrain objects. Which objects an
explosion reaches is S-0303's, what small versus values mean for targeting is S-0304's, and the
armour classes are S-0305's. This spec covers skirmishes; campaigns and network games are out of scope.

A voxel-model debris piece's hits have no attacker and no attacking house (claim 1). A hit without
a warhead is covered by claims 6 to 8: no row of claim 8 that reads the warhead holds for it, it
can cause readiness loss (claim 10) but no berserk effect, and one that does not ignore defences
takes no health except the 1 that claim 7 takes from an undemolishable building.

## Terms

Shared terms used here are defined in the glossary: *armour class*, *warhead*, *weapon*,
*house*, *human-controlled*, *computer-controlled*, *AI house*, *difficulty skill block*, *country*,
*being erased*, *known attacker*, *disguise*, *scenario random stream*, *round-toward-zero mode*,
*departure*, *importer warning*, *quirk switch*, *field*, *lepton*, *tick*, *infantry*, *vehicle*,
*aircraft*, *building*, *foundation*, *contact*, *ammunition*, *removed*, *in play*,
*health, strength*, *health ratio*, *versus value*, *hit*, *type*. S-0205's notation applies: trunc, RZ₅₃, RZ₂₄, *single* and
*double*, and RN₅₃ and RN₂₄, the round-to-nearest counterparts that replace RZ₅₃ and RZ₂₄ when
`quirks.chop_rounding` is off (Formulas). In addition:

- **Inert game.** A game whose map sets the inert special flag (Import mapping). Detonations reach
  no object in it (S-0303, claim 2), and a hit from any other source that does not ignore
  defences, a sonic wave for example, has a modified amount *m*₀ of 0 (claim 6).
- **Killed.** An object is *killed* by the hit that takes its last health (claim 14): that hit
  reports *now dead*, or *post-mortem* when it starts a delayed kill (claim 16). A cyborg that
  claim 15 cripples instead is not killed. A killed object can stay in play for a time (S-0203,
  claim 6; claim 16 here) and stays killed.
- **Tag, trigger tag.** The tag that a map can attach to an object so that triggers can act on it
  (S-0011; the glossary's *cell tag* is the same kind of tag attached to a cell).
- **Terrain object.** A tree or similar scenery object that stands in a cell and has health
  (S-0102 draws them; S-0305 gives the armour class that its type has). Claim 20 is its target-side
  rule.
- **Turn.** In this spec "turn" means only the glossary's second sense, an object's share of one
  tick (S-0203, Terms): claims 16 and 19 use it. A rotation is written with facing words.
- **Height test for mutation hits.** Claim 17 uses the glossary's *height above the surface*,
  measured in integer leptons, and asks whether it is strictly positive. It does not use the
  airborne-state test of S-0308 (Terms), shared by S-0303 and S-0309. A standing infantry unit
  on a high bridge deck has height 0 above the surface and does not meet claim 17's test.
  The existing switch name `quirks.airborne_mutation_hit_raised` is retained; its scope is
  this height test, not the airborne-state test.
- **Demolition warhead.** The one warhead that the rules name in their combat-damage section as the
  warhead of destruction (Import mapping; S-0011, claim 27 and S-0303, claim 19 give its
  uses). It is the warhead of the trigger kill and of the ore burst of claim 20.
- **Trigger kill.** The trigger action of S-0011 (claim 27) that damages every object carrying the
  firing trigger's tag by its whole current health, with defences ignored and with no attacker
  and no attacking house (claim 1).
- **Vehicle factory.** A building whose type has the weapons-factory flag (Import mapping); the
  structures and production specs own what it does. Claim 8 has one row about a vehicle that has
  not yet left one.
- **Difficulty row.** The entry of the *difficulty skill block* (glossary) for one difficulty: one
  of the three rules sections *easy*, *normal* and *difficult* (Import mapping), which give the
  factors of that difficulty. The lead's decision on which row a house uses is recorded in the
  glossary's *difficulty skill block* entry and applied in claim 3: in a skirmish every player house
  uses the normal row unless the player's optional handicap setting picks another (claim 3), and a
  row that a map sets for a house is ignored. The neutral and special
  houses have no row at all (claim 3). The original's computer-easy-bonus shift uses the
  glossary's *human participant*, which includes observers (S-0412 claim 4); it never occurs
  in a skirmish, which has at most one human participant.
- **Versus value *v*.** The glossary's *versus value* (a double), written *v* in the formulas.
- **Hit.** One delivery of an amount of damage, with a warhead or (for some sources, claim 1)
  none, to one object, carrying the two
  markers of claim 1. A projectile's detonation delivers one hit to every object it reaches
  (S-0303); other sources deliver hits too (claim 1).
- **Delivered damage *d*₀.** The integer amount a hit delivers, before the target's side of this
  spec. For an ordinary projectile detonation it is the projectile's damage when it detonates:
  the shot damage of
  claim 2, less whatever S-0306 (claim 18) took off it in flight, then scaled for prism support
  when applicable (S-0518 claim 9 and F1). A disc-laser detonation instead delivers its held shot
  damage unchanged (S-0615 claims 12 and 16). Special effects that replace area damage deliver no
  ordinary hit; electric charging is one such effect (S-0518 claim 11).
- **Distance *r*.** The integer distance in leptons from the detonation point to the target that
  S-0303 gives with each hit; 0 for a hit that has no detonation point.
- **Damage that ignores defences.** The glossary's term; the hit carries the *ignore-defences
  marker*. Claim 1 says which hits carry it, and claim 9 which rules it skips.
- **No-escape marker.** The second marker a hit can carry (S-0312 uses it). It is independent of
  the ignore-defences marker, and no rule of this spec reads it: its only effect known is that a
  destroyed vehicle leaves no crew (S-0312, claim 13). Claim 1 says which sources set it.
- **Attacker, attacking house.** A hit may come with the object that caused it (the *attacker*,
  the glossary's *known attacker*) and with a house to credit (the *attacking house*); either may be
  absent. Claim 1 says which sources give which.
- **Temporarily invulnerable.** The state an iron curtain or force shield effect puts an object in
  for a limited time (the superweapon specs own how it starts and ends).
- **Prone.** The lying-down state of an infantry unit (S-0107, the *go-prone* sequence and the
  crawling sequences).
- **Cyborg.** As S-0107 (Terms): an infantry type flag.
- **Bunkered unit, tank bunker.** A *tank bunker* is a building that a vehicle can drive into for
  protection; a vehicle inside one is *bunkered* and linked to it (the structures specs own
  entering and leaving).
- **Garrisoned building.** A building whose type lets infantry occupy it and fire from it, while
  at least one occupant is inside (the structures specs own occupation).
- **Open-topped transport.** A transport whose passengers fire from inside it (the transport specs
  own which ones).
- **Closed transport.** A transport that is not open-topped: its passengers are off the map and out
  of play (glossary, *in play*).
- **Sonic-wave weapon, fire-particle weapon.** A weapon whose sonic flag is on, or whose
  fire-particles flag is on (Import mapping). Its damage is delivered by a sonic wave or by fire
  particles, which the wave and particle specs own, not by its projectile (claim 2).
- **Start to cloak.** An object able to cloak starts to cloak when it begins to become hidden from
  other houses.
- **Crate.** An overlay (glossary) that gives a power-up to the unit that picks it up.
  S-0520 owns crates (claim 4).
- **Veteran firepower bonus, veteran armour bonus.** The two veterancy abilities (glossary) that
  apply the veteran combat factor to a firer's shot damage (F1) and the veteran armour factor to a
  hit on the target (F2). S-0311 owns which ranks grant them.
- **Undemolishable building.** A building whose type has the can-be-demolished flag off (Import
  mapping). The flag gates the charge cursor and some starts of sabotage (S-0519 claim 12),
  but charge planting at arrival does not test it (S-0519 claim 13).
- **Defence building.** A building whose type's build category is *combat* (Import mapping).
- **Laser-fence section, bridge repair hut.** Building types marked by the type flags of those names
  (Import mapping); the structures specs own what they do.
- **Particle behaviour.** The behaviour word of a particle system type (*smoke*, *gas*, *fire*,
  *spark* or *railgun*, Import mapping); the particle specs own what each does.
- **Damage-state result.** The outcome a hit reports: *unaffected*, *unchanged*, *now yellow*,
  *now red*, *now dead* or *post-mortem* (claim 12).
- **Readiness loss.** The loss of *ammunition* (glossary) that a hit causes to a target whose type
  has the damage-reduces-readiness flag (claim 10). S-0301 owns what ammunition is and how it is
  spent and regained (claims 10 and 11), and the restart of the reload countdown (claim 17). The
  word "readiness" appears here only in this phrase, in the names of that flag and its factor, and
  in two quirk switches.

## Behaviour

### Where hits come from

1. **Hits, their two markers and where they come from.** Every loss or gain of health in combat is
   a hit with a delivered amount *d*₀, a warhead (absent for some scripted sources, and for the
   ground-impact hits that S-0317 claim 7 says a debris piece of a type with no warhead still
   deals; the end of such a piece delivers no hit, S-0303 claim 2), a distance *r*, an optional
   attacker and attacking house, and two yes/no markers: *ignores defences* (what it changes is
   claim 9) and *no-escape* (Terms). The table gives the markers, the attacker and
   the attacking house of each source known to deliver hits; each owning spec says what else its
   source delivers. The table lists the sources that the specs written so far know about, not
   every source the original has.

   | Source (owner) | Ignores defences | No-escape | Attacker | Attacking house |
   |---|---|---|---|---|
   | A projectile's detonation, every hit of its area (S-0303) | no | no | the firing object, unless it has been killed since the shot (below) | the owner of the attacker |
   | The detonation of an object's death weapon (S-0312, claim 6) | no | no | the dying object | the dying object's house |
   | The detonation that follows the impact of an object that crashes while alive (S-0312, claim 16) | no | no | none | none |
   | Planted-bomb detonation (S-0519 claim 5) | no | no | the planter, if it still exists | none |
   | The suicide hit on the firer (S-0519 claim 11) | yes | no | none | none |
   | The direct hit on a crate collector from explosion or napalm (S-0520 claims 16 and 17) | yes | no | none | none |
   | A heal-all crate hit (S-0520 claim 14) | yes | yes | none | none |
   | The damage of a sonic wave (wave specs) | no | no | the wave's firing object | none |
   | Radiation exposure (S-0525 claims 9 and 10), at distance 0 leptons with the radiation-site warhead | no | yes | none | none |
   | Gas particle damage (S-0613 claim 13), with the particle type's damage, warhead and distance from that claim | no | no | none | the owner house of the particle's system (none for the shared gas system) |
   | A parasite's delivery (S-0614 claims 6 and 10) | no (yes for a drone's delivery to an infantry host) | yes | the parasite | none |
   | The detonation of the demolition warhead when a destroyed ore-spawning terrain object bursts (claim 20) | no | no | none | none |
   | A detonation set off by the lightning-storm bolt of the weather superweapon (superweapon specs) | no | no | none | the superweapon's owner |
   | The trigger action that destroys attached objects (S-0011, claim 27) | yes | no | none | none |
   | Expiry of a building's delayed-kill countdown (claim 16) | yes | no | a recorded demolition-charge planter, if present, otherwise none; not the attacker of the delayed-kill hit | none |
   | The destruction of the objects in contact with a destroyed building (S-0312, claim 17) | yes | yes | none | none |
   | The impact of an object that crashes while alive (S-0312, claim 16) | yes | yes | none | none |
   | The elimination of a house's remaining objects (S-0414 claim 5) | yes | yes | none | none |
   | Bridge destruction: ground-layer hits, landing hits after a deck drop, hits on units heading onto a lost deck, and low-strip kill checks (S-0511 claims 9–11 and 15) | yes | yes | none | none |
   | Crushing by trains (S-0210) | yes | S-0210 says | S-0210 says | S-0210 says |
   | Disc laser detonation (S-0615 claim 16) | no | no | the firer | none (`quirks.disc_laser_no_house`) |
   | Genetic mutation, explosion mode (S-0611 claim 3) | no | no | none | the launching house |
   | Genetic mutation, cell mode (S-0611 claim 4) | yes | no | none | the launching house |
   | Psychic domination area damage (S-0612 claim 9) | no | no | none | the current storm owner, or none (below) |
   | The area detonation of a damaging animation, such as a burning one (S-0109 claim 17) | no | no | none | none |
   | A voxel-model debris piece's ground-impact hits and the area damage of its end (S-0317 claims 7 and 11) | no | no | none | none |

   The crate explosion and napalm rows describe their direct hits only; their additional area
   damage follows S-0303 with neither attacker nor attacking house (S-0520 claims 16 and 17).
   Heal-all offers hits to the collecting house's objects taking simulation turns, including
   off-map open-topped passengers, but not out-of-play closed-transport passengers (S-0520
   claim 14). Claim 11 still governs healing, with the exceptions of claims 7 and 8.

   Other sources are believed to deliver damage that ignores defences too (side effects of some
   superweapons, the surfacing of tunnelling units); their owning specs give their markers,
   attacker and house. A detonation and a wave never ignore defences and never carry the
   no-escape marker. A damaging animation's owner house is not passed to its damage: even an
   owned animation delivers its area detonation without an attacker or attacking house.
   S-0109 claim 17 owns the delivered amount, warhead and timing.

   Mutation's explosion mode delivers 10 000 health points with its explosion warhead; cell mode
   delivers the infantry type's whole strength, at distance 0 leptons, with its cell warhead
   (S-0611 claims 3 and 4). These are distinct deliveries, not two stages of one hit.

   Domination samples the current storm owner before that tick's storm update, including cleanup.
   This includes the latest requesting house while a storm is pending, but is none before any
   request and after cleanup, not the owner of the last completed storm. Under
   `quirks.dominator_damage_storm_house` (default on = original), this is the attacking house.
   Off, it is the dominating house (S-0612 claim 9). The disc laser's absent attacking house is
   governed by `quirks.disc_laser_no_house` (default on = original; off, the firer's house),
   owned by S-0615 claim 16. Its attacker still supplies ordinary kill credit (claim 14).

   *The attacker of a projectile's detonation.* The detonation carries the object that fired the
   projectile as its attacker, and that object's owner as its attacking house, unless at some time
   between the shot and the detonation the object was *killed* (Terms: at the hit that takes its
   last health, however long it then stays in play for a death sequence or a delayed kill), was
   removed from the game by any other means or, believed, started to cloak. In those cases the
   detonation carries neither attacker nor attacking house. An object that is out of play but not
   killed, for example one loaded into a closed transport since the shot, is not known to stop
   being carried (low: loading a passenger is not known to end this). So a firer that is a vehicle that neither sinks nor crashes and is killed while the
   projectile flies (S-0203, claim 6a: it is gone at once) leaves its projectile ownerless, as does an infantry unit playing
   its death sequence or a building in the countdown of a delayed kill; a shot fired by a unit that
   is later loaded into a closed transport still carries the unit. A death weapon is set off after
   the killing hit that killed the dying object, so its detonation carries the dying object, killed
   as it is, and its house. A new shot fired by a building during the countdown of a delayed kill
   carries it normally.

   - Backing: `measurable` (detonations, every scenario below; what a detonation carries, scenarios
     D13, D17 and D24 including the firer loaded into a transport, the infantry firer with a death
     sequence, the death weapon and, in D24 part b5, the cloaking firer; the wave's attacker through
     D24 part f; that the ore burst of claim 20 does not ignore defences through D28 part d's
     comparison of two versus values, and its absent attacker and attacking house and unset
     no-escape marker through D28 part f; and that a wave hit does not ignore defences through D9 part b; the storm bolt's
     attacking house through D24 part d, and its two markers through D24 parts d2 and d3; the
     trigger kill through S-0011's scenario T9 (claim 9) and D24 part c; countdown expiry through
     claim 16 and D13 parts e to h; the contact destruction
     and the crash impact through S-0312's scenarios D11 and D12; house elimination through
     S-0414 claim 5 and its scenario V11; parasite deliveries through S-0614 claims 6 and 10,
     scenarios P1 and P6, and D31 below for the markers, attacker and absent attacking house; also
     bridge-destruction hits through S-0511 claims 9–11 and 15 and scenarios B3, B5 and B6, and
     radiation exposure through S-0525 claims 9 and 10, scenarios RF1 and RF7,
     gas-particle hits through S-0613 claim 13 and scenarios G4 and G10,
     planted bombs and suicide hits through S-0519 scenarios T6, T7 and T12, and
     crate hits through S-0520 scenarios CR10 and CR12, with heal-all exceptions in D32,
     mutation through S-0611 claims 3 and 4, scenarios MU1, MU2, MU4 and MU11,
     domination through S-0612 claim 9, scenario PD5 part b,
     disc-laser damage and attacker through S-0615 claim 16, scenarios DL1 and DL4, and
     damaging-animation metadata through D33; a voxel-model debris piece's markers and house
     as S-0317 claims 7 and 11 back them, through S-0317 scenario V5 parts c and d for the
     markers and parts f and g for the house, the house part holding once the check of the
     S-0414 claim 21 kill-tally amendment (S-0317 V5 part f) has run, as S-0317 claim 1 notes,
     and S-0317 scenario V8's no-source-object and own-house parts for the area damage of the
     piece's end),
     `unmeasured` (a voxel-model debris piece's absent attacker, which S-0317 claims 7 and 11
     leave `unmeasured`; the disc laser's absent explicit attacking house: ordinary kill credit falls
     back to its attacker and no discriminating house-only effect is identified (S-0615 claim 16),
     and mutation's unset no-escape marker in cell mode: only infantry receive it, so vehicle crew
     escape cannot distinguish the marker; a firer removed
     from the game in flight by some means other than a killing hit or a cloak: no such means is
     identified, so no scenario can set it up; the wave's
     attacking house column, and the absent attacking house of a planted-bomb detonation:
     ordinary kill credit alone cannot distinguish an absent house from the attacker's house.
     S-0519 claim 5 retains this limitation. A wave kill credits the attacker whether the wave carries its
     firer's house or none (claim 14), so a wave that carries its firer's house is not told apart
     from one that carries none; the wave's no-escape marker: D24 part f observes credit but
     does not provide a crewed target and an escape-chance control, so it cannot distinguish
     the marker values; the other believed scripted sources mentioned outside the table:
     no precise source setup and expected metadata are established here)
   - Confidence: high (detonations and waves never ignore defences and never carry no-escape,
     mutation, domination, disc-laser and damaging-animation metadata; a
     detonation of a firer that has not been killed, removed or started to cloak carries it and its
     owner; radiation-exposure and gas-particle metadata; crate-hit and suicide-hit metadata),
     medium (a voxel-model debris piece's hits, as S-0317 claims 7 and 11 rate them; planted-bomb
     metadata; a
     detonation of a firer that has been killed carries neither, and the same for a firer that
     stays in play after its killing hit; the death weapon's attacker; the markers and attacker of
     the trigger kill, the contact destruction, the crash impact and the wave; the ore burst's
     no-escape marker, like other detonations; the elimination row, owned by S-0414 claim 5,
     the parasite-delivery row and the bridge-destruction row),
     low (the storm bolt,
     which the superweapon specs own; the ore burst's attacker and house, which follow from
     claim 20 having no source object and house; that the detonations of a
     firer that starts to cloak carry neither attacker nor house; that those of a firer loaded into
     a closed transport still carry it; the other believed scripted sources without established
     setups)

### Firing side

2. **Shot damage.** When an object fires a weapon, the projectile, or the disc laser created
   instead of a projectile (S-0615 claim 12), carries the shot damage *D*ₛ of Formulas F1.
   *D*ₛ is fixed at the moment of firing from the weapon's damage field *D*: a change of
   the firer's firepower multiplier or veterancy after the shot does not change it. A disc laser
   delivers its held *D*ₛ at its final step (S-0615 claim 16), without projectile flight loss.
   For a projectile, the projectile's type may lower it in flight (S-0306, claim 18); the detonation delivers the damage
   the projectile has at that moment to each object it reaches (S-0303), except that prism support
   scales it after flight loss and before area damage: without integer wrap, the delivered amount
   is ⌊*D*ₜ × *M* / 256⌋, with integer *D*ₜ the damage remaining at detonation and integer *M*
   the support multiplier. Rounding is toward −∞; S-0518 claim 9 and F1 own the multiplier and
   signed 32-bit product-wrap rule. An electric-charge effect replaces ordinary area damage,
   so delivers no ordinary damage hit (S-0518 claim 11), rather than delivering a zero-amount hit.
   A weapon that delivers its damage through a sonic wave or through fire particles carries
   0 (the wave or particles deliver it, as their specs say). For any other weapon, *D*ₛ is *D*
   scaled by the factors below. Each factor, or product of factors, has its own truncation toward
   zero, so a later factor scales the already-truncated result of an earlier one:
   - the product of the house firepower multiplier and the object firepower multiplier, and then,
     when the firer has the veteran firepower bonus (S-0311 owns which veterancy levels grant it),
     the veteran combat factor. These apply only when *D* > 0; a non-positive *D* is not changed by
     them.
   - whatever the sign of *D*: the garrison multiplier when the firer is a garrisoned building, the
     bunker multiplier when it is a bunkered unit, and the open-topped multiplier when it fires from
     inside an open-topped transport.

   If a firer were in more than one of the garrisoned, bunkered and open-topped situations, F1 gives
   the result: the garrison factor's truncation inside the bunker factor's, and both inside the
   open-topped factor's.
   - Backing: `measurable` (scenario D1 for the plain case; D2 and D19 for the firepower
     multipliers and the order of truncation; D20 for each of the garrison, bunker and open-topped
     multipliers alone and for the sign rule; D15 part c for a firepower crate picked up while the
     projectile flies; D29 parts a and b for the veteran combat factor, and for the rule that the
     firepower multipliers and the veteran combat factor leave a non-positive *D* unchanged,
     and D15 part d for a promotion during flight; S-0518 scenarios PS1 for support scaling
     and TC1 and TC4 for electric-charge area-damage bypass; S-0615 scenarios DL1, DL4 and
     DL6 for disc-laser shot damage and the firing-side factors); `unmeasured` (the 0 carried by sonic-wave and
     fire-particle weapons, which is an amount the projectile carries; what such weapons observably
     deal is the wave's and the particles' own damage, which specs not yet written own, so no
     scenario of this spec can tell it; the combined result when a firer is in two or three of the
     three situations, which D20 cannot set up, since no known firer is in any two of them at once)
   - Confidence: high (the plain case, the firepower multipliers, each of the three multipliers
     alone, the sign rule, support scaling, electric-charge area-damage bypass and disc-laser
     shot damage),
     medium (that a crate picked up in flight changes nothing; the 0 carried
     by those weapons); low (that a promotion in flight changes nothing; the combined result of
     two or three of the multipliers, which no scenario reaches)

3. **House firepower multiplier.** Each house has a firepower multiplier, fixed when the houses are
   set up. In a skirmish the original makes it, for each player house, the product, rounded as the house multiplier formula after F1 says (*F*ₕ), of the firepower
   factor of the house's *difficulty row* and the firepower factor of its country (both
   imported; 1.0 when absent). **The two non-player houses are the exception**, in the original
   and in our engine alike (lead decision, 2026-10-10, S-0412 claim 5, S-0413 claim 8): the neutral
   house and the special house that every skirmish creates get the firepower multiplier exactly
   1.0, with no difficulty row and no country firepower factor, whatever those imported values are.
   Their rate-of-fire factor is also exactly 1.0, with no row or country general factor
   (S-0301 claim 3 owns that factor; S-0412 claim 5 owns this setup exception). In a campaign mission the country factor is not used; campaigns are out of
   scope for now, and which row a campaign house takes is left to the future campaign spec (lead
   decision, 2026-10-10). The row's armour factor and the country's general armour factor are never used by any
   damage rule: no hit changes with them (the factor that divides a hit is the country's factor for
   the target's kind, claim 5, with the object's multiplier). S-0006 (claim 5) lists the row's
   armour factor among the values with no identified effect and measures the row's firepower factor
   in its scenario D1.
   - *Our engine: every player house, human or AI, uses the normal row (a rule of our engine by decision
     of the project lead; no switch).* The normal row is the row the original gives a
     human-controlled house. By the same decision (the glossary's *AI house* and *difficulty skill
     block*), AI difficulty comes from the AI's strategy modules and never from stat factors. A
     house's row can differ only through an optional per-player handicap setting, which is off by
     default and is never tied to an AI house's difficulty automatically. A difficulty row that a
     map sets for a house is ignored in a skirmish (lead decision, 2026-10-10): the only thing that
     can change a house's row is the player's handicap setting, and the map never overrides or
     replaces it.
   - *The original's rows for computer houses (a departure, no switch).* The original gives the
     house of each computer player (a computer slot of the lobby) the row that its lobby entry
     selects, as the table says. S-0412 (claim 4) owns this pairing. The row depends only on the
     entry, not on the order in which the setup screen lists the entries. The shift moves a
     computer player whose row is not the easy row one row towards easy (difficult to normal,
     normal to easy), and applies when more than one human participant is in the game and the
     rules' computer-easy-bonus flag is on (S-0412 claim 4). Human participants include observers.
     A skirmish has at most one, so this original-only shift never occurs in a skirmish.

     | Lobby entry | Index | Row without the shift | Row with the shift |
     |---|---|---|---|
     | easy computer | 2 | difficult | normal |
     | normal computer | 1 | normal | easy |
     | hard computer | 0 | easy | easy |

     So the original's easy computer gets the row named difficult, and its hard computer the row
     named easy; which row's factors favour which house is rules data. Our engine does not
     reproduce this assignment. The same row also scales other house factors outside this spec.
   - Backing: `measurable` (scenario D2 for the country factor and the rounding of the
     product; D19, parts a and b, for the normal row's firepower factor in the product and for the absence of
     any effect of the row's armour factor and the country's general armour factor; part c for the
     rows of computer houses; S-0412 scenario K5 part d for the factor of exactly 1.0 of the neutral
     and special houses); `unmeasured` (the one-row shift, including its observer-inclusive
     count: it needs more than one human participant, which a skirmish never has; network
     games are out of scope, and it does not affect our engine. The normal-row default,
     optional handicap and ignored map row are design rules, not observations of the original)
   - Confidence: high (the product for a human-controlled house, the unused armour factors, the
     exact 1.0 of the two non-player houses, the row of a computer house in a skirmish with one
     person, which the lobby entry selects); low (the shift and its conditions, which are
     not measured)

4. **Object firepower and armour multipliers.** Each object has a firepower multiplier and an
   armour multiplier, doubles that start at 1.0. A crate changes one of them by its imported factor
   only while it is exactly 1.0, so crate bonuses never stack. Nothing else in a skirmish changes
   them. S-0520 claims 10 and 19 own crate conversion, the affected area and recipients,
   including off-map recipients, and the double-precision multiplication. This claim owns the
   starting value and the no-stacking rule: a multiplier not exactly 1.0 is unchanged.
   - Backing: `measurable` (scenario D1 for the starting value, since no multiplier shows in its
     losses; scenario D15 for the no-stacking rule); `unmeasured` (the exclusion of every
     non-crate source of multiplier changes: D15 tests crates only, and no exhaustive scenario
     covering all special abilities is available)
   - Confidence: high (the starting value); medium (no stacking); low (the exclusion of other sources)

### Target side

*Scope.* Claims 5 to 19 are stated for infantry units, vehicles, aircraft and buildings, the
objects that have an owner, defences and a house armour factor. Terrain objects, the only other
kind of object whose health a hit changes, follow claim 20. Walls take the wall hits of S-0303
(claim 16). S-0303 delivers hits to the objects of a scanned cell, and the kinds a cell holds are
those named here and in claim 20; it is believed that no overlay, smudge, animation, particle
system or projectile is ever such an object (S-0303 states which kinds receive hits).

5. **Defences.** A hit that does not ignore defences, and whose delivered amount is not negative,
   is reduced by the target's defences to the defended amount *d*₃ of Formulas F2. The later
   claims (6, 8 and 10) start from the defended amount. It is the delivered amount divided by the
   target's armour, truncated toward zero, and never less than 1. The armour has two parts, each a
   divisor with its own truncation:
   - the product of the house armour factor (the country factor of the target owner's country for
     the target's kind: infantry, vehicles, aircraft, defence buildings or other buildings) and the
     target's object armour multiplier.
   - the veteran armour factor, when the target has the veteran armour bonus (S-0311).

   For a prone infantry target hit by a positive amount, the amount that is divided by the armour is
   the delivered amount times the warhead's prone factor, truncated toward zero and raised to at
   least 1. So the defences can never reduce a non-negative hit below 1. A negative amount (healing)
   is not changed by the defences.

   **Defence category import (Q-0061 part 2).** Only the combat category selects the defence-building
   armour factor. The category is read from the first 31 bytes of the value, with surrounding
   whitespace trimmed after that cut, and compared as a whole with the accepted category words,
   ignoring the letter case of A to Z (Import mapping). A nonempty unknown word replaces the held
   category with the unspecified category, which is not combat: the ordinary-building factor then
   applies. A missing or empty key keeps the held category, initially unspecified. Thus an unknown
   word in a later layer does not preserve an earlier combat category.
   - Backing: `measurable` (scenario D2 for the armour factor and the minimum; D21 for the kinds
     of target, among them the defence-building factor against the other-buildings factor, and
     part b for category comparison, length limit, unknown words and absent keys; D7 for
     the prone factor; D25, part c, for a healing hit against an armour factor other than 1.0; D29
     part c for the veteran armour factor and its own truncation)
   - Confidence: high

6. **Versus value, falloff and cap.** For a hit that does not ignore defences, the defended
   amount becomes the modified amount *m*₀ of Formulas F3:
   - in a game whose map marks it as *inert*, *m*₀ is 0 (detonations in such a game deliver no
     hits at all, S-0303 claim 2, so this reaches only hits from other sources, such as sonic
     waves), and *m*₀ is also 0 for a hit that comes without a warhead.
   - otherwise, for a negative amount, *m*₀ is the amount unchanged when the distance *r* is less
     than 8 leptons, and 0 otherwise. The versus value, the falloff and the cap don't apply to it.
   - otherwise, for a positive amount, *m*₀ is the amount after the spread falloff, floored at 0,
     times the versus value and truncated toward zero, and at most the game-wide maximum damage.
     S-0303 owns the spread and the distance; the falloff is stated in F3 because its rounding
     interacts with the versus value.

   There is no minimum after the versus value: a hit of 1 against a versus value from 0 % up to
   but not including 100 % becomes 0. The game-wide minimum-damage field has no effect anywhere,
   whatever its value. A negative versus value gives a result of 0 or below; a result below 0
   makes the hit heal (claim 11). A non-negative maximum does not limit a negative result.
   A negative maximum still applies as F3's minimum of the result and the maximum for a positive
   delivered amount made negative by its versus value (D3 part c). It does not apply to a negative
   delivered amount: that amount follows the negative path above, unchanged within healing reach,
   with no cap.
   - Backing: `measurable` (scenario D1 for versus and truncation, D3 for the cap, the unused
     minimum and a maximum of 0 or below, D4 for the negative path and S-0314 H1 part c for its
     bypass of a negative maximum, D9 for the inert map, D25 for a positive hit against a negative
     versus value; the falloff at distances above 0 through S-0303's scenario A1, which measures
     the same formula, including variant g for the edge product's use of the integer defended
     amount rather than its single-precision narrowing); D34 part a for a hit without a warhead
     on infantry objects that are not prone (no health lost, which cannot tell such a hit from no
     hit at all), and D34 part b for the same hit on an undemolishable building, where claim 7's
     loss of exactly 1 shows that the piece deals the hit.
     `unmeasured` for a scripted source's hit without a warhead: no scripted source that a test
     map can set up is known to deliver one
   - Confidence: high for every part except a hit without a warhead; medium for the result of a
     hit without a warhead from a debris piece, until D34 runs; medium for a scripted source's hit
     without a warhead, which follows the same rule but has no scenario

7. **Undemolishable buildings always take at least 1.** A building whose type is undemolishable
   loses at least 1 health from every hit that no row of claim 8 stops, whether or not the hit
   ignores defences. Claim 8's zero-amount row looks at the amount after this minimum, so it never
   stops such a hit. A result of 0 becomes 1, and so does a negative result, so such a building cannot be healed by a hit and loses 1 health instead. This
   holds in an inert game too, for the hits that reach the building there (sonic waves, for
   example; detonations reach nothing, S-0303 claim 2). Buildings that can be demolished and all
   other objects get no such minimum. A hit without a warhead that does not ignore defences
   therefore takes 1 from such a building, although its *m*₀ is 0 (claim 6).
   Quirk `quirks.undemolishable_heal_is_damage` (Edge cases).
   A heal-all crate supplies a known hit that ignores defences with a negative or zero amount
   (S-0520 claim 14). It therefore damages an undemolishable building by 1 health rather than
   restoring it, even when the building was already at full health.
   - Backing: `measurable` (scenario D5; D9 for the inert game; D32 for a heal-all hit that
     ignores defences with a negative or zero amount; D34 part b for a hit without a warhead)
   - Confidence: high (medium for the inert game, the heal-all exception and a hit without a
     warhead)

8. **Hits that do nothing.** A hit changes no health when one of the rows of the table below holds
   for it. It then reports *unaffected*, except under the berserk row (claim 10).
   - *Columns.* Whether a row applies to a hit depends on the sign of the **delivered** amount *d*₀
     (after the reset of claim 17), never on the final amount: **+** is a hit with *d*₀ ≥ 0 that
     does not ignore defences (a delivered 0 counts as +), **−** is a hit with *d*₀ < 0 that does
     not ignore defences, and **I** is a hit that ignores defences, whatever its sign. A hit with
     *d*₀ > 0 against a negative versus value is therefore a + hit for every row, although it ends
     as healing (claim 11): on a temporarily invulnerable target, for example, it does nothing.
   - *Stages.* A row's stage says which of the two side effects of claim 10 a hit that the row
     stops still has. A hit that no stage 1 row stops causes readiness loss (claim 10), whatever else
     holds for it. The side effects of a stopped hit are:

     | Rows that hold for the hit | Readiness loss (claim 10) | Berserk effect (claim 10) |
     |---|---|---|
     | A stage 1 row | none | none |
     | A stage 2 row, and no stage 1 row | yes | none |
     | No stage 1 or stage 2 row, and the warhead makes targets go berserk (the stage 3 row) | yes | the berserk row's effect (claim 10, with its exceptions), and no stage 4 row applies to the hit |
     | A stage 4 row, and no row of stages 1 to 3 | yes | does not arise (the warhead does not make targets go berserk) |

     "Yes" under readiness loss means: for a target whose type has the damage-reduces-readiness
     flag. The order of the rows within one stage is not observable, since each stops the hit with
     the same result. The stage of the two no-health rows (the building's in stage 1, the other
     kinds' in stage 4) is different: it is observable, through the readiness loss and the berserk
     effect, for a hit that reaches an object with no health, and an engine implements the table as
     written for such a hit. No known source delivers one (*No-health targets* below), so no
     scenario sets it up, and the stages of those two rows are low confidence.
   - *No-health targets.* An object with no health can stay in play after its killing hit (an
     infantry unit playing its death sequence, a vehicle or aircraft that is crashing, a building
     that is exploding or being sold, claim 1 and S-0312), so the two rows are not ruled out by
     "out of play". They are ruled out, for the known sources, by those sources' own filters.
     S-0303 (claim 9) leaves out of every detonation, so also out of death weapons and storm bolts,
     a candidate that is not alive or has health 0 or less; the trigger kill acts on present objects
     only, which exclude destroyed ones (S-0011, Terms and claim 27). An object that stays in play
     with health left (a sinking vehicle, a building in a delayed-kill countdown or a crippled
     cyborg) is not a no-health target at all, and takes hits by the other rows. Sinking supplies
     no damage or healing immunity: while health is positive, an eligible hit follows the ordinary
     rules, including another killing result if it takes the remaining health. This grants no new
     targeting permission and does not guarantee that an already-fired projectile hits (S-0504
     claims 5, 10 and 13). Not every possible sinking state is asserted to retain positive health:
     sinking can also begin through a cull (S-0614 claim 11). If a sinking target has no health,
     the non-building no-health row still stops ordinary damage and healing hits; it is not revived. The sources that this spec cannot place under a filter are the sonic wave, the
     contact destruction, the elimination of a house's objects, the destruction of a bridge section,
     the train crush (claim 1) and the ground-impact hits of a voxel-model debris piece (S-0317
     claim 7, which leaves this open for them): whether they skip an object with no health is for their specs
     (Open questions).
   - *Amounts.* Only three things in the table look at an amount: the effect that the
     temporarily-invulnerable row shows is sized by twice the amount after the defences of claim 5,
     the berserk row's duration is taken from an amount as claim 10 says, and the last row looks at
     the amount after claims 5 to 7. Whether any other row applies does not depend on an amount.

   | Stage | Rule | + | − | I |
   |---|---|---|---|---|
   | 1 | The target is a building and the attacker is that same building, and its type does not allow self-damage | yes | yes | yes |
   | 1 | The target is a building whose type is a laser-fence section | yes | yes | no |
   | 1 | The target is a bridge repair hut whose type is marked immune (to all damage) | yes | yes | yes |
   | 1 | The target is a vehicle standing inside the vehicle factory it is in contact with (it has not left the factory) | yes | yes | no |
   | 1 | The target's type is marked immune to its own type, and the attacker is of the same type and has the same owner | yes | no | no |
   | 1 | The target is temporarily invulnerable (an effect is shown, as for a hit of twice the amount) | yes | no | no |
   | 1 | The target is being erased | yes | yes | no |
   | 1 | The target is a building that already has no health | yes | yes | yes |
   | 2 | The target is a tank bunker holding a bunkered unit, and the warhead penetrates bunkers | yes | yes | no |
   | 2 | The target is a bunkered unit inside its bunker, and the warhead does not penetrate bunkers | yes | yes | no |
   | 2 | The warhead is a radiation warhead and the target's type is immune to radiation | yes | yes | yes |
   | 2 | The warhead does psychic damage and the target's type is immune to psychic weapons | yes | yes | yes |
   | 2 | The warhead is a poison warhead and the target's type is immune to poison | yes | yes | yes |
   | 2 | The warhead does not affect allies, an attacker is given, and the attacker's owner is allied with the target's owner (a house is allied with itself; a hit that has no attacker is never stopped by this row) | yes | yes | yes |
   | 3 | The warhead makes its target go berserk (claim 10 gives what happens instead of damage) | yes | yes | yes |
   | 4 | The target is not a building and already has no health | yes | yes | yes |
   | 4 | The target's type is marked immune (to all damage) | yes | yes | no |
   | 4 | The amount is 0 after claims 5 to 7 | yes | yes | yes |

   Being erased includes both an attached temporal beam and the beam-free leaving stage
   (S-0514 claims 4 and 10; S-0513 claim 39). It does not include a superweapon warp member from
   its departure update when its beam was locked before the departure: that member takes
   ordinary hits, while the beam stays attached (S-0513 claim 39a), up to a later lock after the
   departure. From such a lock on, the member is being erased again and this claim's protection
   applies to it (S-0513 claim 39a, *A lock after the departure*). The being-erased stage 1 row
   stops ordinary damage, healing and berserk hits before either side effect of claim 10.
   It does not stop a hit that ignores defences.
   That hit is still subject to every other applicable row. Temporal erasure itself is not a
   health-damage hit: its delivery, lock refusals and progress belong to S-0514 claims 1 to 8.

   The last row is what makes a positive hit do nothing when the versus value, the falloff, an
   inert map or a missing warhead took it to 0 (unless claim 7 raised it to 1). A delivered amount
   of 0 is different: when the hit ignores defences it does nothing (except on an undemolishable
   building, which loses 1, claim 7); otherwise its defended amount is 1 (the minimum of claim 5),
   and claim 6 applies to that 1 (a direct hit against a versus value of 0 or more deals trunc(*v*), at most the cap of claim 6).
   Detonations never deliver 0 (S-0303 claim 2: a detonation of 0 reaches nobody); claim 17 is
   one such source; radiation exposure can also deliver 0 while the exposure level is positive
   (S-0525 claims 9 and 10), and so can a voxel-model debris piece, whose ground impact deals its
   hits whatever its damage amount (S-0317 claim 7, scenario V5 part e).
   - *Hits without a warhead.* No row that reads the warhead holds for a hit without one: neither
     bunker row (such a hit counts as neither penetrating bunkers nor not penetrating them), none
     of the three immunity rows of stage 2, not the allies row and not the berserk row. Such a hit
     that no stage 1 row stops therefore causes readiness loss and has no berserk effect. If it
     does not ignore defences, its *m*₀ is 0 (claim 6) and the zero-amount row stops it, unless
     claim 7 raises it to 1 on an undemolishable building. A hit without a warhead that ignores
     defences keeps its delivered amount (claim 9). On a terrain object such a hit is stopped
     (claim 20).
   - Backing: `measurable` (scenario D6 for invulnerability, D8 for the allies rule (with an
     attacker; D17, part g, for a hit with none), the immune
     and type-immune rows and the bunker rows, D16 for the building, vehicle-factory, bridge-repair-hut,
     laser-fence and being-erased rows (the trigger kill on an immune bridge repair hut in its own
     last run, d2, which expects the hang), D17 for the radiation, psychic and poison rows and for
     the stages; D26 for the − column of the rows that D6 and D8 do not test; S-0504 scenarios
     N10 and N14 for eligible positive-health hits during sinking; S-0525 RF1 variant c for a
     zero-amount radiation hit); `unmeasured` (the two
     no-health rows and their stages: every source that a scenario of this spec can set up skips an
     object with no health before the hit (*No-health targets* above), and the sources that might
     not skip it belong to specs not yet written, so no scenario can deliver a hit to such an
     object; the − entry of the self-damage row: no known setup makes a building deliver a healing
     hit to itself; other table entries not exercised by D6, D8, D16, D17, D26 or S-0011 T9:
     those scenarios do not cover every combination of source markers and target protections,
     in particular an ignoring-defences hit with an allied attacker; the invulnerability
     effect's size of twice the defended amount: D6 checks health only and supplies no
     size-sensitive visual comparison; the direction of the one-way alliance test:
     D8 and D17 use mutually allied houses only). For the *Hits without a warhead* bullet:
     `measurable` through D34 parts a and b for its health result, part c for its readiness
     loss, part d for the three immunity rows not holding, part e for the tank-bunker row not
     holding, and part a's berserk record for the berserk row not holding (the row needs a
     warhead that makes targets go berserk, which a hit without a warhead lacks); `unmeasured`
     for the bunkered-unit and allies rows not holding. The only warhead-less source a scenario
     can set up, a debris piece of a type with no warhead, has no attacker (S-0317 claim 7), so
     the allies row cannot arise; and its hit on a bunkered unit, which is not a building and so
     takes no minimum from claim 7, is stopped by the zero-amount row whether or not the
     bunkered-unit row stops it first, so health does not tell the two apart. For those two rows
     the bullet states the rule that no row reading the warhead holds without one, as the
     measured rows show for the others, with no scenario of its own; the prone factor of claim 5
     for such a hit is the one departure (Edge cases). For the warp member that is not being
     erased: `measurable` (S-0513 SW12 part l, variant l1) for hits after the hold, and variant l5
     for the protection from a later lock; `unmeasured` while it phases in, since the hold in
     part l lasts one update
   - Confidence: high for the + column except the no-health rows and for the − and I entries that D6, D8, D16,
     D17 and D26 test; medium for the stages of the rows that only D16 and D17 test, and for the
     vehicle-factory, laser-fence and bridge-repair-hut rows (the conditions are clear; D16 is
     their first measurement); low for the no-health rows (their stages included) and the − entry of
     the self-damage row and other entries without a scenario; medium for a hit without a
     warhead, until D34 parts a to e run, and medium for the rows that no scenario can tell
     apart (`unmeasured`, above); medium for the warp member that is not being erased: hits
     after the hold, hits while it phases in, and the protection from a later lock

9. **What ignoring defences skips.** A hit that ignores defences skips the defences of claim 5
   (including the minimum of 1 and the prone factor), every part of claim 6 (inert map, falloff,
   versus value, cap, the negative-path distance limit) and the rows of claim 8 marked "no" in
   column I. The delivered amount therefore lands as it is, apart from the minimum of claim 7, the
   remaining rows of claim 8 and the limits of claim 11. This agrees with S-0011 (claim 27) and
   adds the building minimum of claim 7, which S-0011 does not mention.

   *A hit that ignores defences and is stopped.* A row marked "yes" in column I stops such a hit
   without any change of health. The trigger kill of S-0011 (claim 27) has no attacker, so of those
   rows only these can stop it: the immune bridge repair hut, a warhead-kind row (radiation,
   psychic, poison), the berserk row and, after the reset of claim 17, the zero-amount row. A target that survives the trigger kill hangs the original
   (S-0011, claim 27). The warhead-kind rows and the berserk row stop a trigger kill only when the
   demolition warhead itself is of that kind, which S-0011 already names (its protections by
   warhead kind, berserk included), so a demolition warhead that is also flagged as a berserk
   warhead is no further case. The same hang follows from the immune bridge repair hut, which is a
   property of the type in the map's data and not of the warhead. The hang also follows from a
   building that the demolition warhead puts into a delayed kill (claim 16), if that warhead
   causes delayed kills, and from an infantry unit at positive height above the surface hit by
   a demolition warhead of infantry-death kind 9 (claim 17). S-0011's departure for the survivor (damage it once per
   firing and go on) and its importer warning cover all these survivor cases. At hand-off,
   confirm this survivor coverage together with S-0011 claim 27, including the distinction
   between general immunity and an immune bridge repair hut and the positive-height-above-the-surface
   test of claim 17, which S-0011 claim 27 also states; it is not S-0308's airborne-state test. The zero-amount
   row stops a trigger kill only in the positive-height case just named: every other
   target has health at the trigger kill, so the amount is above 0. The self-damage
   row cannot stop a trigger kill (it needs an attacker), and the two no-health rows do not apply to
   it (S-0011 acts on present objects only, *No-health targets* in claim 8).
   - Backing: `measurable` (through S-0011's scenario T9 for its claim 27, and S-0210's scenario K6
     for trains; the immune bridge repair hut through scenario D16, part d2, which expects the
     hang; the delayed-kill and positive-height infantry survivors through D16 parts d3 and d4)
   - Confidence: high (what the marker skips); medium (the hang for the immune hut, the delayed
     kill and the positive-height infantry case: the hang has not been observed, S-0011)

10. **Side effects that happen even when the hit does nothing.** Two effects belong to the stages of
    claim 8's table. Neither depends on the final amount, and both happen for hits that end up
    doing nothing:
    - **Readiness loss** (unless a stage 1 row stops the hit). A target whose type has the damage-reduces-readiness
      flag loses ammunition in proportion to the amount after the defences of claim 5 (before the
      versus value), as Formulas F6 says, and its reload countdown restarts (S-0301, claims 10, 11 and 17 own
      ammunition, reloading and that restart). It happens for every hit
      that no stage 1 row stops, whether the hit ignores defences or not, and also for a negative
      hit, and it still happens when a stage 2, 3 or 4 row then stops the hit. A hit that a stage 1
      row stops causes no readiness loss.
      Thus an ordinary hit on a target being erased causes neither readiness loss nor berserk.
      A hit that ignores defences bypasses that particular protection (claim 8; S-0514 claim 17).
      Quirks `quirks.readiness_from_defended_damage` and, for a negative hit that raises
      ammunition above the type's field, `quirks.readiness_heal_over_max` (Edge cases).
    - **Berserk instead of damage** (unless a stage 1 or stage 2 row stops the hit). A hit whose
      warhead makes targets go berserk changes no health. When no stage 1 or stage 2 row stops it,
      it has the following effect, whatever its column. Unless an attacking house is given and the
      target's owner is allied with it, the target's type is immune to psionics or the target is a
      building, the target receives a remaining berserk time of *T* countdown updates (S-0610 claim
      5), as the berserk duration of Formulas gives: the modified amount of claim 6 (F3) at distance
      0, taken of the hit's amount after the defences of claim 5, or of its delivered amount when
      the hit ignores defences. Ignoring defences changes only which amount enters: every case of
      claim 6 (inert game, healing reach, falloff, versus value, cap) applies to *T* in both cases.
      So *T* is 0 in an inert game and for an amount of 0, and a negative amount stays negative.
      S-0610 claims 4 and 5 own entry and expiry: a new berserk state drops the target and queues
      *hunt*, even for *T* ≤ 0, which expires at the next countdown update. For *T* ≥ 1, expiry is
      on the *T*-th countdown update following the hit; the first may be in the hit tick, so *T* is
      not necessarily the number of elapsed ticks. S-0610 claim 5 owns expiry's target loss and
      queued behaviour. Such a hit reports *unchanged*. It sets off none of the reactions that the
      hit's own damage step makes: no anger change (S-0309 claim 16), no retaliation (S-0215 claim
      18), no scatter from damage (S-0215 claim 19, and request A of S-0208 claim 15 for infantry),
      no trigger event of the target's tag, no damage effect or marking of one (claim 13) and no
      loss of a disguise (D23, part f). It makes no sound reaction of the hit either: although it
      reports *unchanged*, the target type's damage sound, which S-0701 claim 20 gives for other
      hits that report *unchanged*, is not played. The reactions that a mobile target makes after
      that step to a surviving hit that reports *unchanged* do follow it, as their own claims give
      them: the no-threat fall-back (S-0215 claim 16), then for an infantry unit request B (S-0208
      claim 15) and the change of fear (S-0107 claim 16), and for a vehicle the crush response
      (S-0215 claim 39) and the announcement that a harvester is under attack (S-0701 claim 28).
      Each of them that needs a known attacker needs one here too. Of the announcements that S-0701
      claim 28 lists, that is the only one a berserk hit can make. Its base and ally announcements
      come only from hits on buildings, which a berserk hit leaves *unaffected*, so a berserk hit on
      any other unit, the local player's or an ally's, makes none of them and no radar event. A team
      member that the hit makes berserk has left its team by then (S-0215 claim 39), so the team
      exception of the no-threat fall-back does not hold it back. In the three excepted cases
      nothing happens and the hit reports *unaffected*. A further berserk hit on a target that is
      already berserk sets the remaining time to the new *T*, even when that is shorter. A berserk
      hit is not stopped by the stage 4 rows, so a target that is immune to all damage still goes
      berserk. A berserk hit that a stage 1 or stage 2 row stops, for example one of a radiation
      warhead on a target immune to radiation, causes no berserk state and reports *unaffected*.
    - **Who goes berserk from ordinary fire.** A detonation whose firer has not been killed,
      removed or (believed, low) started to cloak since the shot carries an attacking house
      (claim 1), so a berserk warhead never makes the firer's own side
      or its allies go berserk. That includes (low) a firer that is out of play but not killed, such
      as one loaded into a closed transport, and the detonation of a death weapon, which carries the
      dying object's house (claim 1). A detonation of a firer that has been killed since the
      shot (claim 1) carries no attacking house: then an allied target goes berserk too (and the
      allies row of claim 8 does not apply to it either, since that row needs an attacker).
    - Backing: `measurable` (scenario D10 for readiness loss, D23 part f for the kept disguise after
      a berserk hit, S-0701 scenario F1's berserk hit for the absent damage sound on a vehicle,
      S-0215 claim 39's backing (S-0210 scenario K15) for the crush response after it, D17 for
      berserk, its duration, its reset, its stages and who goes berserk, including parts g, g2, g3
      and g4, with entry and update-count expiry through S-0610 claims 4 and 5, scenario P2, and *T*
      ≤ 0 through D17 part i, supplementing S-0610 scenario P3's conditional stock setup).
      `unmeasured` for the absent damage sound on a target that is not a vehicle, and for the absent
      announcements and radar events of a berserk hit: no scenario here has an enemy house fire a
      chosen warhead once at a chosen unit while the announcements are recorded. `unmeasured` (*T*
      for a hit that ignores defences: a trigger kill with a berserk demolition warhead can deliver
      one, but the original then hangs on its survivor, preventing observation of when the berserk
      state ends; the absence of trigger reactions to a berserk hit: D23 f observes disguise only
      and no tag-event test is supplied here; the other reactions of the damage step that a berserk
      hit does not make, and the later reactions that it makes other than the crush response: no
      scenario here lands a berserk hit on a subject whose record shows anger, a retaliation, a
      scatter request, a fall-back, fear or the announcement; the direction of the one-way alliance
      exception: D17 uses mutual alliances; readiness loss at strength 0: D10 uses positive strength
      and no setup keeps a strength-0 target alive to receive a hit)
    - Confidence: medium (readiness loss, including that it uses the amount before the versus value
      and happens even when a later row stops the hit); medium (berserk duration, its reset, and the
      stages); medium (a detonation of a firer that has been killed carries no attacking house; one
      of a death weapon carries the dying object's house); low (that one of a firer in a closed
      transport carries its house; *T* for a hit that ignores defences). Entry and update-count
      expiry, including *T* ≤ 0: medium (S-0610 claims 4 and 5). The reactions that a berserk hit
      makes and does not make, the damage sound and the announcements included: medium (the
      announcements, and the damage sound on a target that is not a vehicle, `unmeasured`), and the
      crush response after it has S-0215 claim 39's confidence

11. **Health change.** A hit whose amount *m* after claims 5 to 8 is not 0 changes the target's
    health *H* (Formulas F4):
    - a negative amount raises health by |*m*|, but never above the strength; the result is
      *unaffected*, even when health rose,
    - a positive amount larger than the remaining health counts as the remaining health, and health
      falls by it; the result is given by claim 12.
    Health never goes below 0 or above the strength, subject to the kind-specific survival
    outcomes: claim 15's crippling, claim 16's delayed kill and S-0312 claim 10's sinking at
    1 health. A positive-health vessel already sinking is not exempt from this arithmetic.
    Another lethal hit follows the ordinary death rules; when it again meets the sinking
    condition it returns to 1 health without restarting descent (S-0504 claim 13). A target
    with no health remains excluded by claim 8, including from healing.
    - Backing: `measurable` (scenarios D1 and D4; S-0504 N10 and N14 for positive-health sinking),
      `unmeasured` for a sinking vessel remaining in play at zero health: no reproducible setup
      here establishes that state, as S-0504 claim 13 notes
    - Confidence: high

### Results and thresholds

12. **Damage-state result.** A hit that lowers health reports *unchanged*, unless it crosses a
    threshold:
    - *now yellow*: health was at least half the strength (rounded down) and is now below it, and
      the hit did not take all remaining health. The half is fixed: the game-wide yellow ratio
      (which decides the health bar's colour, S-0111) plays no part here,
    - *now red*: health was above the strength times the game-wide red ratio and is now below that
      product. *Now red* takes precedence over *now yellow*,
    - *now dead*: health is now 0, which takes precedence over both (claims 15 and 16 give the
      exceptions).
    **Imported threshold precision (Q-0075).** The red and yellow ratios are held as doubles,
    but their decimal input is first converted to a single, widened exactly, and, when a percent
    sign is present, scaled in double precision as S-0003 claims 6 and 7 say. They are not the
    double nearest to the written decimal. F5 uses that imported value without an additional
    narrowing. For a red ratio written as 0.6 without a percent sign and strength 1000, the hit
    from 601 to 600 reports *now red*, not the next hit from 600 to 599.

    The result decides the reactions to the hit (a berserk hit has none of those below, whatever it
    reports, claim 10). This claim owns two of them: the damage effect of claim 13, and, for a
    target whose type can disguise itself but does not keep its disguise permanently, the loss of
    its current disguise and a disguise-disruption countdown of 2*m*′ ticks, *m*′ the health it
    lost, after any hit that reports *unchanged*, *now yellow* or *now red* (S-0516 claims 33–34 own
    disguise loss and re-disguise eligibility). It also owns one reaction of a building: on *now
    yellow* the intensity of its light source is scaled by a fixed factor (D11, part c). Two more
    are pointers only, owned elsewhere and not claims of this spec: the trigger events that a hit
    sets off through the target's tag, whose list S-0011 leaves to a later revision, and the
    building's sound reactions, owned by S-0701 claims 20 and 37. Claim 37 specifies the any-owner
    building damage fallback on *now yellow* or *now red* and the death fallback when a lethal hit
    begins the destruction reaction, not at later removal. Quirk `quirks.yellow_state_at_half` (Edge
    cases).
    - *Post-mortem when a reaction removes the target.* When one of those reactions takes the
      target out of play (a trigger that the hit sets off can destroy it, S-0011), the hit reports
      *post-mortem* instead of the result above. Only the source of the hit receives this result,
      and no effect of it is known in a skirmish, so this part of the claim cannot be told from the
      other results. An engine need not represent this result (it may report the result above)
      until an effect of it is found; the claim records the original's value only.
    - Backing: `measurable` (scenario D11 for the thresholds, its parts a to c for the yellow
      threshold, the red threshold and the precedence of red, and for the light source; part d
      for the imported precision and the 601 to 600 crossing; D23 for
      the loss of the disguise and, in part h with S-0516 scenario CS14 part c, the length of
      the disguise-disruption countdown); `unmeasured` (the post-mortem result after a reaction removes the target: the result that a hit reports then
      has no further effect that this spec or its owners could observe, so it stays a documented
      behaviour in case an effect is found)
    - Confidence: high (the thresholds); medium (the light source, the loss of the disguise and the countdown); low (post-mortem after a reaction)

13. **Damage effect and its draw.** After a hit that reports *now yellow* or *now red*, an object
    whose health ratio (S-0214, Terms) is at or below the game-wide yellow ratio (Import mapping; unlike the fixed half of claim 12, this is the imported ratio) and that has no damage
    particle effect yet gets one, chosen from its type's list of damage particle systems by one
    ranged draw from the scenario random stream over 0 … *n* − 1, where *n* is the number of
    eligible entries, provided *n* > 0 and the object's height above the surface is greater than
    −10 leptons (the glossary's surface includes a bridge deck when the object is on it).
    The eligible entries are the particle system types of the list whose behaviour is *smoke* (Import mapping); a draw of *k* picks the (*k* + 1)-th eligible
    entry counted from the end of the list, so a draw of 0 picks the last one. After a hit that
    leaves the health ratio above the yellow ratio, an existing damage particle effect is marked (S-0616 claims 1 and 22): the object keeps it, and
    makes no new one, until that system leaves play.
    This includes healing hits and hits that the stage 4 rows of claim 8 stop (no health on a target
    that is not a building, immune to all damage, zero amount), but not hits that its stage 1 to 3
    rows stop, and not berserk hits. These are the hit-driven marks; S-0616 claim 1 owns the
    additional damage-smoke marks after an object's own self-healing or a non-organic vehicle's
    house-wide self-healing, including their health-ratio and below-surface tests.

    **Building fires and their draws.** After a hit that reports *now yellow* or *now red* on a
    building, when the hit's warhead is one that sets fires (the warhead's sparks flag, Import
    mapping), the hit takes one ranged draw from the scenario random stream for each cell of the
    building's foundation. The draws take the foundation's cells row by row: the rows by
    increasing distance along map *y* from the foundation's top-left cell, and the cells of a row
    by increasing distance along map *x*, the map axes of the glossary's *foundation*; the
    isometric screen plays no part. For every foundation word of S-0108's vocabulary (Import
    mapping, foundation vocabulary) the cells are the full *W* × *H* rectangle, *W* along map *x*
    and *H* along map *y*, except two: the 8-cell 3 × 3 foundation word (Import mapping), whose
    cells are the 3 × 3 rectangle without the cell 2 cells along map *x* and 1 cell along map
    *y* from the top-left cell (8 cells: its three rows have 3, 2
    and 3 cells), and the zero-size foundation, which has no cells and so no draws. S-0108 and S-0312
    may find the same cells and order useful for their own cell-by-cell rules; nothing here
    claims it of them. Each cell's draw is over 0 … *W* + *H* + 5, where *W* × *H* is the
    foundation's size in cells (S-0108, Import mapping, foundation vocabulary). A result from 1 to 5 starts the first of the rules' damage-fire
    animations on that cell, 6 to 8 the second and 9 the third; any other result (0, and every
    result above 9) starts none. A fire of the first or second kind also takes one ranged draw, over
    1 … 3, for its number of loops. On the stream, that loop draw comes **directly after its own
    cell's draw and before the next cell's draw**: the hit's draws are, in stream order, the cells'
    draws in the order above, each followed at once by its cell's loop draw where there is one
    (scenario D18 tells this order apart from the alternatives). The structures specs own the fires
    themselves (S-0109 gives what a burning animation deals).

    **No other draws.** The amount of a hit never depends on a draw. Apart from the two cases
    above, nothing in this spec draws from the scenario random stream. Building sound requests
    follow S-0701 claims 20 and 37, including the threshold fallback only when the type names
    no damage sound of its own. Sounds can take draws from the unsynchronised random stream,
    which never reach game state (S-0202, claim 11).
    Whether each of the two results takes them does not matter to any rule here. The death of an object takes draws of its own (S-0312).
    - Backing: `measurable` (scenario D11 for the damage effect; D22 for the draw, the eligibility
      by behaviour and the counting from the end of the list, by replay of the recorded
      stream; scenario D18 for the building fires and their draws, by replay of the recorded stream and by
      counts, with the zero-size foundation in its fourth run; D27 for the marking of an effect by a
      healing hit); `unmeasured` (the depth limit of
      10 leptons below the surface: which objects can be that deep while they take a hit that
      crosses a threshold is not known, so no scenario can set the case up; the marking of an
      effect by a hit that its stage 4 rows stop, and the absence of marking for hits that its
      stage 1 to 3 rows stop and for berserk hits: these differ from the plain case only for an
      object whose health ratio is above the yellow ratio while it still has an unmarked effect,
      which no hit not stopped by stage 1 to 3 rows leaves behind, since such a hit marks the
      effect itself, so only something that
      raises health without a hit, such as the repair of S-0314, could set it up, and what that
      does to the effect is not known; cell lists and draw order for foundation words other
      than the 2 × 2, 8-cell 3 × 3 and zero-size words: D18 supplies no run for them. Also unmeasured is
      the absence of all other draws: D18 and D22 record their named cases, not every hit
      source and target combination; the original's undefined outcome for a short damage-fire
      list: D18 supplies all three entries, and no missing-entry variant is proposed. The
      no-fire departure, retained draws and importer warning are design rules, not observations
      of that undefined outcome)
    - Confidence: medium (the damage-effect draw, its conditions, the eligibility by behaviour and
      the counting from the end of the list; the building fires and their draws, including the
      position of the loop draws and the cells of every non-zero foundation word and their order,
      row by row and within a row by increasing map *x*; the zero-size foundation's lack of
      cells; the marking by a healing hit); low (the depth limit,
      the marking and non-marking by the hits listed as unmeasured, and the original's
      short-list outcome). Which of *W* and *H* is
      which does not matter to the range.

14. **Death.** A hit that takes the last health reports *now dead*. The kill is credited to the
    attacker when there is one and the attacking house is absent or is the attacker's owner, and
    otherwise to the attacking house; when both are absent nobody is credited. Each hit that takes
    the last health is credited on its own, whether or not the object was killed before (claim 16
    gives the delayed-building case; S-0312 owns repeated sinking deaths, with S-0504 claim 13
    fixing their descent consequence). So a shot of a firer that has not
    been killed, removed or (believed, low) started to cloak since the shot credits the firer (low
    for a firer loaded into a closed transport since the shot). A shot of a firer that was killed while the projectile flew credits nobody, even when the firer
    stays in play for a time (claim 1); a kill by a death weapon credits the dying
    object and its house; a trigger kill credits nobody (S-0011, claim 27); and a lightning-storm
    bolt credits its owner's house (claim 1). A kill of a terrain object credits nobody (claim 20).
    What death does (death weapons, survivors, debris, infantry death sequences) is S-0312's and
    S-0107's; experience and score from the kill are S-0311's.
    - Backing: `measurable` (scenario D24: the shot of a live firer, the shot of a firer killed
      in flight, the trigger kill, the storm bolt, the shot of a firer loaded into a closed
      transport, of an infantry firer with a death sequence, of a building in a delayed-kill
      countdown and the death weapon's kill),
      and `unmeasured` (an attacking house that differs from the owner of a present attacker: no source
      is known to deliver one, claim 1, so no scenario can produce it; the owning specs of any such
      source should add one)
    - Confidence: high (a live firer is credited; a trigger kill credits nobody); medium (a firer
      killed in flight credits nobody, also one that stays in play; a death weapon's kill credits
      the dying object); low (a firer in a closed transport is credited; a storm bolt credits its
      owner's house; an attacking house that differs from the attacker's owner)

15. **Cyborgs are crippled before they die.** When a hit that does not ignore defences takes the
    last health of an infantry unit whose type is a cyborg and that is not prone, the unit does not
    die: its health becomes a quarter of its strength, rounded down, but at least 1, it goes prone
    and crawls from then on, the rules' infantry-explode animation is played at its position, and
    the hit reports *now red*. A cyborg that is already prone, or a hit that ignores defences, kills
    it normally (S-0107, claim 17, gives how it then dies).
    - Backing: `measurable` (scenario D12, including the trigger kill of a cyborg that is not
      prone, which ignores defences)
    - Confidence: high

16. **Delayed kill of buildings.** When a hit kills a building whose type is eligible for a delayed
    kill, and the hit's warhead causes delayed kills, the building does not die yet: it keeps 1
    health, stays in play, and a countdown of *K* ticks starts (Formulas F7; quirk
    `quirks.delay_kill_whole_cell_spread`, Edge cases). Its expiry delivers the hit below, rather
    than removing the building directly. If a countdown is already running,
    the shorter of the remaining time and *K* is kept. The hit reports *post-mortem*. The kill is
    nevertheless credited at this hit, as claim 14 says: the
    destruction is registered (experience and score of the attacker, S-0311; the destroyed-object
    counts of the two houses; the destroyed-object events of the building's trigger tag) at the
    hit. The building is killed at that hit for claim 1: the detonations of projectiles it fired before
    the hit carry neither attacker nor attacking house, and those of projectiles it fires during
    the countdown carry it.

    *A hit during the countdown.* While the building still has 1 health, every further hit
    that lowers health by at least 1 kills it again, and nothing in the countdown stops the hit:
    destruction is credited again, in full, at that lethal hit (a second kill value to the
    attacker's experience and house score, a second destroyed-object count, the events raised
    again). A further lethal hit whose warhead causes delayed kills reports *post-mortem* again
    and keeps the shorter countdown; a further lethal hit by another warhead destroys the
    building at once (S-0312). Healing follows claim 11 and does not itself count as a kill.
    If healing raised health above 1, a later hit must take all of that health to credit another
    kill; nonlethal damage neither credits a kill nor changes the pending countdown.
    Contacts and the other death effects of S-0312 come only when the building is destroyed, not
    at a delayed-kill hit. This also applies to the later hits of one detonation on the building's
    other foundation cells (S-0303, claim 8): each of them that changes health by at least 1 credits
    the destruction again, so a multi-cell building can be credited once for every cell in the
    spread (the hits that the falloff or the versus value reduce to 0 do nothing, claim 8).
    S-0303 claim 8 includes this exception: a building kept alive at 1 health by a delayed
    kill remains eligible for later hits of the same detonation. Its claim 9's alive,
    positive-health, on-map and in-play tests still apply. D13 part d tests this exception. Quirk
    `quirks.delay_kill_credited_again` (default on, the original): on gives the repeated credit
    above, off ("fixed") credits a building already in a delayed-kill countdown no further
    destruction, though its countdown is still shortened.

    **Countdown expiry (Q-0081 parts 1 and 2).** Let *t* be the tick that starts or shortens
    the countdown and *K* its newly set duration. On the building's first turn at or after
    *t* + *K*, while it is still alive and in play and is neither being erased nor under an
    electromagnetic-pulse effect, a building other than a bridge repair hut receives a hit of
    its whole current health, distance 0,
    using the demolition warhead, ignoring defences, with no no-escape marker and no attacking
    house. Its attacker is the unit recorded as having planted a demolition charge, if that
    unit is still carried by the building, otherwise none. A delayed-kill hit does not record
    its attacker as that planter. In the ordinary delayed-kill case there is no planter and
    nobody receives further experience, score or an attacking-house kill count at expiry.
    The victim's loss bookkeeping and destroyed-object events run again if the expiry hit
    kills it, just as for any lethal hit (claim 14). A bridge repair hut instead demolishes its
    bridge and clears the pending countdown and recorded planter, without an
    expiry hit or a combat death. S-0511 claim 16 owns which bridge cells that action destroys.
    With `quirks.delay_kill_credited_again` off, repeated destruction registration at an expiry
    hit is suppressed too.

    **Expiry-turn actions (Q-0170).** Expiry does not deprive a still-living building of the
    actions due earlier in that same turn. Its behaviour is processed when due, including any
    ordinary firing or sale progress; its due self-healing also occurs before the expiry hit.
    A pending animation-delayed shot due in that turn is resolved under S-0301 claim 14 before
    expiry: it fires if its normal firing check permits it, otherwise it is dropped. An expired
    delayed-kill countdown alone does not refuse either kind of shot. Building repair due in
    that turn likewise precedes expiry (S-0314). The expiry hit therefore takes the health left
    after those actions, including any healing or repair, not the health at the start of the turn.
    If an earlier action removes or destroys the building, there is no subsequent delayed-kill
    expiry hit in that turn. A shot launched before a lethal expiry hit remains launched, but
    that hit cuts its carried attacker and attacking house as claim 1 says; a later detonation
    of that projectile is ownerless. These are observable consequences, not a requirement to
    reproduce any other internal ordering within the turn.

    When the expiry hit reports *now dead*, all applicable shared and building death effects
    of S-0312 occur in that same building turn, with their normal draws and order. The building
    death-sound fallback follows S-0701 claim 37: it is requested at this destruction reaction,
    not at later removal or at a delayed-kill hit that reports *post-mortem*. Its later
    removal, if any, follows S-0312 claim 21, not the countdown's duration. The hit still obeys
    column I of claim 8 and the delayed-kill rule itself. If the demolition warhead stops it,
    or causes another delayed kill, the building can survive expiry and receive another expiry
    hit on each later eligible turn. This is a consequence of the imported warhead, not an
    unconditional scheduled removal.

    Capture does not cancel the countdown: the expiry hit acts on the building with its current
    owner and current health. Selling does not itself cancel it either. If the sale removes the
    building before expiry, there is no expiry hit, no further death effect and no later credit.
    Any other removal from play has the same consequence while the building remains out of play.
    If a selling building is still alive when the countdown expires, the expiry hit occurs
    during its turn and its death follows S-0312's selling-building case. Starting temporary
    invulnerability cancels a pending delayed-kill countdown and forgets its recorded charge
    planter (S-0513 claim 25), so expiry then delivers no hit.
    - Backing: `measurable` (scenario D13, including the tick of the credit and a second hit while
      a countdown runs, and its parts c1, c2 and d for the repeated credit; parts e to h for
      expiry timing, its hit, credit, capture, sale, removal and cancellation; part i for
      behaviour, ordinary and animation-delayed firing, self-healing and repair before expiry
      and the resulting health amount and ownerless shot; part j for healing and nonlethal
      follow-up hits; D24 for the credit); `unmeasured`
      (the *post-mortem* result of the hits: it has no known observable, claim 12, so
      no scenario tests it; dropping an animation-delayed shot whose normal firing check
      refuses it on the expiry turn: D13 i aligns permitted shots only, so that refusal case
      is not discriminated here; postponement while being erased: D13 h tests electromagnetic-pulse
      blocking, not an erasing lock across expiry; the expiry hit's no-escape marker: this marker affects vehicle crew,
      not the building target, so D13 cannot distinguish its values; credit to a separately
      recorded charge planter: D13 uses no planted charge and no charge-planting scenario is
      supplied here; forgetting the recorded planter when protection cancels the countdown:
      cancellation prevents the expiry hit that could reveal that identity, so no independent
      observable distinguishes forgetting it from retaining it; removal other than sale before expiry: D13 f tests sale only, and no other
      nonlethal removal setup is identified here)
    - Confidence: medium (the countdown and its length; the credit at the hit; that the shorter
      countdown is kept; the repeated credit at a second hit, which has not been observed, and the credit for each foundation cell of one
      detonation; capture and sale during the countdown); high (the expiry hit, its markers,
      timing in the building's turn, lack of carried original attacker and attacking house,
      ordinary death effects at expiry, repeated expiry after a surviving hit, the bridge-hut
      exception, and cancellation by temporary invulnerability); low (the charge-planter credit
      case, healing and nonlethal follow-up cases, and non-sale removal cases not exercised by D13)

### Kind-specific rules

17. **Infantry above the surface and the mutating warhead.** A hit on an infantry unit whose
    height above the surface is strictly greater than 0 leptons, by a warhead of infantry-death
    kind 9 (S-0107, Terms), has a delivered amount *d*₀ of 0,
    whatever amount it carries (this is the reset that claim 8 and F2 refer to). When the hit
    does not ignore defences, the minimum of claim 5 raises that 0 to 1, so a direct hit deals trunc(*v*) (at most the cap of claim 6, for a versus value of 0 or more), the whole-number part of the versus value: nothing for
    a versus value from 0 % up to but not including 100 %, 1 at 100 %. A hit that ignores defences then does nothing.
    This is a height test, not a flight-state or type test: it also applies during ascent or
    descent at a positive height close to the surface. Standing on a high bridge deck does not
    reset the amount; the deck is the surface for that unit. Quirk
    `quirks.airborne_mutation_hit_raised` (Edge cases).
    - Backing: `measurable` (scenario D14 for ordinary hits, including its near-surface and
      bridge-deck parts; D16 part d4 for unchanged health and the resulting hang when the
      trigger kill ignores defences)
    - Confidence: medium (the reset to 0 and the later raise are both clear; the intent is not)

18. **Expected damage for targeting.** When an object fires at another, the original also computes
    an *expected damage* (Formulas F8). For a weapon of positive damage it is the shot damage
    without the garrison, bunker and open-topped multipliers, taken through the defences of claim 5
    (without the prone factor) and the modification of claim 6 at distance 0. The defences use the
    target's veteran armour bonus, but the house armour factor of the target owner's country for
    the *firer's* kind and the *firer's* object armour multiplier, not the target's (quirk
    `quirks.expected_damage_firer_armour`, Edge cases). The cases are disjoint, in this
    precedence: for a sonic-wave or fire-particle weapon the expected damage is 0, whatever its
    damage field (also when that is 0 or negative); for any other weapon whose damage field is 0
    or negative it is the damage field itself, unchanged; for every other weapon it is the
    positive-damage value above. The target acquisition spec S-0309 owns its use (the target's
    expected remaining health, which automatic searches use to avoid overkill, lowered when a search
    picks the target and when a shot is fired at it: S-0309, claims 4 and 15). It never changes
    health.
    - Backing: `measurable` (D30, part a, for the firer's kind, part c for its object armour
      multiplier in the defences, and part b for the missing prone factor; S-0309's scenario A8, part b, for the
      lowering of the expected remaining health that the expected damage causes); `unmeasured` (the
      absence of the garrison, bunker and open-topped multipliers, the damage field passing
      unchanged for a weapon of 0 or negative damage, the 0 of wave and particle weapons, and
      that the 0 of a wave or particle weapon takes precedence over the unchanged non-positive
      damage field: no scenario of this spec or of S-0309 separates them from the plain formula, since the expected
      value is visible only through the strong-mode pick that D30 uses and these differences are
      not set up there)
    - Confidence: medium (the firer's kind and multiplier, the prone factor and the effect); low
      (the unmeasured parts)

19. **When damage lands.** A detonation's hits are applied in the projectile's own turn, to every
    object it reaches, as S-0203 (claim 5) says; this spec adds no delay. Within one hit, the rules
    above take effect together: no other object's turn falls between them.
    - Backing: `measurable` (S-0203 scenarios O1 and O3)
    - Confidence: high

20. **Terrain objects.** A hit on a terrain object (Terms) is stopped, and reports *unaffected*,
    unless its warhead is one that can damage wooden things (the warhead's wood flag, Import
    mapping: the flag that also lets a hit damage wooden walls, S-0303 claim 16) and the target's
    type is not marked immune. Both tests apply to every hit, whether or not it ignores
    defences, and a hit without a warhead is stopped. A hit that passes them follows
    claims 6, 11 and 12 with the armour class of the terrain type (S-0305): for a hit that does not
    ignore defences the delivered amount takes the versus value, the falloff and the cap of claim
    6, a negative amount heals up to the strength, and a hit that ignores defences lands as it is.
    The rest of the target side (claims 5, 7, 8 and 10: the house armour factor and multipliers,
    the minimum of 1, the invulnerability, bunker, immunity, allies and berserk rows, readiness loss)
    does not apply, so a hit that gets down to an amount of 0 does nothing. A terrain object has
    no owner, so a hit never credits the destruction: no experience, score or count changes for
    anyone. When its health reaches 0 the terrain object is destroyed:
    - a terrain type that spawns ore (the type's spawns-ore flag, Import mapping) shows the
      explosion animation that S-0303 (F5) chooses for a damage of 100 with the demolition warhead
      and sets off a detonation of that warhead with the damage amount 100 at its position, with
      no source object and no source house and with resource reduction allowed (S-0303, claim 17)
    - any other terrain type crumbles: it plays a crumbling animation of its own and then leaves
      play. The length of that animation and the moment it leaves play are not traced.

    S-0303 delivers hits to terrain objects like any other object of a scanned cell, and a sonic
    wave does too (claim 1).
    - Backing: `measurable` (scenario D28 for its named ordinary-hit cases, absence of credit,
      sonic-wave reach and ore burst); `unmeasured` (the terrain rules for hits that ignore
      defences: D28 supplies only detonations and waves, neither of which carries that marker,
      and no scenario delivers an ignoring-defences hit to a terrain object; the exclusion of
      every other target-side protection: D28 does not test each such property on terrain. The
      crumbling and the moment the terrain object leaves play are also unmeasured: no scenario of this spec
      observes it, and no terrain spec owns it yet)
    - Confidence: medium (the two stopping tests, the armour class and versus, the absence of
      credit, and the ore-spawning detonation); low (that claims 6, 11 and 12 are all there is
      between the two tests and the destruction; the crumbling)

## Formulas

All integers are 32-bit (S-0205, claim 1). Every floating-point operation is one arithmetic result
of S-0205 (claim 4): written RZ₅₃ here, it is RN₅₃ with `quirks.chop_rounding` off (S-0205, claim
5). A single value is widened exactly before use. trunc is the conversion of S-0205 (claim 7).

**F1, shot damage** (claim 2). *D*: the weapon's damage field (integer). *F*ₕ: the house firepower
multiplier (double, claim 3). *F*ₒ: the object firepower multiplier (double, claim 4). *V*<sub>c</sub>: the
veteran combat factor (double). *M*<sub>g</sub>, *M*<sub>b</sub>, *M*<sub>t</sub>: the garrison, bunker and open-topped multipliers
(single).

> *E*₀ = 0 for a sonic-wave or fire-particle weapon; otherwise *E*₀ = trunc(RZ₅₃(RZ₅₃(*F*ₕ × *F*ₒ) × *D*))
> for *D* > 0, and *E*₀ = *D* for *D* ≤ 0.
>
> *E*₁ = trunc(RZ₅₃(*E*₀ × *V*<sub>c</sub>)) for a weapon that is neither sonic-wave nor fire-particle, with
> *D* > 0, fired by an object with the veteran firepower bonus; *E*₁ = *E*₀ otherwise.
>
> *E*₂ = trunc(RZ₅₃(*E*₁ × *M*<sub>g</sub>)) for a garrisoned building; *E*₂ = *E*₁ otherwise.
> *E*₃ = trunc(RZ₅₃(*E*₂ × *M*<sub>b</sub>)) for a bunkered unit; *E*₃ = *E*₂ otherwise.
> *D*ₛ = trunc(RZ₅₃(*E*₃ × *M*<sub>t</sub>)) for a firer inside an open-topped transport; *D*ₛ = *E*₃ otherwise.
> Unit: health points. This is the shot damage at the moment of firing; S-0306 (claim 18) says
> what a degenerating projectile type takes off it in flight.

House firepower multiplier of a player house in a skirmish (claim 3; the neutral and special houses have exactly 1.0): *F*ₕ = RZ₅₃(*φ*ᵣ × *φ*<sub>c</sub>), where *φ*ᵣ is the
difficulty row's firepower factor and *φ*<sub>c</sub> the country's (doubles). Unit: a
dimensionless factor.

**F2, defences** (claim 5), for a hit that does not ignore defences, with *d*₀ ≥ 0. *P*: the
warhead's prone factor (double). *A*ₕ: the house armour factor for the target's kind (single,
from the owner's country). *A*ₒ: the object armour multiplier (double). *V*ₐ: the veteran armour
factor (double).

> *d*₁ = max(trunc(RZ₅₃(*d*₀ × *P*)), 1) for a prone infantry target with *d*₀ > 0; *d*₁ = *d*₀ otherwise.
>
> *d*₂ = trunc(RZ₅₃(*d*₁ / RZ₅₃(*A*ₕ × *A*ₒ))).
>
> *d*₂′ = trunc(RZ₅₃(*d*₂ / *V*ₐ)) for a target with the veteran armour bonus; *d*₂′ = *d*₂ otherwise.
>
> *d*₃ = max(*d*₂′, 1). Unit: health points.

For a negative *d*₀, or a hit that ignores defences, *d*₃ = *d*₀ (after claim 17's reset where it
applies). For a hit that does not ignore defences and has *d*₀ ≥ 0, *d*₃ ≥ 1. The targets are the
four kinds of the scope paragraph of the target side, for which *A*ₕ is defined. For a terrain
object (claim 20), F2 does not apply and *d*₃ = *d*₀.

**F3, modification** (claim 6). *r*: the distance in leptons (S-0303). *s*: the warhead's spread
(single, cells). *p*: the warhead's edge percentage (single, a fraction). *v*: the versus value
(double). *D*ₘₐₓ: the game-wide maximum damage (integer). The six cases below are disjoint.
Cases (iii) to (vi) are for a hit that does not ignore defences, in a game that is not inert, with
a warhead:

| Case | Hit | *m*₀ |
|---|---|---|
| (i) | ignores defences | *d*₀ (claim 9) |
| (ii) | does not ignore defences, in an inert game or without a warhead | 0 |
| (iii) | *d*₃ < 0 and *r* < 8 | *d*₃ |
| (iv) | *d*₃ < 0 and *r* ≥ 8 | 0 |
| (v) | *d*₃ = 0 (claim 5 raises 0 to 1 for its four target kinds; this case remains possible for terrain objects and the berserk duration below) | 0 |
| (vi) | *d*₃ > 0 | min(trunc(RZ₅₃(max(*g*, 0) × *v*)), *D*ₘₐₓ) |

In case (vi), *g* is the amount after the falloff:

> *f* = RZ₂₄(*d*₃) (exact for *d*₃ ≤ 2²⁴), *a* = RZ₂₄(RZ₅₃(*d*₃ × *p*)), *R* = trunc(RZ₅₃(*s* × 256)) leptons.
>
> *g* = *d*₃ (no falloff) when *R* = 0 or *a* = *f*; otherwise
> *g* = trunc(RZ₅₃(RZ₅₃(RZ₅₃(RZ₅₃(*f* − *a*) × (*R* − *r*)) / *R*) + *a*)), where *R* − *r* is an
> exact integer.
>
> Unit: health points (*m*₀, *d*₀, *d*₃, *f*, *a*, *g*, *D*ₘₐₓ); leptons (*r*, *R*).

The falloff part (*f*, *a*, *R*, *g*) is shared with S-0303, which owns *r* and which objects are
hit; the two must agree. Note the two narrowings to single precision (*f* and *a*): the edge
product uses the integer defended amount *d*₃, not the narrowed *f*. The falloff radius *R* is
not the whole-cell delayed-kill radius *R*′ of F7. For *r* > *R*
the factor *R* − *r* is negative, so *g* falls below *a* and can reach 0 or below, which the max
raises to 0.

Undemolishable buildings (claim 7), ignoring defences or not: *m* = max(*m*₀, 1), which applies
to every hit that no row of claim 8 stops. For every other target, *m* = *m*₀.

**F4, health** (claim 11). *H*: health before the hit; *S*: strength.

> For *m* < 0: *H*′ = min(*H* − *m*, *S*). For *m* > 0: *m*′ = min(*m*, *H*), *H*′ = *H* − *m*′.
> Exact integer arithmetic. Unit: health points.

**F5, damage-state thresholds** (claim 12), for *m* > 0:

> *Y* = ⌊*S* / 2⌋ (exact; *S* ≥ 0). *X* = RZ₅₃(*S* × *ρ*), *ρ* the red ratio (double). Unit:
> health points (*Y*, *X*, *H*, *m*, *m*′).
>
> now yellow ⇔ *m* < *H* and *H* ≥ *Y* and *H* − *m* < *Y*.
>
> now red ⇔ *X* < *H* and *H* − *m*′ < *X* (exact comparisons of the integers with the double *X*).
>
> now dead ⇔ *H*′ = 0 (subject to claims 15 and 16). Precedence: now dead, then now red, then now
> yellow, then unchanged.

Cyborg health after crippling (claim 15): *H*′ = max(trunc(RZ₅₃(*S* × 0.25)), 1); the product is
exact, so this is ⌊*S* / 4⌋ raised to at least 1, in both settings of `quirks.chop_rounding`.

**F6, readiness loss** (claim 10). *A*: the current ammunition (integer); *A*ₘₐₓ: the type's
ammunition field (integer); *κ*: the type's readiness reduction factor (single); *S*: the strength
(integer); *d*: the amount after F2 (integer; *d*₀ when F2 does not apply). The result *A*′ is an
integer. Unit: shots of ammunition (*A*, *A*ₘₐₓ, *A*′); health points (*d*, *S*).

> *A*′ = max(trunc(RZ₅₃(*A* − RZ₅₃(*A*ₘₐₓ × RZ₅₃(*κ* × RZ₅₃(*d* / *S*))))), 0).

For *S* = 0 (the absent-key default) the quotient is infinite or not a number, so the whole
expression is infinite or not a number, and the conversion of S-0205 (claim 7) gives 0: *A*′ = 0,
whatever the sign of *d*. The ammunition is emptied. For a negative *d* the product is negative and
*A*′ can exceed *A*ₘₐₓ: the original has no upper clamp (`quirks.readiness_heal_over_max`, Edge
cases). The importer warns for a type with the
damage-reduces-readiness flag and a strength of 0.

**Berserk duration** (claim 10). *T* is the value of F3's cases for a hit that does not ignore
defences (cases (ii) to (vi)), with *r* = 0 and with *d* in the place of *d*₃, where *d* is the amount after F2 for a hit
that does not ignore defences and the delivered amount *d*₀ for a hit that does. Whether the hit
ignores defences changes only *d*, never which of F3's cases applies. No minimum for undemolishable
buildings applies (buildings are never made berserk, claim 10). Unit: countdown updates
(S-0610 claim 5), conventionally ticks; the first update may occur in the hit tick.

**F7, delayed-kill countdown** (claim 16). *F*: the warhead's delay field (integer, ticks); *α*:
its delay-at-edge factor (single); *s*: its spread (single, cells); *r*: the distance.

> *R*′ = 256 × trunc(*s*) (the spread's whole cells, not trunc(256 *s*) as in F3).
> *K* = trunc(RZ₅₃(*F* + RZ₅₃(RZ₅₃(RZ₅₃(RZ₅₃(*α* × *F*) − *F*) / *R*′) × *r*))). Unit: ticks.
>
> For a non-negative spread below one cell, *R*′ = 0 and the quotient is infinite or not a number, which the
> conversion of S-0205 (claim 7) maps to 0, so *K* = 0.

**F8, expected damage** (claim 18).

> *E* = 0 for a sonic-wave or fire-particle weapon, whatever *D*; for any other weapon, *E* = *D*
> when *D* ≤ 0; for any other weapon with *D* > 0, *E* is *E*₁ as in
> F1 (without *M*<sub>g</sub>, *M*<sub>b</sub>, *M*<sub>t</sub>), taken through F2 without the prone factor, then through F3 with
> *r* = 0. In F2 here, *A*ₕ is the factor of the target owner's country for the firer's kind,
> *A*ₒ is the firer's object armour multiplier, and the veteran armour bonus is the target's
> (`quirks.expected_damage_firer_armour` on, the original). With the switch off, *A*ₕ and *A*ₒ are
> the target's, as in F2. Unit: health points.

**Dependence on the rounding mode.** F1, F2, F3 (the falloff and the versus product), F5 (*X*), F6
and F7 depend on `quirks.chop_rounding` (S-0205): with it off, every RZ₅₃ above is RN₅₃ and every
RZ₂₄ is RN₂₄. Products of a whole number below 2²⁴ with a factor of 1.0, 0.25 or a power of two are
exact in both settings. Imported decimals follow S-0003, not a conversion straight to the
nearest double. For the red ratio written as 0.6 without a percent sign, *ρ* is the widened
single 0.60000002384185791015625. With *S* = 1000, *X* is exactly
600.00002384185791015625 health points in both rounding modes: 601 > *X* and 600 < *X*,
so the hit from 601 to 600 crosses the red threshold. This example distinguishes input
precision, not arithmetic rounding mode. Non-exact products and F6's quotient *d* / *S*
can still differ between the modes. Most scenarios use binary-exact ratios and quotients at
integer boundaries; D11 part d deliberately tests the non-exact decimal input.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Healing reach | 8 | leptons | Claim 6: a negative hit heals only at a distance below this |
| Spread scale | 256 | leptons per cell | F3 and F7; a single-precision constant, exact |
| Yellow threshold | ⌊*S* / 2⌋ | health points | Claim 12; fixed, not the yellow ratio |
| Cyborg crippled health | ⌊*S* / 4⌋, at least 1 | health points | Claim 15 |
| Minimum after defences | 1 | health points | Claim 5 |
| Minimum for undemolishable buildings | 1 | health points | Claim 7 |
| Damage particle depth limit | 10 | leptons below the surface | Claim 13: the effect starts only at a height above −10 |
| Maximum damage, absent-key default | 1000 | health points | Claim 6 |
| Infantry-death kind of the mutating warhead | 9 | infantry-death kind (S-0107) | Claim 17: resets the amount of a hit on infantry at positive height above the surface |
| Red ratio, absent-key default | 0.5 | fraction | Claim 12; as S-0107, S-0108 and S-0111 |
| Prone factor, absent-key default | 1.0 | factor | Claim 5 |
| Edge percentage, absent-key default | 1.0 | fraction | F3: no falloff |
| Spread, absent-key default | 0 | cells | F3: no falloff |
| Versus value, absent-key default | 1.0 | factor | S-0003 claim 13 |
| House and object multipliers, absent-key default | 1.0 | factor | Claims 3 and 4 |
| Can-be-demolished flag, absent-key default | yes | boolean | Claim 7 |
| Disguise disruption countdown | 2 | ticks per health point lost | Claim 12: 2*m*′ ticks |
| Building fire draw range | 0 … *W* + *H* + 5 | draw values | Claim 13, one draw per foundation cell |
| Building fire bands | 1 to 5, 6 to 8, 9 | draw values | Claim 13: first, second, third damage-fire animation; other values none |
| Building fire loop draw range | 1 … 3 | loops | Claim 13, for fires of the first and second kind |
| Ore burst damage | 100 | health points | Claim 20: the detonation, with the demolition warhead, of an ore-spawning terrain object that is destroyed |
| Building light-source factor on *now yellow* | not stated (Open questions) | factor | Claim 12; scenario D11 part c records it |

## State and ordering

Damage-state results (claim 12) and what reports them:

| Result | When |
|---|---|
| unaffected | Claim 8 stopped the hit (except a berserk hit that made the target berserk, claim 10), claim 20 stopped a hit on a terrain object, or the hit healed (claim 11) |
| unchanged | Health fell without crossing a threshold, or the hit made the target berserk (claim 10) |
| now yellow | Health crossed half the strength, and the target survived |
| now red | Health crossed strength × red ratio, or a cyborg was crippled (claim 15) |
| now dead | Health reached 0 |
| post-mortem | A hit put a building into a delayed kill, or hit it again with a delayed-kill warhead during the countdown (claim 16), or the target left play during the reactions to the hit (claim 12) |

A still-living building whose delayed-kill countdown expires in its turn can make a due
ordinary shot or resolve a due animation-delayed shot before the expiry hit. Due self-healing
and paid repair affect the whole-current-health amount of that hit (claim 16, D13 part i).

Observable consequences within one hit:
- Armour bonuses alone never save a target from a hit, but a low versus value does, because the
  minimum of 1 applies to the defended amount (F2) and not to the result of the versus value (F3):
  a hit of 1 against a versus value of 50 % does nothing, with or without armour bonuses (except on an undemolishable
  building, which loses 1 from every hit that no row of claim 8 stops, claim 7).
- The falloff and the versus value are truncated separately (F3). Considering these two
  integer truncations alone, before the cap and without floating-point rounding error, the
  difference from the untruncated product is less than 1 + |*v*| health points in absolute
  value: less than 2 for a versus value between −100 % and 100 %, and less than 3.5 at 250 %.
  F3's floating-point rounding, especially the narrowings for large amounts, can add further error.
- The maximum damage limits the amount after the falloff and the versus value (F3).
- Readiness loss happens for every hit that no stage 1 row of claim 8 stops, even when a row of
  stages 2 to 4 then stops it (claim 10), so a type immune to a warhead (radiation, psychic,
  poison), a bunkered unit, or a target that is immune to all damage still loses ammunition to it.
  A hit that a stage 1 row of claim 8's table stops costs no ammunition (for example a + hit on a
  temporarily invulnerable target, or a + or − hit on a target being erased; the table lists every
  row and the columns it stops).
- A berserk hit is stopped by the rows of stages 1 and 2 but not by those of stage 4: a target
  immune to all damage still goes berserk, a target immune to radiation does not go berserk from a
  radiation warhead that also makes targets berserk.
- The invulnerability row's effect is sized by the amount after the defences, so an invulnerable
  target still shows the effect of a hit of twice the defended amount.
- The sign that selects a column of claim 8 is the sign of the delivered amount, so a positive hit
  against a negative versus value is stopped by invulnerability and then heals nobody.

## Edge cases and quirks

- **`quirks.undemolishable_heal_is_damage`** (claim 7; default on, the original). On: the
  minimum of 1 applies to every hit on an undemolishable building that no row of claim 8 stops,
  including a hit whose amount is 0 or negative, so healing hits damage it. Off ("fixed"): a result of 0 becomes 1, but a
  negative result heals the building normally.
- **`quirks.yellow_state_at_half`** (claim 12; default on, the original). On: *now yellow* is
  reported when health crosses half the strength, whatever the yellow ratio. Off: it is reported
  when health crosses strength × yellow ratio (the product as F5's *X*, with the yellow ratio in
  place of the red one), consistent with the health bar of S-0111. The two settings agree only
  when the yellow ratio is one half.
- **`quirks.airborne_mutation_hit_raised`** (claim 17; default on, the original). On: the hit
  delivers 0 to infantry at positive height above the surface, which the minimum of claim 5
  raises to 1. Off: such infantry take no damage from this warhead (the 0 is not raised to 1).
  Neither setting changes the damage to infantry standing at height 0, including on a bridge deck.
- **`quirks.readiness_from_defended_damage`** (claim 10; default on, the original). On: readiness
  loss uses the amount after the defences but before the versus value, and happens even when a
  later rule stops the hit. Off: it uses the final amount *m* of the hit (after claim 7) and does
  not happen when claim 8 stops the hit.
- **`quirks.delay_kill_whole_cell_spread`** (claim 16; default on, the original). On: the countdown
  uses the spread's whole cells (F7), so a non-negative spread below one cell gives a countdown of 0. Off: it uses
  *R* of F3.
- **`quirks.delay_kill_credited_again`** (claim 16; default on, the original). On: every hit that
  takes the last health of a building in a delayed-kill countdown credits its destruction again (the
  experience and score of the attacker, the destroyed-object counts, the destroyed-object events),
  including the later hits of the detonation that started the countdown. Off ("fixed"): the
  destruction of a building is credited once, at the first hit that starts a delayed kill, and the
  later hits on it are credited nothing (the countdown still shortens as claim 16 says).
- **`quirks.expected_damage_firer_armour`** (claim 18; default on, the original). On: the expected
  damage divides by the house armour factor of the target owner's country for the firer's kind and
  by the firer's object armour multiplier. Off ("fixed"): it uses the target's kind and the target's
  multiplier, as a real hit does (F2). The two agree whenever the firer and the target are of the
  same kind and both object armour multipliers are 1.0.
- **Maximum damage of 0 or below** (claim 6). The cap then replaces every result, including a
  result of 0, that is not below it with the maximum itself (F3, case (vi)). So with a negative
  maximum, every hit that reaches the cap and whose result is not below the maximum heals by the
  maximum's absolute value, and with a maximum of 0 every such hit whose result is not negative
  does nothing. A result already below the maximum (from a negative versus value) keeps its own
  value. Reproduced, and no quirk switch: the maximum is the mod author's own
  number, a value below 1 is a data error and not a rule that a mod could want played another way,
  and a "fixed" meaning (ignore the cap, or treat it as 1) would replace one meaningless result
  with another of our own invention. The importer warns for a maximum below 1 (scenario D3, part
  c).
- **A hit without a warhead on a prone infantry unit** (claim 5). The original's result is
  undefined (a crash at best). Our engine uses the factor 1.0 (a departure); such
  hits come only from scripted sources and from a debris piece of a type with no warhead (S-0317
  claim 7). Claim 6 gives such a hit an *m*₀ of 0; on an undemolishable building claim 7 still
  raises it to 1.
- **Building fires with a short fire list** (claim 13). When the drawn kind of fire has no entry in
  the rules' list of damage-fire animations, the original's result is undefined. Our engine starts
  no fire for that cell (a departure), still takes the draws claim 13 gives, and the importer warns when the list
  has fewer than three entries.
- **Healing ignores the versus value and works only within 8 leptons** (claim 6). This is how
  healing weapons reach only the object they aim at; it is reproduced and needs no switch.
- **The minimum-damage field has no effect** (claim 6), and neither have the difficulty row's armour
  factor and the country's general armour factor (claim 3). The importer warns when a rules file
  gives the minimum-damage field a value, or gives either armour factor a value other than 1.0,
  saying that the original ignores it. No switch: a "fixed" meaning
  would be our invention.
- **Readiness loss with a strength of 0** (claim 10, F6). A type with the damage-reduces-readiness flag
  and a strength of 0 loses all its ammunition to any hit that passes stage 1 (F6). Reproduced, with
  an importer warning, because it comes only from data that gives no strength.
- **A berserk duration of 0 or less** (claim 10). S-0610 claims 4 and 5 own the entry effects
  and expiry at the next countdown update. The switch `quirks.berserk_zero_duration`, owned by
  S-0610 claim 5, defaults to those original effects; off, a hit with *T* ≤ 0 has no berserk
  effect. The duration calculation here is unchanged. The importer warns for a berserk warhead
  whose damage field is 0 or negative.
- **`quirks.readiness_heal_over_max`** (claim 10, F6; default on, the original). On: a negative
  hit on a type with the damage-reduces-readiness flag raises its ammunition by F6 with no upper
  limit, so ammunition can exceed the type's ammunition field (a healing weapon aimed at such an
  aircraft type, say). Off ("fixed"): the ammunition after the hit is at most the type's ammunition
  field (an ammunition above the field before the hit is left as it is). Low confidence that the
  original has no clamp (scenario D10, third run, uses test types).
- **Overflow.** A delivered amount near the integer limits (only from extreme data) wraps as S-0205
  (claim 1) says; health arithmetic on such values follows F4 with wrapping. The importer warns
  for a weapon damage field whose magnitude exceeds 2²⁴, where F3's narrowing to single precision
  starts to change the value.
- **Rounding mode.** The formulas that depend on `quirks.chop_rounding` are listed under Formulas.
  S-0205 already indexes F1, F2, F3, F5, F6 and F7 of this spec in its effects table.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, weapon section, `Damage` | *D* (F1) | integer | 0 health points |
| Rules, weapon section, `IsSonic`, `UseFireParticles` | sonic-wave and fire-particle weapons carry 0 (claim 2) | boolean | no |
| Rules, warhead section, `Verses` | versus values *v* (F3) | S-0003 claim 13, class order S-0305 | 100 % each |
| Rules, warhead section, `CellSpread` | spread *s* (F3, F7; S-0303 owns its other uses) | decimal (S-0003), held as a single | 0 |
| Rules, warhead section, `PercentAtMax` | edge percentage *p* (F3) | decimal with optional `%` (S-0003), held as a single | 1.0 |
| Rules, warhead section, `ProneDamage` | prone factor *P* (F2) | decimal with optional `%` (S-0003), held as a double | 1.0 |
| Rules, warhead section, `InfDeath` | infantry-death kind 9 of claim 17 | integer (S-0107) | 0 |
| Rules, warhead section, `PenetratesBunker`, `Radiation`, `PsychicDamage`, `Poison`, `AffectsAllies`, `Psychedelic` | claim 8 rows, claim 10 | boolean | no, no, no, no, yes, no |
| Rules, warhead section, `Sparky` | the warhead sets building fires (claim 13) | boolean | no |
| Rules, warhead section, `Wood` | the warhead can damage terrain objects (claim 20); S-0303 (claim 16) owns its use on walls | boolean | no |
| Rules, terrain type section, `SpawnsTiberium` | ore-spawning terrain type of claim 20 | boolean | no |
| Rules, combat-damage section, `C4Warhead` | demolition warhead of claims 1, 16 and 20; claim 16's expiry-turn hit uses current health after due actions (S-0011 Import mapping owns the row) | warhead name | no warhead |
| Foundation word of a building type (S-0108, Import mapping, foundation vocabulary): `3x3Refinery` | the 8-cell 3 × 3 foundation word of claim 13 | word (S-0108) | S-0108 |
| Rules, general section, `DamageFireTypes` | the three damage-fire animations of claim 13, in list order | list of animation names (S-0003) | empty |
| Rules, warhead section, `CausesDelayKill`, `DelayKillFrames`, `DelayKillAtMax` | claim 16, F7 (*F*, *α*) | boolean; integer; decimal (S-0003), held as a single | no; 5 ticks; 1.0 |
| Rules, object type section, `Strength` | strength *S* | integer | 0 health points |
| Rules, object type section, `Armor` | armour class (S-0305) | S-0305 | none |
| Rules, object type section, `Immune` | immune row of claim 8; also immune terrain types (claim 20) | boolean | no |
| Rules, unit or building type section, `TypeImmune` | type-immune row of claim 8 | boolean | no |
| Rules, unit or building type section, `ImmuneToRadiation`, `ImmuneToPoison` | claim 8 rows | boolean | no |
| Rules, infantry, vehicle, aircraft or building type section, `ImmuneToPsionicWeapons`, `ImmuneToPsionics` | claim 8 psychic row, claim 10 berserk exception (S-0610 claim 1) | boolean | yes for building types, no for others |
| Rules, unit or building type section, `VeteranAbilities`, `EliteAbilities` | veteran firepower and armour bonuses (owned by S-0311) | list (S-0311) | empty |
| Rules, unit or building type section, `Ammo`, `DamageReducesReadiness`, `ReadinessReductionMultiplier` | *A*ₘₐₓ and claim 10, *κ* (F6) | integer; boolean; decimal (S-0003), held as a single | −1 (no ammunition limit, S-0301); no; 0.0, so no loss |
| Rules, infantry type section, `Cyborg` | claim 15 | boolean | no |
| Rules, building type section, `CanC4` | undemolishable when no (claim 7) | boolean | yes |
| Rules, building type section, `DamageSelf` | self-damage row of claim 8 | boolean | no |
| Rules, building type section, `WeaponsFactory` | vehicle-factory row of claim 8 | boolean | no |
| Rules, building type section, `LaserFence` | laser-fence row of claim 8 | boolean | no |
| Rules, building type section, `BridgeRepairHut` | bridge-repair-hut row of claim 8 (with `Immune`) | boolean | no |
| Rules, unit, infantry or aircraft type section, `DamageParticleSystems` | the list of claim 13, in list order | list of particle system names (S-0003) | empty |
| Rules, particle system section, `BehavesLike` | eligible for claim 13 when the word is `Smoke` (letter case ignored); `Gas`, `Fire`, `Spark`, `Railgun` and unknown words are not | word | none, not eligible |
| Rules, building type section, `BuildCat` | defence building only when `Combat` (claim 5) | first 31 bytes, then trim surrounding whitespace, whole-word comparison ignoring A–Z case. Accepted: `DontCare` (unspecified), `Tech`, `Power`, `Resource`, `Infrastructure`, `Combat`. Unknown nonempty word gives `DontCare`, not the held value | keeps held category, initially `DontCare` (not a defence) |
| Rules, building type section, `EligibleForDelayKill` | claim 16, including expiry-turn actions; eligibility does not suppress due shots before expiry | boolean | no |
| Art, building image section, `IsAnimDelayedFire`, `DelayedFireDelay` | pending shot resolved before delayed-kill expiry (claim 16); shot timing and firing check owned by S-0301 claim 14 | boolean; integer, ticks (S-0301 Import mapping) | S-0301 Import mapping |
| Rules, combat-damage section, `MaxDamage` | *D*ₘₐₓ (F3) | integer | 1000 |
| Rules, combat-damage section, `MinDamage` | nothing (claim 6); importer warning when given | integer | — |
| Rules, combat-damage section, `OccupyDamageMultiplier`, `BunkerDamageMultiplier`, `OpenToppedDamageMultiplier` | *M*<sub>g</sub>, *M*<sub>b</sub>, *M*<sub>t</sub> (F1) | decimal (S-0003), held as a single | 1.0 each |
| Rules, general section, `VeteranCombat`, `VeteranArmor` | *V*<sub>c</sub> (F1), *V*ₐ (F2) | decimal (S-0003), held as a double | 1.0 each |
| Rules, general section, `InfantryExplode` | the animation of claim 15 | animation name | none |
| Rules, audio-visual section, `ConditionRed` | red ratio *ρ* (F5) | decimal with optional `%` (S-0003 row C9): decimal read as a single, widened exactly, then optional percent multiplication as a double, held as a double | 0.5 |
| Rules, audio-visual section, `ConditionYellow` | yellow ratio (claim 13; and F5 with the quirk off) | as above | 0.5 |
| Rules, difficulty sections `[Easy]`, `[Normal]`, `[Difficult]`, `FirePower` | *φ*ᵣ (claim 3) | decimal (S-0003), held as a double | 1.0 |
| Rules, difficulty sections, `Armor` | nothing (claim 3); importer warning when given a value other than 1.0 | decimal | 1.0 |
| Rules, country section, `Firepower` | *φ*<sub>c</sub> (claim 3) | decimal (S-0003), held as a double | 1.0 |
| Rules, country section, `ArmorInfantryMult`, `ArmorUnitsMult`, `ArmorAircraftMult`, `ArmorBuildingsMult`, `ArmorDefensesMult` | *A*ₕ for infantry, vehicles, aircraft, other buildings, defence buildings (F2) | decimal (S-0003), held as a single | 1.0 |
| Rules, country section, `Armor` | nothing (claim 3); importer warning when given a value other than 1.0 | decimal | 1.0 |
| Rules, AI section, `CompEasyBonus` | the row shift of claim 3 (note for AI designers) | boolean | not traced (low) |
| Map, the special-flags section, `Inert` | inert game (claim 6) | boolean | no |
| Rules, power-up section, `Armor`, `Firepower` | crate factors for *A*ₒ and *F*ₒ (claim 4; S-0520 claims 10 and 19 own them) | S-0520 Import mapping | S-0520 Import mapping |

## Measurement scenarios

All scenarios use the measurement setup with named test files: a test map with override sections
defining test weapons, warheads and types (invented values, never stock ones). Ordinary mods
and engine add-ons are off; only the named test files are active. No external automation gives
orders or changes game state, except for the scripted input specified by the scenario. Health values are
read with a recorder that only reads the running game's state; the health bar is a cross-check.
One human house and one computer house, the computer house one bystander computer player (glossary)
whose objects that a scenario names are its test objects; the observed area is the area of the test
objects. A scenario that needs a house allied with the human house (D8 part c; D17 parts b, g, g2
and g3) has one allied and one enemy bystander computer player instead, and "the computer house"
is the enemy one. Every house has no difficulty or country multipliers other than 1.0 unless the
scenario says otherwise (the test map sets the country factors and the difficulty rows'
firepower to 1.0 where it matters). Any game speed. Every hit is a direct hit (distance 0)
unless stated. Random outcomes are used in D11's and D22's choice of effect, D18's fires and
D28 part f's crew creation and health draws; the latter fixes escape chance to 1.0 and tests
crew presence, not the drawn health. Each scenario runs 3 times and must give the same health values every time (D18 and D22 are statistical and
say how). Parts d, d2 and d3 of D24 depend on where the bolts of a superweapon strike, which this
spec does not own: each of their runs is checked against the part's own expectation, which is
stated per destroyed vehicle, and the runs need not give equal values. For every deterministic
part below, any valid run whose recorded observable differs from its stated expected result
falsifies that part, not only the illustrative alternatives listed as falsifiers. Statistical
parts use their stated replay checks and rejection thresholds instead.

- **D1 (claims 2, 6, 11: versus and truncation).** A test weapon with damage 37, a projectile that
  hits its target directly, and a warhead without spread whose versus values for eleven test
  vehicle types (one per armour class, S-0305) are 100 %, 50 %, 33 %, 25 %, 10 %, 1 %, 150 %, 0.5
  (without `%`), 2 %, 99 % and 75 %. A test unit of the human house attacks each vehicle (strength
  1000) once. *Expected:* losses 37, 18, 12, 9, 3, 0, 55, 18, 0, 36, 27 (each trunc(37 × *v*), with
  33 % giving 12.21 → 12 and 99 % giving 36.63 → 36). *Falsified if* any loss differs, in particular
  a loss rounded to nearest (19 for 50 %, 37 for 99 %) or a minimum of 1 for the 1 % and 2 % vehicles.
- **D2 (claims 2, 3, 5: firepower, house armour, minimum before versus).** Test map 1: the human
  house's country gets a firepower factor of 1.25 and the computer house's country a vehicle armour
  factor of 0.75 (country keys of the Import mapping; both values are exact in binary). A human unit
  with a test weapon of damage 7, whose warhead is 100 % against the target's class, attacks a
  computer vehicle of strength 1000. *Expected:* shot damage trunc(7 × 1.25) = 8, defended
  trunc(8 / 0.75) = 10, loss 10 per hit. Test map 2: damage 1, the computer country's vehicle armour
  factor 4.0, versus 100 %: loss 1 (trunc(0.25) = 0, raised to 1). The same with versus 50 %: loss 0.
  *Falsified if* map 1 loses 11 (one truncation of 7 × 1.25 / 0.75 = 11.67, so no truncation of the
  shot damage), 9 or 12 (rounding), or map 2 loses 0 at 100 % or 1 at 50 %.
- **D3 (claim 6: cap and unused minimum).** Test map whose combat-damage section gives the maximum
  damage 10 and the minimum damage 5 (Import mapping). Weapons of damage 25 (versus 100 %) and 3 (versus 100 %) hit
  test vehicles of strength 1000. *Expected:* losses 10 and 3. *Falsified if* the first loses 25 or
  the second loses 5. (c) *A maximum of 0 or below.* With the maximum damage 0, then −5, a weapon
  of damage 25 (versus 100 %) hits an enemy test vehicle of strength 1000 that is at health 500.
  *Expected:* health stays 500 with the maximum 0; it rises to 505 with the maximum −5 (and the
  same when the versus value is 0 %). *Falsified if* the vehicle loses health, or is unchanged
  with −5.
- **D4 (claims 6, 11: healing).** A test infantry type with a weapon of damage −20 whose warhead
  has versus 50 % against the target's class (so the target is legal, S-0304), aimed at a damaged
  own vehicle at health 100 of strength 1000. *Expected:* +20 per hit (not +10), and health never
  above 1000. A second run gives the warhead a spread of 2 cells; vehicle *A* is the target and a
  damaged vehicle *B* stands in a neighbouring cell, clearly inside the spread but more than 8
  leptons from the point of impact. *Expected:* only *A* heals. *Falsified if* a hit heals 10, or *B* heals.
- **D5 (claim 7).** Two test building types, one with the can-be-demolished flag on and one with it
  off, each of strength 1000, attacked by a weapon of damage 50 with versus 1 % against their class.
  *Expected:* the demolishable one loses 0 per hit, the undemolishable one 1 per hit. A second run
  aims the healing weapon of D4 (with the building class at 100 %) at a damaged undemolishable
  building of the own house: it loses 1 per hit. *Falsified if* the undemolishable building loses 0
  in run 1, or heals in run 2.
- **D6 (claim 8: invulnerability).** A computer vehicle of strength 1000, damaged to health 500
  before the invulnerability superweapon effect starts, is under that effect, hit by D1's weapon at
  100 % and healed by D4's weapon. *Expected:* no loss while the effect lasts; the heal applies
  (+20 per hit). *Falsified if* it loses health, or the heal is blocked.
- **D7 (claim 5: prone factor).** A test warhead with prone factor 50 % and versus 100 %, damage 9,
  hits a standing and a prone infantry unit of one test type (strength 1000). *Expected:* 9 and 4.
  With damage 1: 1 and 1 (trunc(0.5) raised to 1). *Falsified if* the prone unit loses 5, or 0 at
  damage 1.
- **D8 (claim 8: allies, immune, type-immune, bunker).** (a) A warhead that does not affect allies,
  with spread 2 cells, detonates between an enemy vehicle and an own vehicle: only the enemy loses
  health. (b) A test vehicle type marked immune is hit by D1's weapon: no loss. (c) Two vehicles of
  one type marked immune to its own type, same owner, one forced to fire at the other: no loss,
  repeated with different owners (allied): loss. (d) A stock tank bunker holding a vehicle, attacked
  by a warhead that penetrates bunkers: the bunker loses nothing and the vehicle inside loses
  health only if the area reaches it; attacked by a warhead that does not: the bunker loses health
  and the vehicle inside nothing. *Falsified if* any of these differs.
- **D9 (claims 6, 7: inert).** A test map whose special-flags section marks it inert. (a) D1's
  weapon at 100 % hits a vehicle, and D5's weapon an undemolishable test building: neither loses
  anything (S-0303 claim 2: the detonations deliver no hits). (b) A unit of a stock type whose
  weapon delivers its damage through a sonic wave (its stock weapon, with its warhead's versus value
  for the targets' class set to 100 % in the test map) fires through a test vehicle and the
  undemolishable test building. *Expected:* the vehicle loses nothing; the building loses exactly 1
  per wave hit. *Falsified if* anything loses health in (a), the vehicle loses health in (b), or the
  building loses 0 or more than 1 per wave hit in (b).
- **D10 (claim 10: readiness loss).** A test aircraft type with ammunition 10, the damage-reduces-
  readiness flag and a readiness reduction factor of 1.0, strength 100, is hit by a weapon of damage
  30 with versus 50 % against its class. *Expected:* health loss 15; ammunition from 10 to 7 (F6 with
  the defended amount 30), not to 8 (which the final amount 15 would give). A second run makes the
  aircraft's type immune to radiation and uses a radiation warhead: no health loss, and the
  ammunition still falls from 10 to 7. A third run uses a second test aircraft type that is
  otherwise the same but has strength 128 (so that *d* / *S* is exact in binary), at full ammunition
  (10), hit by a healing weapon (damage −32, versus 100 %), and reads the ammunition at once.
  *Expected:* 12 (F6 with *d* = −32: 10 + 10 × 0.25 = 12.5, truncated), above the type's field, in
  both settings of `quirks.chop_rounding`. (The quotient of the first run's numbers, 30 / 100, is
  not exact in binary, but the result 7 is the same in both settings; a healing amount of −20 on
  strength 100 would give 11 with the original's rounding and 12 with round-to-nearest, so it is
  not used.) With `quirks.readiness_heal_over_max` off the same run gives 10 (the
  switch's alternative, not a falsifier). *Falsified if* ammunition differs from 7 in run 1 or 2, or from 12 in run 3 with the switch on,
  or the stated health outcomes differ.
- **D11 (claims 12, 13: thresholds and damage effect).** All parts use a stock vehicle type whose
  type lists damage particle systems, at least one of them of smoke behaviour (strength set to 1000
  in the test map), hit by hits of exactly 100 (damage 100, versus 100 %) from 1000 down, except
  part c and the second run of part b, which say their own strength. *Observable:* the tick at which a damage effect first appears, and the health then.
  - (a) *Yellow threshold.* The test map sets the yellow ratio to 0.75 and the red ratio to 0.25.
    *Expected:* no effect appears at 800 or 700 (the bar becomes yellow at 750, but no hit crosses
    half the strength), the effect appears on the hit to 400 (now yellow: from 500 to 400), and
    the hit to 200 (now red) does not add a second one. A second test map with the yellow ratio
    0.5 makes the effect appear at the same hit. *Falsified if* the effect appears on the hit from
    800 to 700 (a *now yellow* at the yellow ratio), or never appears.
  - (b) *Red threshold.* The yellow ratio is 0.75 and the red ratio 0.75, so that the first hit
    to cross the red threshold (750) leaves the health ratio at or below the yellow ratio while
    health is still above half the strength. *Expected:* the effect appears on the hit to 700
    (from 800: *now red*), not later. Run again with the strength 1600 (set in the test map, and the
    hits of 100 from 1600), the red ratio 0.625 (threshold 1000, exact in binary, so the same in both
    rounding settings) and the yellow ratio 0.75: the effect appears on the hit to 700 (from
    800 to 700, *now yellow*), not on the hits to 1000 or 900, because a hit that ends exactly on the red threshold
    does not cross it, and the next hit starts on it. *Falsified if* the first run shows the effect only at 400 or later (a red
    threshold fixed at half the strength would give that), or at
    800; or if the second run shows it at 1000 (non-strict comparison) or 900.
  - (c) *Red takes precedence over yellow.* A test building type with a light source (S-0104), of
    strength 1000, is hit once by a hit of 600 (1000 to 400, which crosses half the strength, so
    it crosses the yellow threshold). The recorder reads the intensity of the building's light
    source before and after. In run c1 the red ratio is 0.25 (the hit crosses only the yellow
    threshold, *now yellow*); in run c2 it is 0.75 (the hit crosses both, *now red*). *Expected:*
    in c1 the intensity changes by the fixed factor of claim 12 (the recorded ratio is the same
    in repeated runs and differs from 1); in c2 it does not change. *Falsified if* the intensity
    changes in c2 (yellow wins over red), or not in c1.
  - (d) *Decimal input precision (Q-0075).* A test vehicle with strength 1000, a single smoke
    damage particle system, red ratio written as 0.6 without a percent sign and yellow ratio
    0.75 starts at health 601 with no damage effect. A weapon of damage 1, versus 100 %, hits
    it twice. Record health and the first tick of the smoke effect, not just health-bar colour:
    the latter uses a different comparison. *Expected:* the hit to 600 reports *now red* and
    starts the smoke effect, because F5's threshold is just above 600. *Falsified if* it first
    appears at 599, or no effect appears. As a second run, a test building with a light source,
    strength 1000 and red ratio 0.25 starts at 601 and takes a hit to 600, with yellow ratio
    written as 0.6. On our engine only, with `quirks.yellow_state_at_half` off, that hit scales
    the light source (the widened-single yellow threshold), whereas on the original it does not
    (fixed half). This second run checks the yellow-ratio import, not the original's red claim.
- **D12 (claim 15).** A test map marks a stock infantry type as a cyborg and sets its strength to
  200. (a) A cyborg of that type that is not prone is killed
  by a hit of 500: it survives with health 50, prone and crawling, and dies at the next lethal hit. (b) A second
  cyborg of the same type that is not prone is destroyed by the trigger kill of S-0011 (claim 27),
  which ignores defences: it dies at once. In (a) the test map also names a distinct test animation
  as the rules' infantry-explode animation, and the recorder lists the animations in play.
  *Expected:* that animation starts at the unit's position at the crippling hit. *Falsified if* it
  dies at once in (a), survives with another health in (a), survives in (b), or the animation does
  not start at the crippling hit.
- **D13 (claim 16).** A test warhead that causes delayed kills, with delay 300 ticks, delay-at-edge
  factor 1.0 and spread 1.0 cell; a test building type eligible for a delayed kill, health 10, hit
  by 20. The attacker is a trainable test unit with known experience (S-0311). *Expected:* the
  building stays at health 1 and takes its expiry hit in its turn at tick *t* + 300, *t* the
  tick of the killing hit, with the demolition warhead set not to cause delayed kills or to stop
  the expiry hit. The attacker's experience rises at the initial hit, not at expiry.
  With spread 0.5 cells (*K* = 0): expiry happens in the same tick if the building's turn is still
  ahead, otherwise in its next turn; no expiry hit occurs inside the initial hit. Two further runs test a
  second hit while a countdown runs, with a warhead of delay 300, delay-at-edge factor 0.5 and
  spread 3 cells (so *K* = 300 at distance 0 and *K* = 200 at distance 512 leptons; the recorder
  logs the distance each hit carries), on a building that the first hit left at health 1. (c1) The
  first hit at distance 0 (*K* = 300), a second at tick 50 at distance 512 (*K* = 200, shorter than
  the 250 ticks that remain): the building is destroyed about 250 ticks after the first hit (200
  ticks after the second), not at 300. (c2) The first hit at distance 512 (*K* = 200), a second at
  tick 50 at distance 0 (*K* = 300, longer than the 150 that remain): destroyed about 200 ticks
  after the first hit, not at 350. *Credit in c1
  and c2:* the test building's type is scored (not marked unscored, S-0311) and costs little
  enough that two kill values stay below the veteran cap for the attacker (a rookie trainable test
  unit that attacks, S-0311 scenario X1's setup); the recorder reads the attacker's experience and
  the attacking house's count of destroyed buildings after each hit. *Expected:* both rise at the
  first hit, and rise again, by the same amounts, at the second hit (two kill values and a count
  of 2 in all, one pair per hit, the second not at tick 300 and not absent). (d) *One detonation,
  several cells.* The same warhead as c1 (damage high enough to kill from full health, spread 3
  cells) detonates at the centre of a 2 × 2 building's foundation so that all four cells are
  inside the spread and the falloff leaves each hit above 0. *Expected:* four hits, the attacker's experience and the count of destroyed buildings rising four times
  (one kill value and one count per hit), the countdown being the shortest *K* of the four hits.
  *Falsified if* it dies at once in the first run, the experience rises only when the building is
  destroyed, it survives about 300 ticks in the second, (c1) ends at 300 or (c2) at 350 or later,
  or the experience and count rise only once in c1, c2 or d (the credit then happens only at the
  first hit, as `quirks.delay_kill_credited_again` off would give).
  (e) *Expiry timing, credit and effects (Q-0081).* Repeat the first run with a one-cell building,
  no planted charge, and a recorded update order. Make two runs, the killing projectile before
  and after the building in that order. Record its health, destruction events, the firer's
  experience, both houses' destruction/loss counts, animations, contacts and the scenario stream
  through tick *t* + 301. *Expected:* for a duration of 300 both runs take the expiry hit in
  the building's turn at *t* + 300. With duration 0 only the first run expires at *t*, the other
  at its next turn. Shared and building death effects start at expiry with S-0312's draws,
  rather than at the initial hit. No second experience, score or kill count goes to the original
  firer, but the victim's loss count and destruction events occur again. Falsified by expiry
  in a global timer part, by death effects at the initial hit, by credit to the original firer
  at expiry, or by no expiry hit. Event-recording input and the read-only state recorder are
  the tools; check stream draws by replay with S-0202 and S-0312.
  (f) *Sale, capture and removal.* Repeat with duration 300. Begin a sale early enough for
  S-0408's sale to remove the building before expiry. *Expected:* no expiry event or combat
  death effects at *t* + 300. In another run capture the building before expiry without changing
  the object or its countdown. *Expected:* expiry at *t* + 300 on the new owner's building,
  with no credit to the original firer, and the new owner's loss bookkeeping. A third run
  begins selling shortly before expiry, with a build-down that is still running then:
  death effects occur at expiry while selling, and S-0312 owns its subsequent removal.
  *Falsified if* capture clears or restarts the timer, removal leaves a scheduled death behind,
  or a still-alive selling building skips its expiry hit.
  (g) *An expiry hit that is stopped.* Make the demolition warhead psychic-damaging and the
  target immune to psychic weapons, but use a different, non-psychic warhead for the initial
  delayed kill. *Expected:* at and after expiry the building stays at 1 health, with no death
  effects. In another run make the demolition warhead itself cause delayed kills: it stays
  at 1 health, and its loss count and destruction events recur on later eligible turns.
  *Falsified if* it is removed unconditionally at expiry. These tests must end before their
  repeated events or draws overwhelm the recorder. A final variant uses a bridge repair hut
  that is not marked immune, made eligible for delayed kills. *Expected:* no expiry hit or combat
  death, and the countdown is cleared at expiry; bridge demolition takes its place (S-0511
  claim 16). Use an intact bridge linked to the hut and record the affected bridge cells as well
  as hut health and pending countdown. *Falsified if* it takes the ordinary building's lethal
  expiry hit, repeats the countdown or repairs the bridge instead of demolishing it.
  (h) *Protection.* Apply temporary invulnerability during the countdown, then let that effect
  finish before the former expiry tick. *Expected:* no expiry hit at that tick or later.
  Separately use an electromagnetic-pulse effect spanning expiry: the expiry hit occurs in the
  first unblocked building turn, without restarting the countdown. *Falsified if* invulnerability
  merely postpones expiry or an electromagnetic pulse cancels it. Record ticks, health and
  animations as in (e).
  (i) *Actions in the expiry turn (Q-0170).* Use (e)'s tiny map, game speed and
  countdown, with a human-owned defensive test building and an enemy test target in range.
  Keep the bystander outside the observed area under the common void rule. Give the building
  no death weapon and make the demolition warhead lethal without delayed kills or immunities.
  In separate runs align expiry with an ordinary shot due from its attack behaviour, and with
  the due turn of an animation-delayed shot (S-0301 F12). Use a slow projectile that cannot
  detonate until a later tick. For each run also run a control whose countdown ends one tick
  later, to establish that the shot really is due and its normal checks pass. Record tick-stamped
  projectile launches, the expiry hit, target health and attacker credit through the event and
  read-only state recorder of (e). *Expected:* the expiry-turn shot launches before the killing
  expiry hit in both variants; it later damages the target, but credits no attacker or attacking
  house. The control's launch occurs on the same tick. Separately give the test building
  self-healing and align expiry with a healing tick under S-0314 claim 9; then repeat with
  paid repair due under S-0314 claim 11. Record the health gain or repair payment and the
  expiry hit's delivered amount. *Expected:* the due healing or repair happens, and the expiry
  amount equals the raised current health, so it still kills. The paid repair is not skipped
  merely because expiry is due. Use (f)'s sale-removal control to confirm that a sale completing
  earlier in the turn leaves no expiry hit. *Falsified if:* the matched control fires but the
  expiry-turn run does not, a launched projectile is cancelled by expiry or retains attacker
  credit after it, the due healing or repair is skipped, expiry uses the pre-healing health,
  or an expiry hit follows the completed removal. Repeat each alignment over three recorded seeds.
  These timing outcomes must not depend on the seed.
  (j) *Healing and nonlethal damage during the countdown.* Use (e)'s one-cell building,
  strength 100, ordinary demolition warhead and duration 300. After the initial lethal hit
  at tick *t* leaves health 1, heal for 20 at *t* + 50, then hit for 5 with a delayed-kill
  warhead at *t* + 100 whose direct-hit duration is 10. Record health, attacker experience,
  both houses' counts and the expiry tick. *Expected:* health rises to 21 and falls to 16,
  and neither follow-up registers a kill or changes the countdown; expiry still occurs in the
  building's turn at *t* + 300 and delivers 16. *Falsified if* either follow-up adds credit,
  damage starts a shorter countdown ending at *t* + 110, healing cancels expiry, or expiry's
  delivered amount differs from 16. Void a run with any other hit, healing, repair or protection.
- **D14 (claim 17: positive surface-relative height).** A tiny test map with flat ground and
  a high bridge gives a stock flying infantry type (a jumpjet type) strength 1000, and a test
  warhead of infantry-death kind 9 with versus 250 % against its class and damage 50.
  Use a direct-hit projectile that can reach the target in flight, with no spread, and record
  target health, world height, surface height and hit tick with the read-only state recorder.
  (a) Hit while flying well above the surface: loss 2 (trunc(1 × 2.5)); standing at height 0:
  loss 125. With versus 100 %: loss 1 above the surface.
  (b) Order takeoff or landing and time the same direct hit during the movement, retaining
  only hits whose recorded height above the surface is between 1 and 207 leptons inclusive.
  *Expected:* loss 2 at versus 250 %, not the standing loss 125. This tests the strictly
  positive height rule near the surface rather than relying only on established flight.
  (c) A non-jumpjet infantry test type of the same armour class, strength 1000, stands on
  the high bridge deck; an identical unit stands under the bridge, with no prone state or
  armour bonuses. Hit each directly once with the damage-50, versus-250 % weapon. The deck
  unit has positive height above the terrain but height 0 above its surface.
  *Expected:* loss 125 for both, not 2 for the deck unit. Repeat every part three times at
  game speed index 2 under the common measurement setup above. Void a run
  with another hit, an unintended prone state, or movement away from the required height
  at impact. *Falsified if* any named loss differs; in particular, a loss of 125 in a valid
  near-surface run rejects the positive-height test, and a loss of 2 for the standing deck
  unit rejects the surface-relative baseline.

- **D15 (claim 4: no stacking).** A test rules file leaves only a firepower crate and an armour
  crate possible (firepower factor 1.5, armour factor 2.0; the keys are in Import mapping), with
  crates placed on the test map in a row. A test unit with a weapon of damage 10 (versus 100 %)
  picks up the firepower crate, fires at a test vehicle (strength 1000) and then picks up a second
  firepower crate and fires again. *Expected:* losses 15 and 15. The same with the armour crate on
  a test vehicle hit by damage 40: losses 20 and 20. *Falsified if* the second loss is 22 (or 23)
  or 10 (stacking). (c) *Frozen at the shot (claim 2).* The unit with the weapon of damage 10
  (versus 100 %, no crate yet) fires a slow test projectile (a flight of at least 100 ticks) at the
  test vehicle and is then ordered onto a firepower crate that it reaches while the projectile
  flies. *Expected:* the loss is 10 (the shot damage is fixed at the shot); a shot fired after
  the pickup loses 15. *Falsified if* the first shot loses 15.
  (d) *Promotion during flight.* Use a rookie trainable firer with the veteran firepower
  bonus granted at the veteran rank and veteran combat factor 1.5. It fires the same slow
  damage-10 projectile at a durable enemy target, then kills a second, separate enemy test
  vehicle with a fast secondary weapon before that projectile lands. Give the two targets
  different armour classes and configure versus values so that the firing checks permit only
  the primary against the durable target and only the secondary against the promotion target
  (S-0304 and S-0308). Set the second vehicle's worth so that this kill promotes the firer but does not reach the elite rank (S-0311).
  Record the launch, promotion, impact and target health; void a run if promotion is not
  strictly between launch and impact or another hit reaches the durable target.
  *Expected:* the in-flight shot still deals 10, and a fresh shot after promotion deals 15.
  *Falsified if* the in-flight shot deals 15 or the post-promotion control does not deal 15.
- **D16 (claim 8, stage 1 rows).** Test types, each hit by D1's weapon at 100 % unless said
  otherwise. (a) A test building whose weapon, aimed at a unit next to it, has a spread that covers its own foundation, once with
  its self-damage flag on and once off: it loses health only with the flag on. (b) A test building
  that is a laser-fence section: no loss; the trigger kill of S-0011 (claim 27, which ignores
  defences) destroys it. (c) A test vehicle inside the vehicle factory it has just left the
  production of, still in contact: no loss; after it has left the factory, one hit: it loses
  health; the trigger kill destroys it in both cases. (d) A bridge repair hut type marked immune:
  no loss from the weapon. (e) A test unit being locked on by a test erasing weapon: no loss from
  the weapon while the lock holds; after it lets go: loss. *Falsified if* any of these differs.
  (d2) *Run last, alone in its run, because it ends the game:* the trigger kill of S-0011 (claim
  27) on the immune bridge repair hut of (d), with a second tag-carrying test vehicle that the same
  trigger would act on after it. *Expected:* the hut is not destroyed and the original hangs from
  that tick on (the tick counter stops, the vehicle is never destroyed), as S-0011's scenario T9,
  variant C, expects for a survivor. *Falsified if* the hut is destroyed or the game goes on (then
  the predicted survivor or hang is wrong; continuing ticks alone would not disprove the
  immune-hut protection if the hut survives).
  (d3) *Delayed-kill survivor, run separately:* use a tagged test building eligible for delayed
  kills and a demolition warhead that causes them, with a positive delay. Fire the trigger kill.
  *Expected:* the building remains at 1 health and the tick counter stops at that firing.
  *Falsified if* it dies at once or ticks continue. (d4) *Positive-height infantry survivor, run separately:*
  tag a flying infantry unit and give the demolition warhead infantry-death kind 9. Fire the
  trigger kill while its recorded height above the surface is positive. *Expected:* its health
  is unchanged and ticks stop at that firing. *Falsified if* its health falls or ticks continue. In d3 and d4 record health
  and ticks with the read-only recorder, and keep a second tagged target for the never-reached
  continuation as in d2. These tests deliberately hang the original and must each run alone.
- **D17 (claim 8 stages, claim 10).** (a) A test warhead that makes targets go berserk (damage 300,
  versus 100 %) hits a test vehicle of another house. *Observable:* the tick at which the target
  stops firing at units of its own side, and each countdown update read-only. *Expected:* expiry on
  the 300th countdown update after application, health unchanged. A second hit of damage 50 at tick
  100 resets the remaining count to 50, rather than adding time; expiry is on the 50th countdown
  update after reapplication. Count an update in the hit tick when it follows the hit (S-0610 claim
  5). (b) The same warhead, fired by a unit that is not killed until the shot lands (force-fired by
  scripted input where the target is not otherwise legal), against a target of an allied house and
  against a target of the firing unit's own house, against a target whose type is immune to
  psionics, and against a test building: no berserk state and no health change. (c) The same
  warhead, flagged also as a radiation warhead, against a target immune to radiation: no berserk
  state; against a target not immune: berserk. The same with the psychic-damage flag against a
  target immune to psychic weapons, and with the poison flag against a target immune to poison.
  Plain hits (no berserk flag) of each of the three warheads on the immune target do no damage, and
  on a target that is not immune do damage. (d) The berserk warhead against a temporarily
  invulnerable target: no berserk state. (e) The berserk warhead against a target whose type is
  marked immune (to all damage): the target goes berserk. (f) The radiation-immune target of (c) and
  the invulnerable target of (d) are of the test aircraft type of D10 (ammunition 10, readiness
  reduction factor 1.0, strength 100): the ammunition falls in (c), and does not fall in (d). (g)
  The warhead of (b) is fired with a slow test projectile (a flight of at least 100 ticks) at a
  target of an allied house, and a third unit kills the firing unit while the projectile flies; the
  firing unit is a plain test vehicle that neither sinks nor crashes and has no death animation, so
  that it is gone at once (S-0203, claim 6a). *Expected:* the allied target goes berserk for about
  300 ticks (the shot carries no attacking house, claim 1); run with the firing unit left alive, the
  same shot causes no berserk state. (g2) As (g), but the firing unit is an infantry unit of the
  human house whose death sequence outlasts the flight, killed by the third unit: it is killed
  though it stays in play, so the shot carries no attacking house, and the allied target goes
  berserk. (g3) As (g), but instead of being killed the firing unit (infantry) is ordered into a
  closed test transport right after the shot and has entered it before the shot lands: it is out of
  play but not killed, so (if loading a passenger does not stop the shot from carrying it as its
  attacker, as claim 1 believes) the shot carries it and its house, and the allied target does not
  go berserk; if the allied target goes berserk, loading stops the shot from carrying its firer, and
  claims 1, 10 and 14 are to say so. The claimed continued carrying is falsified by that berserk
  state. (g4) *The death weapon's house.* A test vehicle *F* of the human house has a death weapon
  (its explodes flag on) whose warhead is the berserk warhead of (a) with a spread of 2 cells. A
  second vehicle *G* of the human house and a vehicle *E* of the computer house stand inside the
  spread, a cell or more from *F* and well away from each other. A computer unit whose warhead has
  no spread destroys *F* and nothing else. *Expected:* *E* goes berserk for about 300 ticks and *G*
  does not (the detonation carries *F* and *F*'s house, claim 1, so the berserk row's own-side
  exception applies to *G*). (g5) The same setup with a death weapon whose warhead has the
  affects-allies field off, no berserk effect, damage high enough to be noticed and versus 100 %:
  *E* loses health and *G* loses none (the allies row of claim 8 needs an attacker, which the
  detonation carries). (h) The amount that decides the duration: the warhead of (a) (damage 300) on
  a target whose country has the armour factor 2.0 for its kind, once with versus 100 % and once
  with versus 50 %. *Expected:* about 150 ticks (the defended amount trunc(300 / 2), not 300) and
  about 75 ticks (versus applied to the defended amount: trunc(150 × 0.5)). (i) Named test map
  D17-zero-duration gives the berserk warhead a zero versus value for the susceptible vehicle's
  armour class; repeat with a negative versus value and positive shot damage. Keep other protections
  off. Start the human-owned vehicle attacking a durable enemy, apply one hit and record target,
  queued behaviour, berserk state and countdown updates read-only, including between events in the
  hit tick. Expected: unchanged health, target dropped and *hunt* queued at entry, then berserk
  cleared, time set to 0, target dropped and *guard* queued at the first countdown update. Repeat
  with the hit before and after the vehicle's update to distinguish same-tick from later-tick
  expiry. No entry effects or expiry at a different update falsifies this part (S-0610 claims 4 and
  5). *Falsified if* any of these differs, in particular a berserk state in (c) with an immune
  target or in (d), none in (e), none in (g) or (g2) with the firer killed, one in (g) with the
  firer alive, or berserk in *G* in (g4) or damage to *G* in (g5), or none in *E* in either; (g3) is
  the open case of claim 1.
- **D18 (claim 13: building fires, a statistical scenario with a recorder).** 200 test buildings
  of the 2 × 2 foundation word (strength 100) stand far enough apart that their fires do not overlap, on
  a test map with three test fire animations as the damage-fire list. A test warhead with its
  sparks flag set (damage 60, versus 100 %) hits each once (health 100 to 40, crossing the
  yellow threshold, and also the red one at the default red ratio, which gives the same result).
  The recorder reads, for each building, the random stream's state before and after the hit, and
  the fire started on each cell and its loop count. *Observable 1 (exact):* replaying the
  recorded stream state with S-0202's algorithm, cells in row order (rows by increasing map *y*,
  within a row by increasing map *x*: for the 2 × 2 foundation the top-left cell, the cell next to
  it along map *x*, the cell next to the top-left cell along map *y*, and the last) with each loop
  draw directly after its own cell's draw, must reproduce every fire and loop count, and the stream state after
  the hit, for all 200 buildings. A replay with all cell draws first and the loop draws after
  them, or with the cells in column order (map *x* first, then *y*), must fail for at least one building (the scenario
  checks that these alternatives do differ, which they do with high probability over 200 buildings).
  *Observable 2 (statistical):* over the 800 cell draws the counts of no fire, first, second and
  third animation, expected 80, 400, 240 and 80 (probabilities 0.1, 0.5, 0.3, 0.1), pass a
  chi-squared test with 3 degrees of freedom at the 0.001 level (reject above 16.3); the loop
  counts 1, 2 and 3 of the fires of the first and second kind (about 640), each expected to be
  a third of the number of those fires, pass a
  chi-squared test with 2 degrees of freedom at the 0.001 level (reject above 13.8). A second
  run with a warhead whose sparks flag is off must start no fire and take no draw (the stream
  state is unchanged by the hit). A third run uses the 8-cell 3 × 3 foundation word of claim 13 and checks, by the same replay,
  that 8 cell draws (not 9) happen, that the cell without a draw is the one 2 cells along map
  *x* and 1 cell along map *y* from the top-left cell (the recorder sees no fire there in any building,
  and the replay fits only with that cell skipped), and that the draw range is 0 … 11. A fourth run
  (if the test map can place a building of the zero-size foundation) hits such a building with the
  sparks warhead and must show no draw. *Falsified if* a replay differs, a count fails its test, or
  the stream state shows another number of draws.

- **D19 (claim 3: difficulty rows and unused armour factors).** The test map gives each of the
  three difficulty sections completely, because a map that changes one key of a difficulty section
  resets the section's other keys (S-0006, claim 5); every factor is 1.0 unless stated, as are the
  country factors.
  (a) *Normal row, firepower.* The firepower factors are 2.0 (easy row), 1.5 (normal row) and 0.5
  (difficult row), and the human house's country has the firepower factor 1.5 (all exact in binary).
  A human test unit with a test weapon of damage 5 (versus 100 %) attacks a computer vehicle (strength
  1000). *Expected:* loss 11 per hit (trunc(5 × 2.25) = trunc(11.25)). *Falsified if* the loss is 7
  (one factor missing), 10 (the two truncations of 5 × 1.5 and 7 × 1.5), 12 (rounding), 15 or 3
  (another row used).
  (b) *Unused armour factors.* The armour factor is 4.0 in all three rows, and the human house's
  country has the general armour factor 4.0 (the keys of Import mapping that no damage rule uses).
  A computer test unit with a weapon of damage 40 (versus 100 %) attacks a human vehicle (strength
  1000); every firepower factor is 1.0 in this run. *Expected:* loss 40. *Falsified if* the loss is
  10 or 20 or any other value.
  (c) *Rows of computer houses.* The firepower factors are 2.0, 1.0 and 0.5 for the easy, normal
  and difficult rows, nothing else set. One computer house with a test unit of damage 8 (versus
  100 %) attacks a human vehicle in three runs with the lobby entry easy computer, normal computer
  and hard computer (one person in the game). *Expected:* losses 4 (the easy computer has the
  difficult row), 8 and 16 (the hard computer has the easy row). *Falsified if* a loss differs, in
  particular 16, 8 and 4 (the inverse pairing), or if two settings give the same loss.
  (d) *The non-player houses.* S-0412's scenario K5 part d measures it: a weapon of the neutral
  house's defence with the rows 2.0, 1.5 and 0.5 and a country firepower factor of 1.5 loses 8 per
  hit, the factor being exactly 1.0. *Falsified if* the loss is 12, 18, 16, 4, 24 or 6.
- **D20 (claim 2: garrison, bunker and open-topped multipliers, sign rule).** The combat-damage
  section of the test map gives the garrison multiplier 0.5, the bunker multiplier 0.75 and the
  open-topped multiplier 1.5 (exact in binary). The test weapon has damage 37, a second test weapon
  damage −20, both with versus 100 % against the targets' class and aimed at the target directly
  (the legality of the healing weapon as in D4). Shooters fire at an enemy test vehicle (strength
  1000) for the damaging weapon, and at a damaged own vehicle (health 100) for the healing one.
  (a) A test infantry unit inside a stock garrisonable building type, whose occupant weapon is the
  test weapon: loss 18 (trunc(18.5)); with the healing weapon: +10. (b) A test vehicle inside a
  stock tank bunker: loss 27 (trunc(27.75)); healing +15. (c) A test infantry unit firing from
  inside a stock open-topped transport: loss 55 (trunc(55.5)); healing +30. (d) Controls: the same
  weapons fired without the garrison, the bunker and the transport: loss 37 and +20. *Falsified if*
  any value differs, in particular 19, 28 or 56 (rounding), or a healing equal to the control's
  (the sign rule is not applied).
- **D21 (claim 5: the kinds of target).** The test map gives the computer house's country the armour
  factors 0.5 for infantry, 0.25 for vehicles, 2.0 for aircraft, 4.0 for defence buildings and 0.125
  for other buildings. A weapon of damage 8 (versus 100 % against every test class) hits a test
  infantry type, vehicle type and aircraft type (each of strength 1000), a test building whose build
  category is combat, and a test building whose build category is not (also strength 1000). *Expected:*
  losses 16, 32, 4, 2 and 64. *Falsified if* any differs, in particular if the two buildings lose
  the same.
  (b) *Category input (Q-0061 part 2).* Repeat the two-building test with a base layer giving
  the combat category and successive map variants giving the mixed-case combat word, a word
  with an extra letter, a wholly unknown word, no key and an empty key. *Expected:* losses
  2, 64, 64, 2 and 2 respectively. With no category in any layer, loss 64. For the length limit,
  use the combat word followed by 25 spaces and then an extra letter: the first 31 bytes end
  in spaces, which trim away, and the result is combat (loss 2). Use only 24 spaces before
  that letter: it remains among the first 31 bytes, so the whole word is unknown (loss 64).
  *Falsified if* case changes the result, an extra letter is accepted, an unknown later word
  preserves combat, absence resets combat, or the two length-limit runs give equal losses.
- **D22 (claim 13: the effect draw and the list, statistical with a recorder).** A test vehicle type
  of strength 1000 lists four damage particle systems, in this order: A (smoke), B (gas), C (smoke),
  D (spark), all test types that the recorder tells apart. A hundred such vehicles (no two effects
  overlapping) are each hit once for 600 (1000 to 400, crossing half the strength, with the health
  ratio below the yellow ratio at its default). The recorder reads the random stream's state before
  and after each hit and the effect started. *Observable 1 (exact):* replaying the stream with
  S-0202's algorithm, one ranged draw over 0 … 1 per vehicle must reproduce the effect of every
  vehicle (a draw of 0 gives C, the last eligible entry; a draw of 1 gives A) and the state after the
  hit, for all 100. *Observable 2 (statistical):* the counts of A and C, each expected 50, pass a
  chi-squared test with 1 degree of freedom at the 0.001 level (reject above 10.8). A second run
  with a list of B and D only starts no effect and takes no draw. *Falsified if* the replay differs
  for any vehicle (in particular if B or D is ever chosen, or if a draw of 0 gives A), a count fails
  its test, or the stream shows another number of draws.
- **D23 (claim 12: loss of the disguise).** A spy type of the human house, with the permanent
  disguise flag cleared and strength set to 1000 in the test file, is disguised as an enemy
  infantry type and stands among computer-house units. All other disguise settings remain stock, and
  the recorder reads whether it shows as the disguise to the computer house (S-0516 claim 34). (a) A hit of damage 5
  (versus 100 %): the disguise is lost (the spy shows as itself) until the human house disguises it
  again. (b) A hit that reports *unaffected* (the same weapon with versus 0 %): the disguise is
  kept. (c) A healing hit: the disguise is kept. (d) A hit that crosses half the strength (*now
  yellow*) loses the disguise as in (a). (e) A stock type that keeps its disguise permanently, hit
  as in (a): the disguise is kept. (f) A berserk hit (the warhead of D17 (a), the test map making
  the spy type not immune to psionics) on the spy of (a), which makes it go berserk: the disguise
  is kept (claim 10). (g) Start a fresh disguised spy
  at health 300, strength 1000 and red ratio 0.25, and hit for 100: the crossing from 300 to
  200 reports *now red* and loses the disguise as in (a). (h) For the countdown, use a still,
  non-permanently-disguised disguise-when-still vehicle, with no contact or neighbouring enemy
  infantry, as in S-0516 claim 33 and scenario CS14 part c. A hit taking 5 health clears its
  disguise and sets a 10-tick re-disguise lockout; record the hit tick, remaining lockout and
  tick it takes a disguise again. It cannot re-disguise while the lockout remains and does so
  on its first eligible turn after expiry. *Falsified if* any of these differs.
- **D24 (claims 1, 14: credit for a kill).** A trainable test unit *A* of the human house (known
  experience, S-0311) and test computer vehicles of strength 1; the recorder reads the units-destroyed
  count of each house as the score screen shows it. (a) *A* fires a test weapon at a vehicle, which
  dies: *A*'s experience rises by the vehicle's worth at the tick of the hit and the human house's
  count rises by 1. (b) *A* fires a slow test projectile (a flight of at least 100 ticks) at a
  vehicle, and a computer unit kills *A* while the projectile flies (*A* is a plain test vehicle
  that neither sinks nor crashes and has no death animation, so it is gone at once): the target
  vehicle dies at impact and no experience or count of the human house changes. (b2) As (b), with
  *A* an infantry unit whose death sequence outlasts the flight: *A* is killed but stays in
  play, so again no experience or count changes. (b3) As (b), but *A* (infantry) is ordered into a
  closed test transport after the shot and enters it before the shot lands, and nothing kills
  it: the human house's count rises by 1 and *A*'s experience rises if the shot still carries *A* as its attacker, as claim
  1 believes (low). (b5) As (b), with *A* a test vehicle of a type that can cloak, which starts to cloak by
  itself after its shot (the test map makes the delay before cloaking shorter than the
  flight) and is not killed: *expected* (believed, low): no experience or count of the human
  house changes, as in (b). (b4) As (b), with *A* a building that fires before the hit that starts its
  delayed-kill countdown (claim 16) and whose projectile lands during the countdown: no change of
  the human house's count (the building's experience is not read). (c) The trigger kill of S-0011 (claim 27)
  destroys a vehicle: no experience and no count changes. (d) The human house owns the stock
  lightning-storm superweapon, and thirty vehicles of strength 1 of the computer house stand in its
  area, the bolt's damage set high in the test map: the human house's count rises by the number of
  vehicles the bolts destroy, and no experience changes anywhere. (e) *A* is a test vehicle with
  a death weapon (explodes flag) and strength 1 of the human house, killed by a computer unit
  next to a test computer vehicle of strength 1 that stands inside the death weapon's spread
  (versus 100 %): the computer vehicle dies from the death weapon, the human house's count rises
  by 1 (the dying object's house is credited); the experience that the dying object itself gains
  is not observable, since a destroyed vehicle leaves play at once (S-0203, claim 6a). (f) *A sonic wave's kill.* *A* is a unit of the stock type whose weapon delivers its damage
  through a sonic wave (the test map sets its warhead's versus value to 100 % for the target's
  class); the wave passes through a test computer vehicle of strength 1: the vehicle dies, *A*'s
  experience rises at that tick and the human house's count rises by 1 (the wave carries its firing
  object, claim 1). (d2) *The storm bolt does not ignore defences.* As (d), with the bolt's warhead
  given the versus value 0 % for the vehicles' class: none of the vehicles is destroyed (a hit that
  ignored defences would destroy them). (d3) *The storm bolt carries no no-escape marker.* As (d),
  with the vehicles of a type with the crewed flag and the game-wide crew escape chance set to 1.0
  in the test map (S-0312, claim 13): each destroyed vehicle leaves a crew unit. *Falsified if* any
  of these differs, in particular a credit in
  (b), (b2), (b4) or (c), none in (d), (e) or (f), a destroyed vehicle in (d2), or none leaving a
  crew in (d3). In (b3), failure of the human house's count to rise by 1 or of the living
  firer's experience to rise falsifies continued carrying. In (b5), any such rise falsifies
  loss of the source on starting to cloak. These two expectations remain low-confidence
  hypotheses of claim 1.

- **D25 (claims 5, 6, 8: negative versus value, healing and armour).** (a) A test weapon of damage
  40 whose warhead has the versus value −50 % against a test vehicle class (legal as in D4; forced
  by scripted input where needed) is aimed at a damaged own vehicle (health 100 of strength 1000).
  *Expected:* +20 per hit (trunc(40 × −0.5) = −20). With the test map's maximum damage set to 10, as
  in D3: still +20 (the cap does not limit it). (b) The same weapon against a temporarily
  invulnerable damaged vehicle: no change (the hit counts as a + hit, claim 8). (c) The healing
  weapon of D4 (damage −20, versus 100 %) against an own vehicle whose country has the vehicle
  armour factor 4.0: +20 per hit. *Falsified if* (a) gives +10 or 0, or caps at +10, (b) heals,
  or (c) gives +5.
- **D26 (claim 8: the − column).** The healing weapon of D4 (damage −20, versus 100 % against the
  targets' class; legality as in D4, forced by scripted input where needed) is aimed at damaged
  own targets (health 100 of strength 1000, or of a building's own strength), each in the situation
  of one row. (a) *Expected: no heal* for a laser-fence section building, a vehicle standing in
  contact inside the vehicle factory it has just left the production of, a unit being erased by a
  test erasing weapon, a bridge repair hut marked immune, and a vehicle type marked immune. (b) A
  healing warhead that does not affect allies, aimed at an own vehicle: no heal; the same warhead
  without that flag: +20. (c) A healing warhead with the radiation flag, aimed at an own vehicle
  of a type immune to radiation: no heal; at one that is not immune: +20; the same with the
  psychic-damage flag and a type immune to psychic weapons, and with the poison flag and a type
  immune to poison. (d) A tank bunker holding a vehicle: a healing warhead that penetrates bunkers
  aimed at the bunker heals nothing; one that does not penetrate bunkers, aimed at the bunkered
  vehicle, heals nothing. (e) The entries marked "no" in the − column: a unit of a type marked immune
  to its own type, healed by another unit of that type of the same owner: +20; the invulnerable
  vehicle of D6: healed (as D6 already says). *Falsified if* any heal happens in (a) to (d)
  where none is expected, or none in the expected cases of (b), (c) and (e).
- **D27 (claim 13: marking of the damage effect).** D11 part a's test map (yellow ratio 0.75, red ratio
  0.25) and a stock vehicle type whose list has exactly one smoke particle system (so the draw's
  outcome is fixed; strength set to 1000). Hits of 100 take it to 400 (the effect appears, D11).
  A healing weapon of damage −100 (versus 100 %) then raises it to 700 in three hits: the effect is
  still there. A fourth healing hit takes it to 800 (ratio 0.8, above the yellow ratio): the effect is marked at that hit,
  so it emits no more particles and no new effect starts; the system leaves play only once it holds no
  particle, which the recorder reads over the following ticks. *Falsified if* the effect is marked at 700,
  or still emits after the hit to 800, or a new effect appears.

- **D28 (claim 20: terrain objects).** A test map with test terrain types of strength 100 and
  the armour class *wood*, standing in open cells, and a test vehicle of strength 1000 next to
  each. (a) A test weapon of damage 37, versus 100 % against *wood*, whose warhead has the wood
  flag off, hits a tree directly: no health change. With the flag on: loss 37; with versus 50 %:
  loss 18. A weapon of damage −20 (versus 100 %, its warhead with the wood flag on) aimed at the
  damaged tree heals it by 20 (legality of the order as in D4). (b) The same tree type marked
  immune: no loss from the weapon with the wood flag on. (c) A tree killed by the weapon of (a)
  that is fired by a trainable test unit of the human house: no change of the unit's experience
  and of the house's counts. (d) An ore-spawning test terrain type (the spawns-ore flag) killed by
  the weapon: the test vehicle next to it loses exactly 100 health, once (the test map sets the
  demolition warhead's versus value for the vehicle's class to 100 %, its edge percentage to
  100 % and its spread to 2 cells, the tree's own cell is the detonation's centre, and the
  vehicle stands in a neighbouring cell at least 100 leptons inside the spread's radius of 512
  leptons, so no boundary is tested); the same
  vehicle next to a tree type without the flag loses nothing. With the demolition warhead's
  versus value for the vehicle's class set to 50 % instead, the vehicle loses exactly 50 (the burst
  does not ignore defences). (e) A sonic-wave weapon of stock
  type fires through a tree (its warhead with the wood flag on, versus 100 % for *wood*): the
  tree loses health. *Falsified if* any of these differs, in particular a loss with the wood flag
  off in (a) or from the immune tree in (b), any credit in (c), no damage to the vehicle in (d), or
  a loss of 100 in (d) with the versus value of 50 %.
  (f) *Ore-burst crew escape and absent credit (claim 1).* Repeat (d) with one computer-house
  vehicle of strength 50, crewed, with no passenger capacity, no hijacker and no death weapon,
  standing in the neighbouring cell. Set the game-wide crew escape chance to 1.0, provide a
  valid crew type of strength at least 20, and leave room for infantry at the vehicle's cell.
  Give the demolition warhead versus 100 %, edge percentage 100 % and spread 2 cells, with
  no damaging secondary effects. A trainable human unit destroys the ore-spawning terrain
  object using a wood-damaging weapon with no spread, so its own shot cannot hit the vehicle.
  Record the tree's death, burst hit, vehicle death, crew creation, the firer's experience and
  every house's kill count. *Expected:* the burst kills the vehicle, a crew unit is created
  at its position (then scatters under S-0312 claim 13), and no object or house gains kill
  credit. Thus the burst does not inherit the tree-killing unit or its house, and it does
  not suppress crew escape. As a placement control, destroy an identical vehicle at the same
  position by ordinary direct fire in a separate run: a crew must appear and the firing unit
  and its house receive the normal credit. Repeat each run three times; void it if another
  hit reaches the vehicle or crew, placement is blocked, or the control fails to produce a crew.
  *Falsified if* a burst-killed vehicle produces no crew in a valid run, the tree-killing
  firer gains experience, or any house's kill count rises from the burst kill. The read-only
  event and state recorder is the tool; observe creation at the killing hit, not merely
  whether a crew survives later ticks.

- **D29 (claims 2 and 5: veteran factors, and the sign rule of the firing factors).** The test map's
  general section gives the veteran combat factor 1.5 and the veteran armour factor 2.0, and the
  human house's country the firepower factor 1.25 (all exact in binary; Import mapping); every other
  factor is 1.0 unless stated. The test types' veterancy abilities grant the veteran firepower bonus
  (the firer's type) or the veteran armour bonus (the target's type) at the veteran rank (S-0311
  owns the list and its form). Each veteran object is brought to the veteran rank as S-0311 gives
  (for example by its starting experience), and a rookie of the same type is its control.
  (a) A veteran human test unit with a test weapon of damage 7 (versus 100 %) attacks a computer
  vehicle (strength 1000) without the armour bonus. *Expected:* loss 12 per hit
  (trunc(trunc(7 × 1.25) × 1.5) = trunc(8 × 1.5)); the rookie control loses 8. *Falsified if* the
  veteran's loss is 13 (one truncation of 7 × 1.875 = 13.125), 10 (house factor missing) or 8
  (veteran factor missing).
  (b) A veteran and a rookie human unit of a second test type, with the same veterancy abilities
  and the healing weapon of D4 (damage −20, versus 100 %; legality as in D4), each fire at a
  damaged own vehicle (health 100 of strength 1000). *Expected:* +20
  per hit from both. *Falsified if* a heal is 25 (the house factor applied to a negative damage), or
  30 or 37 (the veteran combat factor applied).
  (c) A rookie human test unit with a weapon of damage 8 (versus 100 %) hits a computer test vehicle
  of strength 1000 at the veteran rank whose type has the veteran armour bonus: shot damage
  trunc(8 × 1.25) = 10, *expected* loss 5 per hit (trunc(10 / 2.0)); the rookie target loses 10.
  A second test map separates the two truncations of F2: the veteran armour factor 0.625, the
  computer country's vehicle armour factor 0.625 and the human country's firepower factor 1.0 (a
  factor below 1 only to separate them), damage 6. *Expected:* loss 14 (trunc(6 / 0.625) = 9,
  trunc(9 / 0.625) = 14); the rookie target loses 9. *Falsified if* the veteran target loses 10 in
  the first map, or 15 (one truncation of 6 / 0.390625 = 15.36) or 9 in the second.

- **D30 (claim 18: expected damage).** Reads the strong-mode pick of S-0309's scenario A8, part b:
  a firer of target-health scan mode *strong* does not pick a target whose expected remaining
  health is 0 or less. Two human infantry firers of one test type (object armour multiplier 1.0, a
  slow test projectile with a flight of at least 100 ticks, damage 8, versus 100 % against the
  target's class) stand in range of one computer test vehicle of strength 10. The test map gives
  the computer house's country the armour factors 0.5 for vehicles and 1.0 for infantry (exact in
  binary). The first firer fires; the recorder reads whether the second firer picks the vehicle
  while the first shot is in flight. (a) *Firer's kind.* The real hit is trunc(8 / 0.5) = 16 and
  kills the vehicle, but the expected damage divides by the factor for the firer's kind (infantry,
  1.0): 8, so the expected remaining health is 2. *Expected:* the second firer picks the vehicle at
  once. With `quirks.expected_damage_firer_armour` off the expected damage is 16 and the second
  firer does not pick it (the switch's alternative, not a falsifier). (b) *Prone factor.* The
  target is a prone infantry unit of strength 6 whose country's infantry factor is 1.0, and the
  warhead's prone factor is 50 %: the real hit is 4. *Expected:* the second firer does not pick it
  (expected damage 8 without the prone factor, remaining health 0 or less); with the prone factor
  it would be 4 and the second firer would pick it. *Falsified if* the second firer does not pick
  the vehicle in (a), or picks the prone unit in (b).
  (c) *Firer's object armour.* Repeat (a), but the first infantry firer picks up a named test
  armour crate with factor 0.5 before firing; the target receives no crate and all other
  settings stay as in (a). F8 now gives expected damage trunc(8 / 0.5) = 16, leaving expected
  remaining health at or below 0. *Expected:* the second firer does not pick the vehicle
  before impact; the no-crate control of (a) does pick it. *Falsified if* the second firer picks
  it in both runs or neither. Record the pickup, shot and second firer's target with the
  read-only recorder; void a run with any other hit before that target choice.

- **D31 (claim 1: parasite-delivery metadata).** Use S-0614 P1's drone setup and P6's
  grappling setup, with named test-map overrides as follows. Record attachment, delivery ticks,
  host health, crew creation, parasite experience, house kill counts and berserk state through
  video and the read-only recorder. Run each part three times under the common setup above.
  (a) Give a drone a latchable infantry host with temporary invulnerability and a zero versus
  value for the delivery warhead. Expected: its first delivery kills the host despite both
  protections. With an otherwise identical non-infantry host, invulnerability stops the hit.
  Without invulnerability, a zero versus value stops it. This discriminates the ignore-defences
  exception from the ordinary delivery. (b) For both parasite kinds, use an enemy crewed vehicle
  host with no passengers, hijacker or death weapon, set crew escape chance to 1.0, give a valid
  crew type and leave placement room. Disable culling for the grappling run and let a delivery
  kill the host. Expected: no crew, and ordinary kill credit and experience for the parasite
  under claim 14 and S-0311 claims 7 to 10. In a separate control, ordinary direct fire killing
  an identical host at the same position must produce a crew. Void a run if that control fails
  or another source hits the host. (c) Give each parasite kind a berserk delivery warhead that
  affects allies, with positive damage and a positive versus value. Use a latchable host mutually
  allied with the parasite's owner and force an attack to attach it. Disable grappling culling.
  Expected: the delivery makes the host berserk under claim 10 despite their alliance, because
  no attacking house accompanies it; health is unchanged. A direct-fire control from the same
  house with that warhead does not make the allied host berserk, because its detonation carries
  an attacking house. Void the run if no attachment occurs. Any contrary result in a valid run
  falsifies the corresponding metadata claim.

- **D32 (claims 1 and 7: heal-all exceptions).** Use a tiny named test map with fixed heal-all
  crates, one collecting house and no incidental combat. Place two undemolishable buildings of
  that house, one damaged and one at full strength, both with more than 1 health.
  Include an ordinary damaged building, a damaged
  open-topped passenger and a damaged closed-transport passenger. Run at game speed index 2
  under the common measurement setup above, using the read-only recorder and scripted pickup input.
  Record health immediately before and after collecting the crate, and after unloading both
  passengers. Expected: each undemolishable building loses 1 health, the ordinary building and
  open-topped passenger reach strength, and the closed-transport passenger retains its earlier
  health. Any different health delta falsifies the corresponding rule. S-0520 CR10 supplies
  the ordinary heal-all and transport controls; CR12 tests the separate direct damaging hits.

- **D33 (claim 1: damaging-animation hit metadata).** Use a tiny named test map at speed 2,
  no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps
  excepted; see *measurement setup*), with one bystander computer player kept away from the test area
  under the glossary's void rule. A named animation-damage fixture gives an ordinary damaging animation
  a known owner, damage and game-wide warhead, with no secondary effects; compare an ownerless
  copy. Place isolated infantry and crewed vehicle targets within the area, with enough health
  for several advances. Use two armour/versus settings that give different predicted ordinary
  losses, and a separate lethal run with guaranteed crew escape and a legal free exit.
  *Procedure:* let each animation advance without other attacks. *Record:* health per advance,
  kills and losses credited to houses, and surviving crew, using the read-only recorder and
  frame capture. *Expected:* ordinary defended area losses in both ownership cases, no attacking
  house credited with kills, and crew escape on a lethal hit. The victim's house still records
  its loss. *Falsified if:* animation ownership supplies kill credit, damage bypasses the stated
  defences, or the lethal animation hit alone suppresses the guaranteed crew escape. S-0109
  claim 17 supplies the per-advance amount; S-0312 claim 13 supplies the escape control.
- **D34 (claims 6 to 8: a hit without a warhead).** Part a: as S-0317 scenario V5's setup (its
  fixed-path test type, damage amount and radius, thrown over a cell holding two infantry objects),
  with neither object prone, but the named test file gives the piece type no warhead, so it takes
  the absent-key default (S-0317 Import mapping). A control run uses V5's test warhead. *Record:*
  health and berserk state of both objects per tick with the read-only recorder, and the tick of
  the piece's ground impact. *Expected:* in the control run both objects lose health at the
  impact; without a warhead neither does, and neither goes berserk. *Falsified if:* either object
  loses health from the warheadless piece's impact or goes berserk after it. A
  control run in which either object loses no health at the impact is void. Part a shows only that
  no health is lost: it cannot tell a hit that takes nothing from no hit at all.
  Part b: as part a, with the piece thrown over the cell of a one-cell test building whose type
  is undemolishable (a named test file, Import mapping), so that the building's reference point,
  the centre of that cell, lies within the piece type's damage radius of the impact point (S-0317
  claim 7; the run is void if the recorded impact point is farther than that radius from the
  cell's centre), and a control run with V5's test warhead.
  *Expected:* in the control run the building loses health at the impact; without a warhead it
  loses exactly 1 health at the impact. *Falsified if:* without a warhead it loses none (no hit
  dealt) or more than 1; a control run in which it loses none is void. The run shows both that
  the piece deals the hit and claim 7's minimum for it.
  Part c: as part a, with the piece thrown over the cell of D10's test aircraft type, landed,
  instead of the infantry objects, so that the aircraft's position lies within the piece type's
  damage radius of the impact point (S-0317 claim 7; the run is void if the recorded impact
  point is in another cell or farther than that radius from the aircraft's recorded position),
  and a control run with V5's test warhead. *Record:* the aircraft's position, ammunition and
  health per tick, and the impact point. *Expected:* without a warhead the ammunition falls
  at the impact by Formulas F6 with the piece's damage amount after claim 5's defences, and the
  health does not change; in the control run the health falls at the impact. *Falsified if:* the
  ammunition does not change at the impact, or the health changes at it; a control run in which
  the health does not fall is void.
  Part d: as part b, with the named test file also marking the undemolishable building's type
  immune to radiation, to psychic weapons and to poison, but not immune to all damage, and no
  control run. *Expected:* the building loses exactly 1 health at the impact, as in part b, since
  none of the three immunity rows holds for a hit without a warhead. *Falsified if:* it loses
  none (an immunity row stopped the hit) or more than 1.
  Part e: as part b, with the piece thrown so that the impact point lies within the damage radius
  of the reference point of a test tank bunker type, a copy of a stock tank bunker marked
  undemolishable in the named test file, holding a test vehicle, and a control run with a test
  warhead that penetrates bunkers. *Record:* the bunker's health per tick. *Expected:* in the
  control run the bunker loses nothing (the tank-bunker row stops the hit); without a warhead it
  loses exactly 1 at the impact, since that row does not hold. *Falsified if:* without a warhead
  it loses none or more than 1; a control run in which it loses health is void.

## Changes for other specs

- **Glossary, *hit*.** "A *hit* is one delivery of an amount of damage, with a warhead, to one
  object." becomes "A *hit* is one delivery of an amount of damage, with a warhead or, for some
  sources (S-0302 claim 1), none, to one object." Reason: a debris piece of a type with no warhead
  still delivers its ground-impact hits (S-0317 claim 7), and claims 6 to 8 say what such a hit
  does.
- **S-0317 claim 7, the row for a type with no warhead.** Its table row "A type with no warhead |
  `unmeasured`: no scenario here isolates its hit consequences | medium, resting on S-0302"
  should read that the hit consequences are `measurable` through S-0302 scenario D34, still
  medium and resting on S-0302 until D34 runs.
- **S-0215 claim 16 and its open question on a berserk hit.** The trigger should also name a hit
  that makes its target berserk or renews that state and reports *unchanged* (claim 10 here), which
  lowers no health, and the open question can then be closed. When the hit makes the actor berserk,
  a behaviour that the fall-back queues replaces the *hunt* that the berserk entry queued (S-0610
  claim 4). Level and backing: those of claim 10 here for the trigger. The replacement of the queued
  *hunt* is medium and `unmeasured`: no scenario here makes an actor in a no-threat behaviour
  berserk.
- **S-0215 claim 39.** No rule change: its berserk sentences agree with claim 10 here.
- **S-0208 claim 15.** Request B is also sent after a berserk hit with a known attacker (claim 10
  here), which takes no health. Request A is not. Its "takes damage and survives it" should cover
  that hit. Level and backing: those of claim 10 here.
- **S-0107 claim 16.** Its table also applies to a berserk hit (claim 10 here), which takes no
  health, with the health ratio as it stands. Level and backing: those of claim 10 here.
- **S-0701 claim 28.** The announcement that a harvester is under attack is also made for a berserk
  hit on the local player's harvester (claim 10 here), which takes no health. Level and backing:
  those of claim 10 here.

## Open questions

- From which side the alliance tests are made is not settled. Alliance is one-way (glossary,
  *alliance*). Claim 8's allies row is worded from the attacker's owner ("the attacker's owner is
  allied with the target's owner"), and claim 10's berserk exception and its "own side or its
  allies" from the target's owner ("the target's owner is allied with" the attacking house).
  Which side the original tests in each place is to be confirmed. The two sides differ only after
  a one-way alliance change (S-0414, claims 12 and 13); scenarios D8 and D17 use the mutual
  alliances of the lobby only.
- Which objects can be 10 leptons or more below the surface while they take a hit (claim 13, the
  depth limit of the damage effect) is not known.
- A hit with an attacking house that differs from the owner of a present attacker (claim 14) has no
  known source.
- Whether the superweapon sources of claim 1 (the storm bolt and the others it lists) deliver the
  carried data and the ignore-defences marker as claim 1 says is for the superweapon specs.
- Which hits from scripted sources carry the ignore-defences marker (claim 1) should be confirmed by
  each owning spec as it is written.
- A hit on a unit that carries a parasite can also affect the parasite: S-0315 owns it (claim 6, the
  suppression of the parasite by a hit above the threshold of its type or by a healing hit, and
  claim 7, its release by a sonic warhead and its silent removal when it leaves while suppressed).
  Those effects do not depend on any rule of this spec, so they happen even when claim 8 stops
  the hit; this spec does not cover them further.
- How long a crumbling terrain object stays in play, and when it leaves (claim 20), is not traced; a
  terrain spec should own it.
- The value of the fixed factor by which a building's light-source intensity is scaled on *now
  yellow* (claim 12) is not stated in this spec. Scenario D11, part c, records the ratio; the value
  belongs in Constants as an engine constant once known.
- Whether the sources of claim 1 that no filter of another spec covers (the sonic wave, the
  contact destruction, the elimination of a house's objects, the destruction of a bridge section
  and the train crush) skip an object that has no health (claim 8, *No-health targets*) is for
  their specs. If one does not, the two no-health rows of claim 8 become reachable and that spec
  should add a scenario that reads the readiness loss and the berserk effect on an infantry unit
  in a death sequence, or on a crashing vehicle, as D10 and D17 do.
- Whether the superweapon, wave and elimination sources of claim 1 pass the attacker, house and
  markers that the table gives is for their owning specs to confirm; whether a sonic
  wave stops carrying its firing object in the cases where a projectile's detonation does (claim
  1) is not traced.
