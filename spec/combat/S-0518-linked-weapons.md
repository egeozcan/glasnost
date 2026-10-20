---
id: S-0518
title: Linked weapons (prism support beams, charging of overpowerable buildings, beam drawing)
topic: combat
revision: 6
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-20
measurements: []
requests: [Q-0985, Q-1521]
supersedes: []
---

# S-0518: Linked weapons (prism support beams, charging of overpowerable buildings, beam drawing)

## Summary

There are two ways in which one weapon strengthens another. In the first, buildings of the prism type
send support beams to another building of that type, which then fires one stronger shot. This spec
covers which buildings may support and which one is chosen, how many may support at once, when the
beams and the shot happen, how much the shot gains, and what happens when a participant is lost. In
the second, infantry with an electric-charge weapon charge a building of an overpowerable type. This
spec covers how a charger is registered and counted, when the building is overpowered, how infantry
start charging by themselves, and how they stop.

Claims 17 to 23 also cover how a beam is drawn: the support beam, the prism master's shot and every
other laser weapon's beam. That part is presentation only. It changes no game state, draws no number
from the scenario random stream and has no effect on any other claim here.

Other specs own the following, and this spec only points to them:
- the ordinary rearm countdown and firing check (S-0301 claims 1 to 6) and the timing of a building's
  delayed shot (S-0301 claim 14),
- the drawing of charge turrets (S-0301 claim 16), which is not a charging rule,
- shot damage (S-0302 claim 2) and area damage (S-0303 claims 1 and 2), whose detonation input claim 9 here scales,
- how a building processes its attack behaviour (S-0215 claim 32),
- the weapon an overpowered building and a charging unit choose (S-0308 rows 10 and 11),
- the operation rule that chargers bypass (S-0405 claim 8).

Superweapon charge (S-0512) has nothing to do with charging here. No deployed weapon state (S-0517)
enters these rules.

## Terms

- **Prism type.** The one building type that the rules name for prism support (Import mapping).
  When no type is named, nothing in claims 1 to 10 happens.
- **Prism master, supporter.** A building of the prism type that is making an attack is the
  *master*. A building of the prism type that it recruits to send it a beam is a *supporter*.
- **Support beam.** The beam that a supporter draws towards its master. It is drawing only: it has
  no projectile and does no damage (claim 6).
- **Support count *N*.** A per-building whole number, 0 at creation. It counts recruitments
  for the master's next shot (claims 4, 8 and 9), not necessarily distinct supporters if
  a supporter's beam and rearm finish soon enough to let it be recruited again.
- **Support cap *N*ₘₐₓ, support modifier *s*, support rearm *R*ₛ, beam duration *L*ₛ.** Four
  game-wide imported values (Import mapping). *s* is held as a whole-number percentage.
- **Support reach.** The master's weapon range ρ(master, 1) for its slot-1 weapon (S-0307 claim 13).
  It is 0 when the master's type has no slot-1 weapon.
- **Charger list, charger.** Each building has a list of *registered chargers*: infantry units whose
  electric-charge shots have reached it (claim 11). The list holds each unit at most once. *n* is
  its length at a given moment. This is S-0405's *n*<sub>c</sub> (S-0405, Terms, *Charger*).
- **Overpowered.** A per-building state of an overpowerable building, decided in each of its turns
  (claim 13). S-0308 row 10 reads it.
- **Charging mark.** A per-unit flag that a unit carries while it is charging a building by itself
  (claims 14 and 15).
- **Electric-charge warhead, overpowerable type.** As in S-0308 (Terms, rows 11 and 10).
- **Beam.** A straight drawn line between two points that lasts a whole number of ticks and fades
  (claims 18 and 19). Laser weapons, support beams and other specs' effects (for example S-0615
  claims 15 and 16) create beams. A beam is drawing only.
- **Beam parameters.** What a beam is created with: its two end points, its duration *D* in ticks,
  its start and end intensity *S* and *E* with a fade flag, a blink flag, an inner, outer and spread
  colour (each three channels from 0 to 255), a house-colour flag, a thickness *W* and a supported
  flag (claim 18). Without other assignment, *W* = 1 and the house-colour, supported and blink flags
  are off.
- **House laser colour *L*.** A per-house colour of three channels from 0 to 255, derived from the
  house's display colour (claim 17).
- **Intensity *I*.** A real number for one drawn frame of a beam (F6), usually between 0 and 1.
- **Sector *s*.** The beam's direction rounded to one of eight, from 0 to 7 (claim 20).
- **Ordinary drawing, house-colour drawing.** The two ways of drawing a beam, chosen by its
  house-colour flag (claims 21 and 22).
- **Fine mode, coarse mode.** The two modes of the house-colour drawing (claim 22).

## Behaviour

### Prism support

Claims 1 to 10 cover the ordinary building-attack path: a prism type without the SAM flag and
without installed upgrades. Combinations with those special attack paths remain outside these
claims (Open questions).

1. **Who links.** Only a building of the prism type takes part in support. It does so when its
   attack behaviour gets *can fire* (S-0215 claim 32). Then claims 2 to 5 replace the ordinary
   shot of that processing. Other buildings, including buildings of other types with the same
   weapons, fire as S-0301 and S-0215 give. A unit never takes part.
   - Backing: `measurable` (scenario PS1)
   - Confidence: high

2. **Who may support.** When the master gets *can fire* and its support count is below the cap
   (*N* < *N*ₘₐₓ), it looks for a supporter. A building *B* is eligible when all of these hold:
   - *B* belongs to the master's own house (an ally's building never supports),
   - *B* is of the prism type and is not the master,
   - *B* is alive,
   - *B*'s rearm countdown has run out (S-0301 claim 1),
   - *B*'s delayed-fire countdown is 0 (S-0301 claim 14). This keeps out a building whose beam or
     shot is pending, except one whose countdown was set to 0 (quirk
     `quirks.prism_zero_delay_double_count`),
   - *B* is not being drained (glossary; S-0405, Terms),
   - *B*'s current behaviour is not *attack*,
   - the distance F2 between *B* and the master is at most the support reach.

   Nothing else is checked. *B* need not be in operation, switched on or powered, it may be
   under a pulse (S-0405 claim 8), and it may be under construction or being sold (only the *attack*
   behaviour excludes it; quirk `quirks.prism_support_ignores_operation`). *B* need not be able to see or reach the master's
   target.
   - Backing: `measurable` (scenarios PS2 and PS3, including a named test-data pulse case)
   - Confidence: high (each condition); medium (the build-up and sale cases, never observed)

3. **Which one is chosen.** Of the eligible buildings, the one at the smallest distance F2 is
   chosen. On a tie, the one that the house acquired first wins (S-0404 claim 7).
   Acquisition means joining the house's currently owned buildings: placement into play or an
   ownership change to that house. A captured or transferred building ranks after every building
   the new owner already held, even if it was created earlier. Losing and later reacquiring a
   building gives it a new, later rank. Buildings not lost keep their relative order. Acquisitions
   within one tick rank in the order those acquisition events take effect, not by object number.
   An unchanged-owner assignment does not reacquire the building. Thus capture time matters,
   but a capture tick alone is not a complete tie-breaker.
   - Backing: `measurable` (scenario PS4)
   - Confidence: high

4. **One supporter per processing.** When a supporter is found, the master's support count rises
   by 1 and the master does not fire in that processing. The supporter's beam becomes pending
   (claim 6), and the supporter starts its firing animation. Since *can fire* processing repeats
   every tick (S-0215 claim 32), the master recruits at most one supporter per tick. The first
   *can fire* processing that finds no eligible building, or finds *N* ≥ *N*ₘₐₓ, starts the
   master's own delayed shot (claim 7). With initial count 0, a non-negative cap and *K*
   initially eligible buildings that remain available until recruited and cannot become eligible
   again during recruitment, the shot therefore starts at its (min(*K*, *N*ₘₐₓ) + 1)-th
   consecutive *can fire* processing. These are integer counts, with no rounding.
   - Backing: `measurable` (scenarios PS1 and PS5)
   - Confidence: high

5. **No chains.** A supporter never recruits supporters of its own for the master. Recruiting
   happens only in a master's *attack* behaviour, and a building in *attack* is never eligible.
   So the master gains only from direct recruitments. Starting at count 0, at most
   max(*N*ₘₐₓ, 0) recruitments are made for one shot; a non-positive cap gives no support.
   - Backing: `measurable` (scenario PS5)
   - Confidence: high

6. **Support beam.** The supporter's beam uses the supporter's delayed-fire countdown, set to its
   type's delayed-fire field *d* (ticks) when it is recruited.
   - **When.** The beam comes in the supporter's turn max(*d* − 1, 0) ticks after the tick of
     recruitment, if the supporter takes its turns after the master in the update order (S-0203
     claim 3). If it takes them before the master, the beam comes one tick later.
   - **Path.** The beam runs from the supporter's fire point for weapon slot 0 to the master's fire
     point for weapon slot 0 (glossary, *fire point*), as that point was at recruitment. The beam
     does not follow the master afterwards.
   - **Drawing.** The beam lasts *L*ₛ ticks and is drawn in the house-colour drawing with
     thickness parameter 3 and the supporter's house laser colour (claims 18 and 22).
   - **Effects.** None in play: no projectile, no damage, no ammunition, no burst count.
   - **Afterwards.** In the same turn, the supporter's rearm countdown starts with exactly *R*ₛ
     ticks: no rate-of-fire factor and no random draw apply. Its own support count becomes 0.
   - **While pending.** The supporter's firing check answers *rearming* (S-0301 claim 14).
   - Backing: `measurable` (scenario PS5)
   - Confidence: high (path, rearm, no damage); medium (the one-tick dependence on update order)

7. **The master's shot.** For a master, claim 4's last processing always starts S-0301 claim 14's
   delayed shot, whether or not its type has the animation-delayed-fire flag. The countdown is the
   type's delayed-fire field. The shot always uses weapon slot 0, even if weapon choice gave slot 1
   for the check that answered *can fire*. When the delayed shot comes, the check and the shot
   follow S-0301 claim 14. The shot's rearm countdown is the ordinary one (S-0301 claims 1 to 6).
   With a delayed-fire field of 0, the shot comes in the same turn as the processing that started
   it.
   - Backing: `measurable` (scenarios PS1 and PS5)
   - Confidence: high

8. **The bonus is fixed when the master fires.** The gain depends only on the master's support
   count *N* at the moment its delayed shot launches a projectile. It does not depend on whether
   the beams have been drawn yet or whether the supporters are still alive. A supporter destroyed,
   sold or captured after it was recruited still counts. Recruitment, not arrival of a beam,
   establishes the contribution. All participants have the same type and thus the same
   delayed-fire field; independently longer supporter delays are not a stock-play case.
   - Backing: `measurable` (scenario PS6)
   - Confidence: high

9. **Supported damage.** When the master's delayed shot launches a projectile and *N* > 0, the
   projectile's damage multiplier becomes *M* (F1) and the master's *N* becomes 0. Every
   projectile starts with *M* = 256, that is, ×1. For an ordinary damaging detonation, the
   amount supplied to area damage is ⌊*D*ₜ × *M* / 256⌋ (F1), where *D*ₜ is the integer
   damage still carried at detonation: S-0302 claim 2's shot damage after any in-flight loss
   (S-0306 claim 18). Falloff, versus values and target defences apply afterwards, not before
   this scaling. This is not necessarily the target's health loss. A shot with *N* = 0 is
   unchanged. When *N* > 0 the master's own beam, if its weapon draws a house-colour beam,
   has thickness parameter 5 instead of 3 and is drawn in supported style (claims 18 and 22),
   for drawing only. For a beam without the house-colour flag, thickness has no visible effect.
   - Backing: `measurable` (scenario PS1)
   - Confidence: high

10. **Count reset and participant loss.** The master's support count becomes 0 in these cases:
    - its shot launches a projectile (claim 9),
    - its attack behaviour is processed with no target,
    - its attack behaviour drops the target (S-0215 claim 32: *illegal*, *impossible*,
      *ammunition* or *out of range*).

    The count is kept, with no beam redrawn, when the delayed shot is dropped by its own firing
    check (S-0301 claim 14) or launches no projectile while the master keeps its target. The next
    *can fire* processing then adds up to *N*ₘₐₓ − *N* further supporters, and the shot after
    that is scaled by the total (quirk `quirks.prism_count_survives_dropped_shot`).

    If the master is destroyed or sold after recruiting, its supporters still draw their beams to
    the stored point and then rearm by claim 6.
    - Backing: `measurable` (scenario PS6) for resets, participant loss and a dropped firing check.
      Retention after a projectile-less launch: `unmeasured`, because no concrete data-only
      fixture is known that makes launch fail while preserving the target and support count.
    - Confidence: high (the resets and the master's loss); low (how often the kept count can
      occur in play)

### Charging of overpowerable buildings

11. **The charge shot.** A detonation that selects the electric-charge effect skips ordinary
    area damage at any target. S-0514 claim 20 owns the complete special-effect selection and
    precedence: mind control and demolition charge planting outrank electric charging, which
    outranks parasite attachment and temporal erasure. A selected charge effect that cannot
    register a charger still suppresses all lower-priority effects and ordinary damage; it does
    not fall through to another effect. It does not suppress effects shared
    by detonations, including radiation creation before effect selection and the later
    detonation animation, debris and flash where configured. If the projectile's target is a
    building and its firer is an infantry unit, the electric-charge effect puts the firer on
    the building's charger list:
    - this happens to any building, whatever its type or owner,
    - a firer already on the list is not added twice,
    - a firer that is not infantry registers nothing.
    - Backing: `measurable` (scenarios TC1 and TC4)
    - Confidence: high (area-damage bypass and registration); medium (combined effects)

12. **Who stays on the list.** A charger leaves the list when it leaves play. For a building of an
    overpowerable type there is a second way off: in each of the building's turns, every charger
    whose current target is not that building is removed. So *n* counts the registered chargers
    that are still aiming at the building, as of its last turn. For a building of another type,
    a charger stays on the list until it leaves play (quirk `quirks.stale_chargers_kept`).
    S-0405 claim 8 reads *n* whenever it judges the building.
    - Backing: `measurable` (scenarios TC1 and TC4)
    - Confidence: high

13. **Overpowered.** In each of its turns, after claim 12's removal, a building of an
    overpowerable type is *overpowered* when either of these holds:
    - *n* ≥ 3,
    - *n* ≥ 1, the building is switched on, and its owner has full power (*p* = 1 in S-0405 F3).

    Otherwise it is not overpowered. A building of another type is never overpowered. While a
    building is overpowered, weapon choice gives it slot 1 (S-0308 row 10). Independently of this
    state, *n* ≥ 2 bypasses only the switched-on and house-power requirements of operation,
    every other requirement still applies (S-0405 claim 8).
    - Backing: `measurable` (scenario TC1)
    - Confidence: high

14. **Starting to charge on its own.** During *guard* or *area guard* processing, an infantry unit or vehicle whose slot-1 weapon has an electric-charge warhead searches for a building only if it has no target at that point. In *area guard*, the guard-spot and automatic-search steps have already taken place. The search covers the eight cells next to the unit's cell in step-direction order 0 through 7 (glossary, *step direction*); the first cell in that order containing an overpowerable building owned by the unit's own house determines the result. An allied building is not eligible. When such a building is found, it becomes the unit's target, the unit's charging mark is set, and *attack* is queued without starting immediately. Finding a building does not select a special 1-tick delay.
    The processing keeps its ordinary delay, using the rate of the behaviour being processed,
    not the queued *attack* rate:

    | Behaviour being processed | Delay after a successful charging scan | Scenario-stream draws for this delay |
    |---|---|---|
    | *guard*, weapon still rearming | rearm ticks remaining | none |
    | *guard*, not rearming, distributed-fire type with nonempty distributed-fire candidates | 0 ticks | none |
    | *guard*, otherwise | standard delay, S-0215 claim 24 | one ranged draw over 0 to 2 ticks inclusive |
    | *area guard* | rate delay plus one ranged draw over 1 to 5 ticks inclusive, shortened by division by 6 under S-0215 claim 26's close-range condition on the newly taken building | one ranged draw over 1 to 5 ticks inclusive |

    Delay arithmetic is signed 32-bit integer arithmetic with wrapping sums and division truncated
    toward zero; rate conversion and the close-range test are those of S-0215 claims 24 and 26.
    A delay of at most 1 tick schedules the next processing on the next tick (S-0215 claim 2).
    In *guard*, if no such building is found, ordinary guard processing runs instead; when one
    is found, its idle action does not run, but delay selection still does.
    A human player can also make a unit charge with
    an attack order on an allied overpowerable building (S-0308 row 11). There is no special
    cursor for this. A vehicle charges in the same way, but registers nothing (claim 11).
    - Backing: `measurable` (scenarios TC2 and TC4)
    - Confidence: high (the scan and the own-house rule); medium (the order of the steps within
      *area guard*)

15. **Breaking off.** While an infantry unit's charging mark is set, each processing that reaches
    the general infantry *attack* rules runs an automatic search of the range kind centred on its
    position, before ordinary target approach. Earlier special-case outcomes still take precedence
    (S-0215 claim 23). The search is the range search of S-0309
    claim 1. The held building target does not prevent this search: it is not tested for
    usability or protected by S-0309 claim 4's target-retention rule, and is not cleared to make
    the search possible. A nonempty search result becomes the target and clears the mark,
    without a further test of the held target's owner, firing legality or identity against the
    result. Even a result equal to the held building clears the mark. If the search finds
    nothing, the held target and mark remain unchanged. Ordinary candidate tests and ranking
    still apply (S-0309 claims 5 to 17); in particular the ordinary damaging charger's own-house
    building is not an enemy candidate merely because it is being charged.

    Neither S-0309 claim 2's passive-acquisition permissions nor claim 3's opportunity gate
    limits this search. The intrinsic no-auto-fire restriction of S-0309 claim 5 still does.
    There is no targeting-timer gate or targeting-timer reset draw for this search; ordinary
    behaviour-delay draws still apply (S-0215 claim 21). The resulting target is approached
    in the same processing.
    The mark also clears whenever the unit is given a target by an
    order or by any other ordinary target change. A unit that stops aiming at the building drops
    off the building's list at the building's next turn (claim 12). This search belongs to the
    *attack* behaviour of infantry only. A vehicle with the mark set does not break off and keeps
    its target until another target change (Edge cases).
    - Backing: `measurable` (scenarios TC3 and TC4)
    - Confidence: high (search and switch); medium (the other clears)

16. **Support-modifier layering.** Reading a rules layer that contains the general section
    rescales the held support modifier by F4, even if that section omits the modifier key.
    A layer without the general section does not rescale it. Consequently a map can change
    supported damage without naming the modifier (quirk `quirks.prism_modifier_reread_scale`).
    - Backing: `measurable` (scenario PS7)
    - Confidence: high

### Beam drawing (presentation)

Claims 17 to 23 are presentation only. A beam is created in the simulation tick in which its
cause happens, but nothing in play reads it: it has no damage, no collision and no effect on
targeting, and it uses no number from the scenario random stream. Its lifetime is counted in
simulation ticks (claim 19). The renderer draws every live beam in each drawn frame after all
layers (S-0102 claim 2), but still tests it against the terrain picture's depth (claim 20).

17. **House laser colour.** Each house has a house laser colour *L*. It is computed once, when
    house colours are assigned at game setup, from the house's display colour of S-0103 claim 15
    (its widened three-channel value), by F5. It does not change during the game. A house that is
    never given a colour keeps the starting value (255, 255, 255). Its effect is to turn any house
    colour into a bright, saturated colour of fixed overall strength: weak channels are cut to 0
    and the rest are scaled up. There is no data key for *L*; it follows from the house colour.
    - Backing: `measurable` (scenario LB1) for assigned colours. The never-assigned starting
      white: `unmeasured`, because normal firing houses are assigned a colour before play.
    - Confidence: high

18. **Beam parameters by source.** The beam parameters (Terms) are set as follows. Every source
    below uses the fade flag with *S* = 1 and *E* = 0, and leaves the blink flag off.

    | Source | Duration *D* | Colours | House-colour flag | *W* | Supported |
    |---|---|---|---|---|---|
    | Support beam (claim 6) | *L*ₛ | inner *L* of the supporter's house; no outer or spread colour | on | 3 | off |
    | Prism master's shot, *N* = 0 | weapon's laser duration | weapon's colours (below) | weapon's flag | 3 | off |
    | Prism master's shot, *N* > 0 (claim 9) | weapon's laser duration | weapon's colours | weapon's flag | 5 | on |
    | Another building's laser weapon | weapon's laser duration | weapon's colours | weapon's flag | 1 | off |
    | A vehicle's, infantry unit's or aircraft's laser weapon | weapon's laser duration | weapon's colours | weapon's flag | 2 if the flag is on, else 1 | off |

    - **Thickness and supported flag.** These two parameters show only in the house-colour
      drawing (claim 22). The ordinary drawing ignores them.
    - **Which shots draw a beam.** A shot draws a beam when the weapon that fires has the laser flag.
      The beam is created when the shot is fired. It is drawn in addition to whatever the weapon's
      projectile does. The prism type is recognised only for buildings.
    - **Weapon's colours.** Without the house-colour flag the inner, outer and spread colours are
      the weapon's three laser colour fields. With it, the inner colour is the firer's *L* and the
      outer colour is *L* with each channel halved, rounding down, and the spread colour is the
      weapon's. The house-colour drawing reads only the inner colour (claim 22).
    - **Duration.** *D* is the weapon's laser-duration field in ticks. A *D* ≤ 0 gives no
      visible beam (claim 19).
    - **Original bug, building look.** For a building, the original takes the colours, the
      house-colour flag and the duration from its slot-0 weapon, even when another slot fires; the
      laser flag and the fire point come from the slot that fires. The claim above, which uses the
      firing weapon throughout, is the fixed behaviour.
    - **Original bug, long durations.** The original holds the laser-duration field as a signed
      8-bit value, so a written value from 128 to 255 gives no beam and larger values wrap modulo
      256. The fixed behaviour is the claim above: *D* is the field's value.
    - **Other specs.** Beams created by other specs carry the parameters those specs give. When a
      spec gives no thickness or flags, the Terms defaults apply.
    - Backing: `measurable` (scenarios LB2, LB3 and LB5)
    - Confidence: high (sources and thickness); high (both bug notes)

19. **Lifetime, blink and intensity.** A beam's age *P* is 0 in the tick it is created and rises by
    1 in each later tick, after all objects' turns and before that tick's drawn frame. The beam is drawn in the drawn frames
    of the ticks where *P* = 0, 1, …, *D* − 1, so *D* consecutive ticks, the first being the
    creation tick. It is gone from the tick where *P* = *D*. A beam with *D* ≤ 0 is never drawn.
    - **Intensity.** With the fade flag, a drawn frame uses *I* by F6, from *S* at *P* = 0 to
      *E* + (*S* − *E*)/*D* at the last drawn tick. *E* itself is never reached on a drawn frame.
      Without the fade flag, *I* = 1.
    - **Blink.** With the blink flag, an ordinary-drawing beam (claim 21) is drawn only at odd *P*,
      and missing at even *P*, including the creation tick. The house-colour drawing ignores the
      blink flag. No source in claim 18 sets it.
    - Backing: `measurable` (scenario LB3). The blink rule: `unmeasured`, because no known stock
      source sets it and no known data key exposes it for a named-test-data fixture.
    - Confidence: high (lifetime, intensity); medium (blink)

20. **End points, direction and depth.**
    - **End points.** A weapon's beam runs from the firer's fire point for the slot that fired
      (glossary, *fire point*) to the target's target point when the target is an object, or the
      target cell's centre at its height otherwise. A support beam's end points are those of
      claim 6. Both are fixed when the beam is created: the beam does not follow either object.
      Each end is placed on screen by S-0101's projection, height lift included.
    - **Sector.** The beam's sector *s* is the plan direction from the source to the target,
      using world X and Y only, rounded to the nearest eighth of a turn and counted clockwise from
      world north (decreasing Y), so *s* = 0 is north and *s* = 2 is east. Height is ignored, so
      for a steep beam the on-screen slope may differ from what *s* suggests. Directions exactly
      half-way between two sectors are not asserted (Open questions).
    - **Depth.** A beam reads depth and never writes it (S-0102 claims 8 and 10). Its depth
      references at the source and target are S-0102 F1's screen-row depths plus the respective
      biases *b*₀ = *a* − *L*(*z*₀) − 2 and *b*₁ = −*L*(*z*₁) − 2. *L*(*z*) is each end's
      height lift (S-0101 claim 3), and *a* is 0 except as given below. For an unclipped opaque
      or additive line, F9 specifies the exact interior samples and integer depth steps; “linear”
      does not mean truncating the interpolated bias at every pixel. A sample contributes only
      where its depth is strictly less than the stored depth. Terrain, cliffs, overlays and
      buildings drawn by the cell pass can therefore hide parts of a beam, while units never do.
      Exact interior depth sampling for blended lines and for lines clipped at the view edge
      remains open; the reference biases, strict test and absence of depth writes still apply.
    - **Building source.** For a weapon beam fired by a building, *a* = min(0, *y*ₛ − *y*ₒ),
      where *y*ₛ is the screen row of the fire point and *y*ₒ the screen row of the point 128
      leptons from the building's position toward decreasing X and decreasing Y. This brings the
      source end of a tall building's beam nearer, so the building's own image does not hide it.
    - **Clipping.** Each line is clipped to the battlefield view.
    - Backing: `measurable` (scenarios LB4, LB5 and LB7)
    - Confidence: high (end points, sector rule, read-only depth); medium (bias values, the
      building adjustment, F9's sampling)

21. **Ordinary drawing.** A beam without the house-colour flag is drawn as a centre line in the
    inner colour and, when the outer colour is not black (0, 0, 0), two outer lines. The outer lines
    are the centre line moved by one screen pixel, using the sector's pair of shifts below (screen *x*
    to the right, *y* downward).

    | *s* | first outer line | second outer line |
    |---|---|---|
    | 0 | (0, −1) | (1, 0) |
    | 1 | (0, −1) | (0, 1) |
    | 2 | (1, 0) | (0, 1) |
    | 3 | (−1, 0) | (1, 0) |
    | 4 | (−1, 0) | (0, 1) |
    | 5 | (0, −1) | (0, 1) |
    | 6 | (0, −1) | (−1, 0) |
    | 7 | (−1, 0) | (1, 0) |

    - **Spread.** When outer lines exist, each drawn frame of each beam draws one value per
      channel, red, then green, then blue, from the unsynchronised random stream (glossary), each
      uniform over the whole numbers from −σ to +σ, where σ is that channel of the spread colour.
      Each outer channel used in that frame is the outer colour's channel plus its draw, clamped to
      0 … 255. No draw happens when there are no outer lines.
    - **Detail level 0.** The lines are opaque (claim 23), in their full colours; intensity does
      not apply. Where depth-passing samples overlap, the second outer line's colour wins over
      the first outer line's colour, and either wins over the centre colour.
    - **Detail level 1 or 2, no blink flag, outer lines present.** The lines are additive at
      intensity *I* (claim 23). At an overlap the resulting pixel is
      *C*(*O*₂(*O*₁(*d*))), where *d* is the underlying screen pixel and each symbol is that
      line's F7 pixel transformation, including its separate colour reduction. This matters:
      spill and colour reduction mean the result need not equal one addition of summed colours.
    - **Detail level 1 or 2, otherwise.** The centre line is blended (claim 23) at intensity *I*.
      With the blink flag and outer lines present, both outer lines are blended at intensity 1, and
      an overlapping pixel equals *O*₂(*O*₁(*C*(*d*))) using the F8 transformations. A sample
      failing depth contributes the identity transformation. These expressions constrain the
      final pixels, not an implementation's drawing sequence.
    - **Original bug, sector 5.** The original uses (0, −1) and (1, 0) in sector 5, so one outer line
      lies along the beam and only one shows; the table gives the fixed pair.
    - Backing: `measurable` (scenarios LB2, LB4, LB6 and LB8), except blink-only blended overlap,
      which is `unmeasured` for claim 19's lack of a data-only fixture. The random draws:
      `measurable` with the random-state recorder (LB6).
    - Confidence: high (line set, shifts, draw order and count, detail-level split)

22. **House-colour drawing.** A beam with the house-colour flag is drawn from the inner colour *L*
    (claim 18) as a centre line with up to *W* pairs of side lines.
    - **Colours.** The centre colour *K* is *L*, or, when the supported flag is on, 2*L* clamped to
      255 per channel. The first pair's colour is *L* halved per channel, rounding down, or *K* when
      supported. After pair *i*, the next pair's colour is *L* when supported and *i* = 1, and
      otherwise the current pair's colour halved per channel, rounding down. When that halved colour
      has all three channels below the stop threshold, no further pairs are drawn. The threshold is
      8 in fine mode and 64 in coarse mode.
    - **Shifts.** Pair *i* (1 ≤ *i* ≤ *W*) is two lines on opposite sides of the centre line.
      In sectors 1, 3, 5 and 7 (lines that are horizontal or vertical on screen) the two lines of
      pair *i* are moved by *i* × *e*ₐ and *i* × *e*ᵦ. In sectors 0, 2, 4 and 6 (diagonal on
      screen) the steps alternate: the first line is moved by (⌈*i*/2⌉ × *e*ₐₓ, ⌊*i*/2⌋ × *e*ₐᵧ)
      and the second by (⌊*i*/2⌋ × *e*ᵦₓ, ⌈*i*/2⌉ × *e*ᵦᵧ). The vectors are:

      | *s* | *e*ₐ | *e*ᵦ |
      |---|---|---|
      | 0 and 4 | (−1, −1) | (1, 1) |
      | 1 and 5 | (0, −1) | (0, 1) |
      | 2 and 6 | (−1, 1) | (1, −1) |
      | 3 and 7 | (−1, 0) | (1, 0) |

    - **Overlap result.** In coarse mode an overlapping pixel takes the centre colour when
      its sample passes depth; otherwise it takes the colour of the highest-numbered passing
      pair, with that pair's second line winning a tie. In fine mode the pixel is the composition
      *C* ∘ *B*ⱼ ∘ *A*ⱼ ∘ … ∘ *B*₁ ∘ *A*₁ applied to the underlying pixel, where *j* is the
      last visible pair, *A*ᵢ and *B*ᵢ are its first and second line's F7 transformations, and
      *C* is the centre's. Every transformation includes colour reduction; a depth-failing
      sample or a line missing that pixel is the identity. Thus different ways of generating
      the lines must agree at overlap.
    - **Width.** A beam is therefore at most 2*W* + 1 pixels across when it is horizontal or
      vertical on screen. A colour whose channels halve below the threshold sooner draws fewer pairs.
    - **Fine and coarse mode.** Fine mode applies when the detail level is not 0 and the frame
      rate test passes. The test compares a held integer activity count *f* with a floor *m*
      and margin *z* (Import mapping), with a shared remembered low-rate state that starts clear.
      Despite the settings' frame-rate terminology, *f* counts simulation ticks, not displayed
      frames, and is not divided by elapsed seconds. A separate integer counter gains 1 at the
      start of every simulation tick. At the cycle-end rate check, when the pacing clock has
      advanced at least 60 units since the previous publication, *f* becomes that counter and
      the counter becomes 0; the publication clock restarts at the current clock reading.
      One clock unit is 16 ms: the clock reading is the host's integer millisecond clock
      divided by 16, rounding down before publication readings are subtracted.
      Thus a regular window is 960 ms, extended to the first cycle-end check at or after expiry.
      delayed checks do not normalise the count. The first such check in a fresh program session
      publishes immediately; *f* starts at 0 before it. Between publications *f* is unchanged.
      The hysteresis state is shared with other visual effects using the same test, so those
      effects may change it even between beam frames. While the state is clear, the test passes when *f* ≥ *m*, and a failure sets the state.
      While it is set, the test passes only when *f* ≥ *m* + *z*, and a pass clears it. Otherwise
      coarse mode applies. In fine mode every line, centre included, is an additive line (claim 23)
      at intensity *I*. In coarse mode every line, centre included, is an opaque line whose colour
      *c* is first moved toward white by *c*′ = *c* + trunc((255 − *c*) × *r* / 256), per channel,
      with *r* = trunc(*I* × 255). The intensity product is double-precision floating point
      with the single-precision *I* widened, truncated toward
      zero; the remaining arithmetic is integer, with division by 256 rounded down (equal to
      truncation for the non-negative colours and intensities in claim 18). So at full intensity
      a coarse beam is nearly white.
    - **Random draws.** None.
    - Backing: `measurable` (scenarios LB1, LB2, LB3, LB5 and LB8)
    - Confidence: high (colours, shifts, pair count, thresholds, order), medium (mode choice and
      its activity-count measure), medium (the coarse-mode colour step)

23. **How lines meet the screen.** All three kinds of line below are depth-tested and clipped as in
    claim 20. Opaque and additive lines have a one-pixel-wide path; a blended line distributes
    colour over two neighbouring pixels across that path. The screen holds 16-bit colour, and the light-alpha value *a*
    of each screen pixel is S-0104's (Terms; 127 where nothing changes it).
    - **Opaque line.** Each drawn pixel is replaced by the line's colour reduced to 16-bit colour.
      Light-alpha does not apply, so opaque beams stay fully visible over shroud.
    - **Additive line.** The line's colour is first scaled by its intensity: each channel becomes
      trunc(*c* × *I*). The line is not drawn at all when all three scaled channels are below 8, so
      a fading house-colour beam loses its faint outer pairs first. Each drawn pixel then gets the
      scaled colour added by F7, scaled by *a*. Where *a* = 0 (full shroud) nothing is added and
      the beam is invisible.
    - **Blended line.** The line is anti-aliased: at each step along its long axis, the two pixels
      bracketing the path share it by integer coverage values. Each gets a mix of its colour and
      the line's colour by F8, with the coverage-to-integer-weight conversion specified there,
      scaled by *a*. A sample pair uses the first pixel's light-alpha for both pixels, not two
      independently sampled values. Only channels where the line's inner colour is above 0 change. F8 specifies
      coverage for unclipped lines whose depth-bias difference does not exceed their long-axis
      length; other coverage cases remain open.
    - Backing: `measurable` (scenarios LB2, LB4 and LB6)
    - Confidence: high (which kind of line ignores light-alpha); medium (F7 and F8 within its
      stated domain); coverage outside F8's stated domain remains open

## Formulas

**F1, supported damage multiplier** (claim 9):

> *U* = ((*s* × *N* + 100) × 256) modulo 2³², interpreted as a non-negative integer.
> *M* = ⌊*U* / 100⌋, unsigned integer division truncating toward zero.
> *P* = the signed 32-bit interpretation of (*D*ₜ × *M*) modulo 2³².
> *D* = ⌊*P* / 256⌋, integer division rounded toward −∞.

All integer additions and multiplications here retain only the low 32 bits. When no product
wraps, this is simply *M* = ⌊(*s* × *N* + 100) × 256 / 100⌋ and
*D* = ⌊*D*ₜ × *M* / 256⌋. *D* is the area-damage input, not a defended health loss.

*s* is the held whole-number percentage, *N* the master's support count (1 to *N*ₘₐₓ) and *D*ₜ
the projectile's damage at detonation (claim 9). For example, *s* = 50 and *N* = 2 give
*M* = 512, which is ×2.

**F2, support distance** (claims 2 and 3):

> δ = trunc(√(Δx² + Δy² + Δz²)), in leptons.

Δx, Δy and Δz are the differences between the two buildings' positions, each the centre of the
building's top-left foundation cell at its height (glossary, *top-left cell*). These are integers
in leptons. Differences use signed 32-bit integer subtraction. The height difference is
converted through single precision, then widened to double. In the original's floating-point
round-toward-zero mode (S-0205), the sum is formed as (Δx² + Δz²) + Δy²
in double precision, followed by the square root, then truncation toward zero to an integer. Height counts, so a supporter on a cliff is farther away
than one at the master's level at the same plan distance.

**F3, overpowered** (claim 13): overpowered ⇔ *n* ≥ 3 ∨ (*n* ≥ 1 ∧ switched on ∧ *p* = 1).

**F4, held support modifier** (Import mapping): each read of a rules layer that has the general
section turns the held value *h* into *h*′ = trunc(*v* × 100), where *v* is a double.
- With the key present, *v* is the field's decimal value (S-0003 claims 6 and 7): the written
  number in single precision, times the double nearest to 0.01 when the text contains a percent
  sign.
- With the key absent, *v* = *h*.

The product *v* × 100 is one double multiplication in floating point in the original, rounded as
S-0003 claim 7's percent product is, and the result is then truncated toward zero to an integer.
So a written percentage whose ×0.01 product falls just below the exact value loses 1: "10%" gives
9, while "125%" gives 125. The held result is a signed 32-bit integer; values outside the
conversion domain follow S-0205's invalid-conversion rule.

**F5, house laser colour** (claim 17). The input is the house display colour (*r*, *g*, *b*), each
channel from 0 to 255 as S-0103 claim 15 widens it. All arithmetic is in double precision until the
final truncation.

> *n* = √(*r*² + *g*² + *b*²). If *n* = 0, *L* = (255, 255, 255).
> Otherwise, for each channel *c*: *c*′ = min(255, *c* × 240 / *n*), and *c*′ = 0 if *c*′ < 96.
> *n*′ = √(*r*′² + *g*′² + *b*′²); for each channel, *c*″ = min(255, *c*′ × 240 / *n*′).
> *L* = (trunc(*r*″), trunc(*g*″), trunc(*b*″)), truncated toward zero.

The second scaling cannot meet *n*′ = 0: after the first scaling the largest channel is at least
240/√3 ≈ 138.6, which is above the cut-off. (Should it occur, the original gives white.)

**F6, beam intensity** (claim 19), in single precision:

> *I* = *E* + (*S* − *E*) × (*D* − *P*) / *D*, with the fade flag; *I* = 1 without it.

With *S* = 1 and *E* = 0 this falls linearly from 1 at *P* = 0 to 1/*D* at *P* = *D* − 1.

**F7, additive pixel** (claim 23), integers. The screen pixel's 16-bit colour is widened to three
channels from 0 to 255 by appending zero bits. For each channel with line colour *c* and screen
channel *d*:

> *t* = *d* + ⌊*c* × *a* / 128⌋.

Let *k* be the number of channels with *t* ≥ 255 and *X* the sum of their *t* − 255. When *k* is 1
or 2, every channel with *t* < 255 gets a spill added: ⌊*X*/2⌋ when *k* = 1, and *X* when *k* = 2.
Then each channel is clamped to 255 and the result is reduced to 16-bit colour by dropping the
low bits. So a strong channel overflowing whitens the pixel instead of being lost.

**F8, blended pixel** (claim 23), integers. An easing table has 257 entries:

> *T*[*i*] = trunc(((*i*/128 − 1)³ + 1) × 128), for *i* = 0, …, 256.

For integer coverage *q* from 0 to 256, *w* = trunc(*q* × *I*), a double-precision product
with the single-precision intensity widened, truncated toward zero. This gives an integer weight from
0 to 256 for intensities in 0 … 1. Screen channel *d* is widened from 16-bit colour by appending
zero bits; *c* is the line's channel. Then:

> *d*′ = ⌊(*d* × *T*[256 − *w*] + *c* × *T*[*w*]) × *a* / 2¹⁵⌋.

Only channels with *c* > 0 change, and a pixel with *a* = 0 is unchanged, not replaced by black.
The result is reduced to the screen's 16-bit colour by dropping low bits. The table expression
is evaluated in double-precision floating point before truncation toward zero; the pixel formula
uses integer arithmetic and floor division.

For unclipped lines with distinct screen end points and a depth-bias difference no greater than
the long-axis length, coverage is as follows. Name the end with smaller screen *x* the first end
(retain source first on equal *x*). Let *H* = max(|Δ*x*|, |Δ*y*|) pixels and
*h* = trunc(256 × min(|Δ*x*|, |Δ*y*|) / *H*), evaluated in double precision and truncated toward
zero. The long-axis sample index is *k* = 0, …, *H* − 1; the terminal end is excluded.

> *v* = *k* × *h*.
> *q*₂ = 0 for *v* = 0; otherwise *q*₂ = 1 + ((*v* − 1) modulo 256).
> *q*₁ = 256 − *q*₂; *j* = (*v* − *q*₂) / 256, exact integer division.

The first pixel lies *k* pixels along the long axis and *j* pixels along the short axis from the
first end, with each axis's sign toward the other end. The second lies one further pixel along
the short axis. On an axis-aligned line the first pixel alone has nonzero coverage. At a slope
of one half, the sequence begins (256, 0), (128, 128), (0, 256), (128, 128), (0, 256).
This is a quantised slope, not exact geometric coverage for arbitrary slopes. Coverage after
view-edge clipping or for larger depth-bias differences remains open.

**F9, unclipped opaque/additive line samples** (claim 20), exact integer arithmetic. Reorder the
end points by increasing screen *x*, retaining source first on equal *x*, and carry each end's
bias with it. Let Δ*x*, Δ*y* and Δ*b* be the signed differences from the first end, and
*H* = max(|Δ*x*|, |Δ*y*|, |Δ*b*|). For *H* > 0, samples have indices *k* = 0, …, *H* − 1.
For each component difference Δ, its displacement at sample *k* is

> sign(Δ) × ⌊(2*k* × |Δ| + *H* − 1) / (2*H*)⌋.

Add these displacements to the first end's screen coordinates and bias. This is nearest-integer
interpolation with exact half ties toward the first end, including a negative bias difference.
The sample depth is its row depth plus its interpolated bias. More than one sample can land on
the same screen pixel when the bias difference dominates; each passing sample contributes the
line's pixel transformation. A zero-length triple (*H* = 0) contributes no sample. This formula
does not assert how clipping changes end points, biases or the sample phase.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Support beam thickness parameter *W* | 3 | side-line pairs | claims 6 and 18; drawing only |
| Master beam thickness parameter when supported | 5 (3 otherwise) | side-line pairs | claims 9 and 18; drawing only |
| Thickness parameter of a unit's house-colour laser | 2 | side-line pairs | claim 18 |
| Default thickness parameter | 1 | side-line pairs | claim 18 |
| House laser colour target length | 240 | channel units | F5 |
| House laser colour cut-off | 96 | channel units | F5; a channel below it becomes 0 |
| Beam depth nearness | 2 | depth units | claim 20 |
| House-colour side-line stop threshold | 8 (fine), 64 (coarse) | channel units | claim 22 |
| Additive-line skip threshold | 8 | channel units | claim 23 |
| Light-alpha scale | 128 | – | F7, F8 |
| Easing table size | 257 | entries | F8 |
| Activity publication window | 60 | clock units of 16 ms | claim 22; not one second |
| Unscaled damage multiplier | 256 | 1/256 | claim 9 |
| Chargers that overpower whatever the power | 3 | infantry units | claim 13 |
| Chargers that overpower when switched on at full power | 1 | infantry units | claim 13 |
| Cells scanned for a building to charge | 8 | cells | claim 14 |
| Supporters recruited per processing | 1 | buildings | claim 4 |
| Support modifier scale per read | 100 | – | F4 |

## State and ordering

**Master and supporter stages.** Each building of the prism type is *idle*, *support pending* or
*shot pending*.

| State | Event | Result |
|---|---|---|
| master idle, *N* < *N*ₘₐₓ | *can fire*; an eligible building exists | the nearest becomes *support pending*; the support count increases by 1; no shot |
| master idle | *can fire*; no eligible building, or *N* ≥ *N*ₘₐₓ | master *shot pending* (claim 7) |
| master *shot pending* | its delayed shot comes and launches a projectile | projectile scaled by F1 when *N* > 0; the support count becomes 0; master idle; ordinary rearm |
| master *shot pending* | its delayed shot is dropped or launches nothing | master idle; *N* kept (claim 10) |
| master idle | attack processing with no target, or the target is dropped | the support count becomes 0 |
| supporter *support pending* | its beam comes (claim 6) | beam drawn; its rearm countdown starts at *R*ₛ; its own support count becomes 0; idle |
| supporter *support pending* | the master is destroyed | no change; the beam still comes |

**Charger and overpowered state** (overpowerable building).

| State | Event | Result |
|---|---|---|
| any | an infantry unit's electric-charge projectile detonates on it | that unit is on the list (once) |
| any | its turn | chargers no longer aiming at it are removed; its overpowered state is set according to F3 |
| any | a charger leaves play | removed from the list at once |

Observable consequences:
- Recruitment gives a bonus without waiting for an observed beam. Loss of a supporter after
  recruitment therefore cannot remove that contribution (claim 8).
- Under claim 4's assumptions, a master with *K* eligible supporters starts its delayed shot
  min(*K*, *N*ₘₐₓ) ticks later than it would alone. Its own delayed-fire time is unchanged.
- Two masters of one house share the pool. Except for the zero-delay window described below,
  a building recruited by one is not eligible for another recruitment until it has drawn its beam
  and its support rearm has run out. Very short support rearm can permit repeated recruitment
  of one building during a single master's recruitment period.
- The overpowered state changes only in the building's own turn. A change in power, or a charger
  that stops aiming, takes effect at that turn. A charger that leaves play stops counting for
  S-0405 claim 8 at once.

## Edge cases and quirks

- **`quirks.prism_support_ignores_operation`** (default on). Supporters need not be in operation
  (claim 2). A supporter that is switched off, under a pulse, under construction or being sold
  still supports a working master. (At low power the master itself is out of operation and does
  not attack, so low power alone shows nothing.) *Fixed:* a building that is not in operation
  (S-0405 claim 8) is not eligible.
- **`quirks.prism_count_survives_dropped_shot`** (default on). A dropped or projectile-less delayed
  shot keeps the support count, and the next shot gets the kept bonus on top of new supporters,
  with no beams for the kept part (claim 10). *Fixed:* the count becomes 0 whenever the delayed
  shot comes, fired or not.
- **`quirks.prism_zero_delay_double_count`** (default on). When the prism type's delayed-fire field
  is 0, a recruited supporter's delayed-fire countdown stays 0 until its turn. Another master of the
  same house processed before that turn finds it eligible and also counts it. One beam is drawn
  and both masters gain. *Fixed:* a building with a pending beam is not eligible. Whether this case arises with unmodified data is not asserted here.
- **`quirks.prism_modifier_reread_scale`** (default on). Every rules layer that has a general
  section is re-read with the held modifier as its default, and the result is multiplied by 100
  (F4), even when the layer lacks the key. A map with a general section but no support-modifier
  key therefore multiplies the support contribution by 100 before F1's integer quantisation,
  it does not multiply the unassisted contribution by 100. Whether this case occurs on any particular map or in any particular game mode is not asserted here. *Fixed:* a layer without the key leaves
  the held value unchanged.
- **`quirks.stale_chargers_kept`** (default on). A building of a non-overpowerable type never
  removes chargers that stopped aiming at it (claim 12). Two such registrations keep it in
  operation's switch and house-power conditions through S-0405 claim 8 until the chargers leave play. Whether unmodified data can register a charger on such a building is not asserted here. (S-0308 row 11). *Fixed:* the same removal as for overpowerable types.
- The support reach is the master's slot-1 range. A master type with no slot-1 weapon has a reach
  of 0, so only a supporter at distance 0 qualifies. Normally placed buildings cannot share
  that position, so such a master fires alone after one processing.
- A master always fires slot 0. If weapon choice gave slot 1 (for example, an overpowered master
  of a type that is also overpowerable), the slot-0 weapon still fires (claim 7).
- A supporter's own weapons play no part. Its rearm comes from *R*ₛ and its beam from *L*ₛ, not
  from its weapons' fields.
- A vehicle with an electric-charge secondary goes through claim 14's scan and fires charge shots
  that do nothing (claim 11). Since it lacks claim 15's break-off search, it keeps firing at the
  building while enemies approach, until an order or another target change. The vehicle case is covered by the named test data in TC4. Quirk `quirks.vehicle_charge_no_break_off` (default on). *Fixed:* vehicles break
  off as infantry do.

- **Display bugs without switches.** Three beam-drawing faults of the original change only what
  is drawn, so this spec states the fixed behaviour and gives no quirk switch for them (see
  claims 18 and 21). A building's beam takes its look from its slot-0 weapon, a laser duration
  above 127 ticks shows no beam or a wrapped one, and sector 5's ordinary beam shows only one outer
  line.

## Import mapping

| Source key | Concept | Parse form | Absent |
|---|---|---|---|
| Rules, `[General]`, `PrismType` | prism type (Terms) | building type name | none: no prism support |
| Rules, `[General]`, `PrismSupportModifier` | support modifier *s* | real with optional percent sign, then F4 | held value (F4); the engine's starting held value is 100 |
| Rules, `[General]`, `PrismSupportMax` | support cap *N*ₘₐₓ | integer | held value; 8 |
| Rules, `[General]`, `PrismSupportDelay` | support rearm *R*ₛ, ticks | integer | held value; 100 |
| Rules, `[General]`, `PrismSupportDuration` | beam duration *L*ₛ, ticks | integer | held value; 15 |
| Rules, `[General]`, `PrismSupportHeight` | not used by these support rules | integer, leptons | held value; 300 |
| Rules, prism type's section, `Secondary` (that weapon's `Range`) | support reach (Terms) | as S-0307 claim 13 | no slot-1 weapon: reach 0 |
| Art, building image, `DelayedFireDelay` | delayed-fire field *d* (claims 6 and 7) | integer, ticks | 0 |
| Art, building image, `IsAnimDelayedFire` | not read for the prism type (claim 7) | boolean | no |
| Rules, warhead section, `ElectricAssault` | electric-charge warhead (claims 11 and 14) | boolean | no |
| Rules, building type section, `Overpowerable` | overpowerable type (claims 12 to 14) | boolean | no |
| Rules, `[Guard]` and `[Area Guard]`, `Rate` | rate delay after automatic charging starts (claim 14) | decimal game minutes, optional percent sign; S-0215 claim 24 or 26 | held value; engine default double nearest 0.016 game minutes |
| Rules, weapon section, `IsLaser` | the weapon's shot draws a beam (claim 18) | boolean | no |
| Rules, weapon section, `LaserInnerColor`, `LaserOuterColor`, `LaserOuterSpread` | inner, outer and spread colours (claims 18 and 21) | three integer colour channels, S-0003 | each (0, 0, 0) |
| Rules, weapon section, `IsHouseColor` | house-colour flag (claim 18) | boolean | no |
| Rules, weapon section, `LaserDuration` | beam duration *D*, ticks (claims 18 and 19) | integer | 10 |
| Rules, `[AudioVisual]`, `DetailMinFrameRateNormal` | activity-count floor *m*, ticks per publication window (claim 22) | integer | held value; 15 |
| Rules, `[AudioVisual]`, `DetailBufferZoneWidth` | activity-count margin *z*, ticks per publication window (claim 22) | integer | held value; 5 |
| Rules, `[AudioVisual]`, `DetailMinFrameRateMovie` | an alternative floor used in some situations not traced here (Open questions) | integer | held value; 20 |
| Settings file, detail level key (S-0104 Import mapping) | detail level (claims 21 and 22) | as S-0104 | as S-0104 |
| No data key | house laser colour (claim 17, from the house colour of S-0103), beam sector shifts, thresholds and the depth bias (claims 20 to 23) | fixed behaviour | not applicable |
| No data key | supporter acquisition tie order (claim 3), successful-scan delay selection (claim 14), break-off gate bypass and replacement (claim 15) | fixed behaviour, not an imported setting | not applicable |

The secondary weapon and its warhead are imported by S-0308 claims 1 to 3. Search candidate
properties, including the no-auto-fire restriction, retain S-0309 claim 5's import mapping.
Claim 15 changes which search permissions apply, not those properties' parse forms or defaults.

The modifier is a percentage field (S-0003, Terms and claim 7) scaled by F4. The invented written value
"125%" becomes the held value 125, while a bare "125" becomes 12500.

## Measurement scenarios

Use tiny purpose-made skirmish maps, game speed index 2, 800 × 600, no engine add-on and no mods (the named test files of a scenario excepted),
except the named test data below. Use one enemy bystander computer player outside the observed
area, following the glossary's void rule. The observed area is the test buildings and the cells
within their weapon and support reach. Repeat each case three times. Record ticks, targets,
health, firing and beams through video and a read-only recorder; count random draws with the
random-state recorder where specified. Recorder extensions are proposals, not assumed existing
bench features. Never write running game state. Named test data is removed after each case.
For every case, a different recorded result from the stated expectation falsifies that part.

- **PS1 (claims 1, 4, 7 and 9).** Give house A master *P*, enemy house B a stationary target *T*
  within its primary range, and *K* idle own-house supporters behind *P*, unable to attack *T*.
  Run *K* = 0, 1, 2 and cap + 1. Order *P* to attack; record first-shot tick, recruitment counts,
  projectile damage at detonation and target health. Expect count min(*K*, cap), recruitment
  one per consecutive *can fire* processing, and F1's area-damage amount. Predict health loss
  by applying S-0303 and S-0302 to that amount, not by treating F1 as health loss directly.
  Repeat with named test-map overrides giving invented damage/modifier values that expose
  integer rounding, signed damage and 32-bit product wrap, and an in-flight-loss projectile.
  The supported shot must scale its remaining damage, not its original launch damage.
  Compare a different building type with identical weapons: it fires ordinarily, without links.
- **PS2 (claim 2).** Start from PS1 with two candidates. Separately make one candidate attack
  another enemy, belong to an allied house, have a rearm countdown running from an earlier
  shot, have a beam pending, be drained by a stock draining aircraft, or be destroyed before
  recruitment. Expect only the other to count. In separate runs switch one off, recruit it
  during its build-up, or order its sale and recruit before its removal: expect both to count.
  In a separate run, use named test data to apply a pulse to the eligible candidate without changing its other eligibility conditions; record whether it is recruited and the recruitment tick. Expect it to be recruited; a failure to recruit falsifies the pulse-disabled eligibility rule.
  Record behaviour and recruitment ticks to distinguish current from merely queued sale.
- **PS3 (claim 2, reach).** A named test-map override sets the secondary's range to the measured
  F2 separation of two normally placed buildings, then to one lepton less. Expect support
  only in the first case. Repeat across different terrain heights and with a missing secondary:
  expect F2 to include height and no normally placed supporter to qualify without the secondary.
- **PS4 (claim 3).** Use a named test-map override with cap 1, two candidates at unequal F2
  distance, then at equal F2 distance. Swap build order and repeat with capture-order reversed.
  Expect nearest in the unequal case, earliest acquired in a tie; record which draws the beam.
  Create candidate A before B, let the observed house acquire B first and capture A later:
  expect B to support. Transfer B away and reacquire it while A remains owned: expect A instead.
  Include two ownership changes in one tick, reversing their event order while keeping object
  creation order unchanged; expect the first acquisition event to win. A transfer to the same
  owner must not change the winner. Log placements and ownership-change events with ticks and
  their within-tick order; a creation-number winner contrary to acquisition order falsifies claim 3.
- **PS5 (claims 4 to 7).** Use three candidates, and another prism building outside the master's
  reach but within one candidate's reach. Expect three direct beams, no chained beam, and the
  master starting its own delay in the fourth consecutive *can fire* processing. Vary placement
  order to put supporters before and after the master in the update order; record all beam and
  shot ticks and compare claim 6's formula. Read rearm remaining immediately after each beam:
  expect exactly the support rearm field, with no rearm draw. Count down from that event,
  rather than timing the next shot, which also needs attack processing and its own delay.
  Use named test art/rules files to vary delayed-fire time, disable the animation-delayed-fire
  flag, give the secondary a visibly different shot, and make weapon choice select it. Expect
  the master still to delay and fire slot 0. Test delay 0 with two masters processed before a
  candidate's turn: expect one beam but both masters counting it. Also test delay 0 and support
  rearm 0 with one master and one candidate: expect repeated recruitment on later ticks up to
  the cap, not a distinct-supporter limit. Observe beam end points,
  colour, duration and cross-beam pixel profile. Expect claim 6's end points and duration, the
  house-colour drawing of claim 22 with thickness parameter 3 (5 for a supported master's shot),
  and no damage, ammunition or burst use by support. Pixel details belong to LB1 and LB2.
- **PS6 (claims 8 and 10).** Recruit support, then separately destroy, sell or capture a supporter
  before the master's shot: expect the recruited bonus unchanged. Destroy or sell the master
  instead: expect surviving supporters to beam towards its stored point and rearm normally.
  Move the target out of range early enough for attack processing to drop it: expect count 0,
  not a retained bonus on the next attack. Repeat by destroying the target before the next
  attack processing: expect the no-target reset. For the retained-count case use named test art/rules files
  with a long delayed shot and a slowly turning turret. Move a target sideways but keep it in
  range, timing movement so the final delayed-shot check fails for facing. Keep the target
  until another shot succeeds. Expect retained count, no replacement beams for that count,
  and F1 scaling on the successful shot.
  A run where attack processing drops the target does not test retention.
- **PS7 (claim 16 and F4).** Compare named test maps with no general section, a general section
  lacking the modifier, and one explicitly giving an invented modifier. Repeat PS1 with one
  supporter and record area-damage input before target defences, choosing a target that survives.
  Expect respectively the held modifier *s*, 100 × *s*, and the explicitly parsed F4 result,
  then F1 in each case. Do not expect total damage to grow exactly 100-fold: its unassisted
  part is not rescaled. Include the invented percent examples of F4 to distinguish rounding.
- **TC1 (claims 11 to 13).** Own-house infantry charge an overpowerable building *C*. Test 0, 1,
  2 and 3 distinct chargers, both switched on and off, at full and low power. Record registrations,
  *C*'s weapon choice and health. Expect F3 and no ordinary damage from charge shots. Two
  chargers bypass switch and house-power gates, not the other operation gates. Repeated hits
  by one infantry unit must never count as two. Order one charger away, and separately kill it:
  expect target-change pruning in *C*'s next turn, but leave-play removal at once. Compare ticks
  just before and after *C*'s turn to distinguish a cached overpowered state from list length.
- **TC2 (claim 14).** Put an infantry charger without a target in *guard*, then *area guard*,
  next to own-house and allied overpowerable buildings. Expect only an own-house building to
  be chosen automatically. With two own-house candidates in different adjacent cells, expect
  the one first in step-direction order. Reverse directions and repeat. Give an attack order
  to an allied building: expect charging, with no special charge cursor. Put an enemy where
  area guard acquires it first: expect that target rather than charging in that processing.
  Record the processing that sets the charging target, its selected delay, and the next processing
  tick, with random state. Use named test-map overrides giving invented distinct rates for guard,
  area guard and attack. Expect claim 14's delay table using the old behaviour's rate, not a
  fixed 1-tick delay or the attack rate. Repeat guard while rearming, and with named test data
  enabling distributed fire and retaining candidates: expect the table's no-draw cases. Repeat
  area guard inside and outside S-0215 claim 26's close-range band to test truncation by 6.
  Earlier behaviour-start or forced-processing events invalidate a next-processing-interval test,
  but not a read-only observation of the selected delay.
- **TC3 (claim 15).** While infantry charges automatically, bring an eligible enemy into range.
  Expect replacement at the next attack processing, followed by list pruning at *C*'s next
  turn. Repeat after an explicit target-setting order to the same building: the mark must clear,
  so that later approach must not trigger this special break-off search. Record random state:
  expect ordinary behaviour-delay draws but no additional targeting-timer draw for the search.
  Repeat with no enemy: expect the same building and mark to remain, not an empty target.
  Introduce two eligible enemies at once: expect S-0309's ordinary ranking despite the retained
  usable charging target. In named test-map overrides, disable passive acquisition and
  opportunity fire separately on an otherwise armed charger: expect break-off still. Enable
  the no-auto-fire restriction on the human-owned charger: expect no break-off candidate.
  To distinguish unconditional mark clearing from an identity comparison, use named test data
  allowing allied candidates under S-0309's candidate rules and make the held building the sole
  eligible result. Record target and mark: expect the same target with the mark cleared, then
  no special break-off when an enemy arrives. Failure to realise that eligibility makes this
  sub-test inconclusive, not evidence for retaining the mark.
- **TC4 (claims 11, 12, 14 and 15, named test-map overrides).** Supply an infantry charge primary
  able to target non-overpowerable and enemy buildings, and a vehicle with a charge secondary.
  Expect infantry registration irrespective of building type or owner, no vehicle registration,
  and no ordinary area damage even when aimed at a non-building. Register two infantry units
  on a non-overpowerable building, then order them away: expect the registrations and operation
  gate bypass to persist until they leave play, but no overpowered state. The vehicle must
  automatically choose an own-house building and continue charging when an enemy approaches.
  Add configured charge-warhead debris/flash and a charge weapon's radiation field: expect
  those common effects despite the ordinary-area-damage bypass. Combine electric charge with
  mind control or charge planting separately: expect the higher-priority effect, not registration.
  Separately combine charging with parasite attachment or temporal erasure, using firers capable
  of the lower-priority effect: expect registration only on a building hit by infantry. Then
  deliver the same mixed-effect shots to a non-building, or from a non-infantry firer: expect
  no registration, parasite attachment, erasure or ordinary damage. Confirm delivery with a
  delayed projectile if ordinary firing legality otherwise prevents the test. A lower-priority
  effect or ordinary damage after failed charging falsifies claim 11's no-fallthrough rule.

- **Common setup for LB1 to LB8 (claims 17 to 23).** Beams are presentation: use tick-labelled
  lossless 16-bit captures with at least one drawn frame per tick, recording the detail level,
  the held activity count, publication clock, hysteresis state and house colours. The director
  is off. Log beam end points, parameters, terrain depth and light-alpha through a proposed
  read-only recorder extension; none of these observations may modify the game. Sample only pixels in explored ground with
  light-alpha 127, except where a case names shroud. Because claims 18 and 21 state fixed behaviour
  where the original has a display bug, the original is expected to show the bug in those cases.
  That result confirms the bug note and does not falsify the claim.
- **LB1 (claims 17 and 22, F5).** For each house colour available in skirmish, give one house a
  support beam over a flat dark tile, at detail level 2 and a frame rate well above the detail
  system's minimum. Read the centre-line pixel at *P* = 0 and the first two side pairs. Expect
  the centre line to be F5's *L* scaled by F6's *I* = 1 and added by F7, and each pair to halve.
  Repeat with *N* > 0 for the master's shot: expect the centre and first pair in 2*L* clamped,
  the second pair in *L*, then halving.
  Repeat with named test house-colour definitions producing a black display colour and an
  unequal-channel display colour: expect F5's white zero-input case and its cut-off/scaling
  result respectively. Record the actual display RGB as input, not the written colour triple.
  A centre colour that is not F5's prediction falsifies claim 17.
- **LB2 (claims 18, 21, 22 and 23).** At detail levels 0 and 2, fire one beam of each claim 18
  source along horizontal and vertical screen lines (sectors 1 and 3) and both diagonals (0 and 2).
  Count the lit pixels across the beam and their colours. Expect: 2*W* + 1 pixels across, or
  fewer where halving stops, in fine mode, opaque full colours at detail 0 for ordinary beams,
  and the pair shifts of claim 22. Use named test data giving a non-prism building and a vehicle the
  same house-colour laser weapon: expect *W* = 1 and *W* = 2. Give a building a laser weapon only
  in slot 1 with distinct colours: the original is expected to show slot 0's look (bug note).
- **LB3 (claims 18 and 19, F6).** Fire one beam with an invented duration of 4 ticks, then 1, 0
  and 200, in named test data. Expect drawing in exactly *D* ticks from the firing tick, centre
  intensity following F6 down to 1/*D*, none for 0, and for 200 the original's missing beam (bug
  note). Blink is not tested: no known data-only source exposes that flag.
- **LB4 (claims 20, 21 and 23).** Fire an ordinary laser with a non-black outer colour across a
  cliff, so that part of the line lies behind higher terrain, and with a unit standing on the line.
  Expect terrain to hide the beam pixels behind it and the unit never to hide them. Repeat across a
  shroud edge: at detail 0 the beam stays visible over shroud; at detail 2 it vanishes where
  light-alpha is 0. Repeat in each of the eight sectors: expect two outer lines everywhere, with
  sector 5 showing one outer line in the original (bug note).
- **LB5 (claims 6, 18, 20 and 22).** Supporters and master on different terrain heights: record
  both end pixels of each support beam and master shot. Expect the fire points of claim 6 and
  claim 20, not object centres, and a beam that stays put when its master is destroyed.
  Force coarse mode with a named test-rules override that sets the frame-rate floor far above the
  achievable rate, and compare against fine mode: expect opaque lines, more distant pairs dropped by the 64
  threshold, and colours moving toward white as claim 22's coarse step predicts. For example,
  an invented channel *c* = 100 at *I* = 1 becomes 254 before screen-colour reduction, not 255.
  Then use an invented floor of 20 and margin of 5, changing the lobby speed through normal
  controls to bring the recorded activity count below, between and above those values. Expect
  a clear hysteresis state to fail below 20, a set state still to fail at 24, and a set state to
  pass and clear at 25. Compare each beam's mode with the state at its test, not just the previous
  beam: another visual effect can change the shared state. Log at least ten publication windows
  and tick starts: every published *f* must equal the integer number of ticks since the previous
  publication, held unchanged until the first cycle-end check at or after 60 clock units.
  With delayed checks it remains the raw count, not a per-second quotient. A count normalised
  to 1000 ms, a moving average, a mid-window change, a 24-count recovery or a 25-count failure
  (without an intervening shared-state change) falsifies the respective rule. This fixture uses
  named rules data only for the floor and margin; host timing is recorded, never fixed by a write.
- **LB6 (claims 21 and 23, F7 and F8).** Fire an ordinary laser with a non-zero spread at detail
  level 2 and record the unsynchronised random stream's state before and after each drawn frame:
  expect exactly three draws per beam per drawn frame when outer lines exist and none otherwise,
  and no change in the scenario random stream. Over a flat mid-grey tile, read the centre and outer
  pixels and compare with F7, including a colour whose red channel overflows (spill). Fire a beam
  with a black outer colour and compare its two-pixel anti-aliased profile with F8. Use named
  test data with inner colour (129, 129, 129), a black test terrain tile, duration 5 ticks and
  no outer colour. At *P* = 2, *I* is the single-precision value of 3/5. An axis-aligned interior
  sample has coverage 256 and weight 153, not 154. With light-alpha 127 its widened result
  before packing is (63, 63, 63), giving (56, 60, 56) in 5-6-5 colour. A result of (64, 64, 64)
  distinguishes and falsifies nearest-integer weight conversion. A slope of one half gives the
  coverage sequence in F8; capture enough pixels to include (128, 128) and (0, 256). Any other
  profile falsifies the asserted coverage rule within its stated domain. Use only unclipped
  lines with a recorded bias difference at most their long-axis length. Repeat the blended
  case over full shroud: expect the pre-beam pixel unchanged wherever light-alpha is 0.
  At a shroud boundary, record both pixels' alpha values: the pair must use the first pixel's
  value for both outputs, and leave both unchanged when that first value is 0.
- **LB7 (claim 20, F9).** With named test weapon and art data, arrange unclipped opaque and
  additive lines with projected end differences (8, 0) pixels and recorded bias differences
  +3, −3 and +4 depth units. Use a named test terrain tile with a black picture and per-pixel
  depth data that exercise equality and differences of one depth unit against the predicted
  sample depths. Record end coordinates, biases, stored terrain depths and final pixels.
  For +3 expect bias displacements 0, 0, 1, 1, 1, 2, 2, 3 at sample indices 0 to 7; for −3
  their negatives; for +4 expect 0, 0, 1, 1, 2, 2, 3, 3. In each case a sample passes only when
  the stored depth is larger, never on equality. Reverse source and target while keeping the
  screen end points and biases identical: expect the same left-to-right samples. Any different
  displacement or equality result falsifies F9 or its strict test. Keep both ends inside the
  view; clipping and blended-line interior depths are not predictions of F9. Repeat with
  projected differences (0, 8), then with (2, 0) and a bias difference of +4. The last case
  has four samples, screen displacements 0, 0, 1, 1 and bias displacements 0, 1, 2, 3.
  Record pixels to test repeated additions at a shared screen coordinate. A zero-length
  triple must contribute nothing, and an isolated terminal end must receive no sample.
- **LB8 (claims 21 and 22, overlap).** Use named test weapons with distinct inner and outer
  colours and zero spread, over a mid-grey test tile, in sectors and slopes whose shifted lines
  share pixels. For ordinary beams at detail 0, expect the second outer colour wherever both
  outer lines pass at the same pixel; either outer colour wins over a passing centre sample.
  Only test pixels with an actual overlap of the recorded paths. At detail 2 compare the observed 16-bit pixel with
  *C*(*O*₂(*O*₁(*d*))) from F7, packing after each contribution; choose colours for which
  reversing two contributions or packing only once gives a different prediction. For a
  house-colour beam repeat in coarse and fine modes: expect claim 22's opaque winner and
  composed F7 result respectively. Log the background pixel and light-alpha so the result is
  numerical, not a judgement of apparent brightness. A matching reversed-composition result
  that differs from the specified one falsifies the overlap rule. The blink-only blended
  overlap has no known data-only fixture and remains unmeasured as in claim 19.

## Cross-spec changes to hand off together

- S-0102 claim 2 needs the beam depth-test qualification of claim 20 here, rather than implying
  that drawing after all layers makes beams visible through the cell pass's nearer pixels.
- S-0103 claim 15 should identify its display colour as the input to claim 17 here.
- S-0615 claims 15 and 16 should use claims 17 to 23 here for beam drawing. Its disc beams use
  ordinary drawing even when their colours come from the house. Its final beam's last drawn
  intensity is 1/*D*, not 0; an arc's half-intensity end parameter is likewise not reached.
- The glossary should point its beam and house laser colour vocabulary to this spec.

## Open questions

- **Beam corner cases.** Which sector a direction exactly half-way between two sectors falls into,
  blended-line interior depth steps; coverage and depth sampling after view-edge clipping and
  coverage when the bias difference exceeds the projected long-axis length; and when the
  alternative movie floor replaces the normal one (stock skirmish play is expected to use the
  normal floor). F8 and F9 deliberately do not assert those cases.
  The original also contains a hardware-accelerated beam drawing that the stock game never
  enables; it is not specified here.

- How the support multiplier affects detonation side effects other than ordinary area damage.
  No general rule about those effects is claimed here.
- The exact place of claim 14's scan within the *guard* processing of infantry with special guard
  states. It runs only when nothing earlier in that processing has taken over.
- Prism-type combinations with the SAM flag or installed upgrades. Those can enter special
  attack paths instead of recruitment. Stock skirmish prism types do not need those combinations,
  and their complete interaction remains untraced here.
