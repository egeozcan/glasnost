---
id: S-0316
title: Electric-bolt shots (the shot, its sparks, and how the bolt is drawn)
topic: combat
revision: 27
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-23
measurements: []
requests: [Q-1701]
supersedes: []
---

# S-0316: Electric-bolt shots (the shot, its sparks, and how the bolt is drawn)

## Summary

A weapon with the electric-bolt flag fires an ordinary shot: it launches its projectile like any
other weapon, and the projectile alone decides where and when the warhead detonates. What the flag
adds is a drawn effect, the *bolt*, and one spark system at the bolt's end (in our engine only when
the game-wide spark type is present). A firing that launches no projectile, such as a spawner
weapon's shot, makes neither. This spec owns when a
bolt is made, where its ends are, which end can follow its firer, how long it lasts, its jagged
shape, its colours and how its lines meet the screen. It corrects the glossary and S-0616, which
describe the shot as fired instead of a projectile.

## Terms

- **Bolt.** The drawn effect of one electric-bolt shot, or of one shrapnel fragment's launch
  (claim 8): three jagged lines between a *start* and an *end*. A fragment's launch is not a shot
  in claim 2's sense: it makes its bolt by claim 8's rule, and the firing rules of S-0301 do not
  apply to it. The bolt itself changes nothing in play: its own draws come from the
  unsynchronised random stream (S-0202 claim 11). Its spark system (claim 6) does change play:
  it is made with every bolt, whether or not the bolt is ever drawn, and its turns take draws
  from the scenario random stream under S-0616 claim 2.
- **Bolt start, bolt end.** The start is the firer's fire point when the bolt is made (claim 3),
  or, for the bolt of a shrapnel fragment, the point where the fragments burst (claim 8).
  The end is fixed when the bolt is made: the target's target point when the target is an object
  (the sense used by beams, glossary *target point*), or the target cell's centre at its height
  otherwise. The bolt end uses the target-point rule of S-0518 claim 20. The target here
  is the shot target (S-0306 Terms), not the firer's target: a redirected shot, such as a spray
  attack's (S-0313 claim 9) or an area-fire weapon's (the firer's own cell, S-0306 claim 3 a),
  ends at the cell it is redirected to (claim 3).
- **Alternate-colour flag.** A per-weapon flag, imported from the user's weapon data (Import
  mapping), that only selects the colour of a bolt's two side strands (claim 10). It has no other
  effect.
- **Attached bolt.** A bolt whose start follows its firer's fire point (claim 4). At most one bolt
  is attached to a vehicle at a time.
- **Bolt phase.** An integer that each bolt carries. It is drawn when the bolt is made (*φ*) and
  goes up by 1 after each drawn frame of the bolt's life, drawn or not, so drawn frame *n* of the
  life (*n* = 0 to 16) uses *φ* + *n*. It sets the slow sway of the bolt's middle (F2).
- **Bolt length *λ*.** In one drawn frame, the distance between the bolt's start and end in
  leptons: *λ* = trunc(√̃(Δ*x*² + Δ*y*² + Δ*z*²)), with √̃ the table square root (S-0204 claim 5
  and F1, with that formula's rounding), under that claim's switch `quirks.table_root_distance`,
  which covers every use of the table square root that measures a distance. The sum of squares is formed in floating point from the
  integer coordinate differences, where it is exact and does not wrap for a sum below 2⁵³ square
  leptons (S-0204 F2). So, for such a sum, *λ*, and with it the
  levels and the count of draws (F1, claim 13), are fixed exactly by this formula. The table root
  falls short of the exact root by at most one lepton for lengths below about 90 cells (S-0204
  claim 5), and, with S-0205's `quirks.chop_rounding` on, gives a different number of levels only
  when ⌊√(Δ*x*² + Δ*y*² + Δ*z*²)⌋ is exactly one of the bounds 65 × 2^*d* of F1 and the table root
  falls short there. With that switch off, F1's rounding of the sum can also carry the table root
  above the exact root and across a bound (S-0204 F1). Claim 13 rates these cases
  and the height term Δ*z*² separately. The other floating-point values here (the sway of F2) only
  shape the drawn bolt, never game state or the count of draws, so their order and rounding are not
  specified: any double-precision evaluation conforms, including one that moves a point by a pixel
  at a truncation boundary.
- **Strands A, B and C.** The three lines of a bolt. A is the main strand, and B and C follow it
  at the coarse scale (F3).

## Behaviour

### The shot

1. **An electric-bolt weapon launches its projectile.** Claims 1 to 4 are about shots that launch
   a projectile under S-0301 (last paragraph of this claim for the firings that launch none),
   claim 8 about the launch of shrapnel fragments, which is not a shot, and claims 5 to 7 and 9
   to 13 about every bolt, whichever of the two made it. The
   electric-bolt flag does not replace the projectile. The shot is made by the firing cycle of
   S-0301 and launches the weapon's projectile as S-0306 says. That projectile decides where and
   when the warhead detonates, whether the shot can miss, and what a wall or cliff on the way does
   to it. So an electric-bolt shot can miss exactly when its projectile can, by the projectile's
   own rules (for example the scatter of a projectile that is inaccurate and arcing, S-0306 claim
   4), and the flag adds no hit or miss rule of its own; the bolt still ends at the bolt end (claim
   3). Range, minimum range and the line-of-fire rules are the weapon's ordinary ones (S-0301,
   S-0307, and S-0306 claim 22's walk), because the flag plays no part in deciding whether or at
   what a weapon fires. Only the bolt and its sparks come from the flag.

   A firing that launches no projectile makes no bolt and no sparks, even when its weapon has the
   electric-bolt flag. Four such firings are a spawner weapon's shot, a drain attempt, the firing
   of a disc laser weapon and the firing of a suicide weapon. These firings launch no projectile
   and so make no bolt. A drop attempt is a fifth: S-0522 claim 11 gives it no projectile (no run
   here tests it: low, below). The flag adds nothing to
   these firings. The aircraft a spawner weapon spawns fire
   their own weapons, which make bolts only if they have the flag themselves. A shot made void by
   S-0301 claim 15 makes none either (claim 2).
   - Backing: `measurable` (scenario EB1 for the projectile weapon, its parts e and e′ for range
     and minimum range, part j for the line of fire and part k for the miss; EB9 parts a to d for
     the spawner, draining, disc laser and suicide weapons); `unmeasured` for the drop attempt,
     since no scenario here arranges a carrying aircraft whose dropping weapon has the flag
   - Confidence: medium until EB1 runs (the projectile weapon, range and minimum range), since this
     corrects the glossary entry and S-0616 Terms on a scenario not yet run; medium (the line of fire
     and the miss, which EB1 parts j and k compare with a control weapon, and the four firings
     without a projectile that EB9 tests, all until they are run); low (the drop attempt, which
     rests on S-0522 claim 11's absence of a projectile and has no run)

2. **When a bolt is made.** Each shot that launches a projectile, of a weapon with the
   electric-bolt flag, makes one bolt, in the tick of the shot. A shot that S-0301 claim 15 makes
   void (its projectile cannot be put on the map) makes no bolt and no sparks, so every bolt that a
   shot makes belongs to a shot whose projectile was put on the map (a fragment's bolt is claim
   8's). Each shot of a burst makes its own bolt and its own spark system (S-0301 claim 4). Of a
   weapon's line-effect flags, the laser flag wins over the electric-bolt flag, and the
   electric-bolt flag wins over the radiation-beam flag (S-0514 claim 23), so a weapon that also
   has the laser flag draws a laser and makes no bolt and no bolt sparks (the sparks of claim 6
   come only with a bolt). The electric-bolt flag also wins over the magnetic-beam flag. This part
   of the ranking is this claim's own; no other spec gives it. A weapon with the
   electric-bolt flag and the magnetic-beam flag makes the bolt and no magnetic wave, even when its
   firer has no live wave, and the shot itself is still fired, as S-0607 claim 21 says of a shot
   that makes no wave (Changes for other specs).
   The particle-effect flags of S-0301 claim 5 (fire, spark and railgun) and the sonic flag are
   outside this ranking: a weapon with the electric-bolt flag and any of them makes those effects
   as well, by their own rules. The single wave slot that a sonic wave and a magnetic wave share
   (S-0301 claim 5) is S-0301's rule and does not bring the sonic flag into the ranking.
   - An electric-bolt shot adds no delay of its own. The next shot waits only by S-0301's rules:
     a bolt still being drawn is not one of the effects whose life S-0301 claim 5 makes a
     weapon wait for.
   - Backing: `measurable` (scenarios EB1 and EB3; EB1 part i for a void shot, and EB1 part l for
     the magnetic-beam flag). `unmeasured` for the independent flags and
     the absence of a delay: no scenario gives the test weapon those flags or times its shots
   - Confidence: high; medium for the void shot, as S-0301 claim 15 rates failed launches; medium
     for the magnetic-beam flag, until EB1 part l is run; low for the
     independent flags and the absence of a delay

3. **Its ends.** The bolt runs from the firer's fire point (the fire point of S-0313, claims 1 to 8) to the bolt end
   (Terms). The fire point is the one for the weapon slot that fired the shot. For a redirected
   shot the shot target is the cell it is redirected to: the cell of S-0313 claim 9 for a spray
   attack, and the firer's own cell for a weapon with the area-fire flag (S-0306 claim 3 a). So
   the bolt of an area-fire shot runs down from the fire point to the centre of the firer's own
   cell, at the terrain height there. The end is fixed when the bolt is made
   and never moves: the bolt does not follow its target, and it is not shortened by walls, cliffs
   or buildings between the ends. If the projectile detonates somewhere else (it is stopped on the
   way, it scatters, or it is aimed with lead), the bolt and its sparks still end at the bolt end.
   - A garrisoned building's bolt starts at the muzzle point that the shot was fired from
     (S-0313 claim 6), in our engine (design). Edge cases records the original's drawing glitch
     here.
   - Within a burst, the side of the fire point (S-0313 claim 2) is that of the shot's own
     projectile, both when the bolt is made and, for an attached bolt (claim 4), in its later
     drawn frames, in our engine (design). Edge cases records the original's drawing glitch
     here.
   - Backing: `measurable` (scenario EB1 part g for the fixed end, parts h and h′ for the shot
     target of a spray attack and of an area-fire weapon, EB5 for walls, cliffs and buildings,
     and every scenario's fire point is that of the slot that fired, which with a single-slot
     firer does not tell slots apart). The original's slot in shots
     where another slot could be chosen is `unmeasured` (Edge cases, Open questions)
   - Confidence: high (fixed end, no shortening, the shot target); medium for the end of an
     area-fire shot, which follows S-0306 claim 3 a's redirected cell, rated medium there, until
     EB1 part h′ runs; medium for the slot that fired in ordinary firing, since no scenario
     here fires from a second slot. The garrisoned
     building's start and the side within a burst are design rules of our engine; the original's
     sides and its garrisoned building's start are `unmeasured` and rated low, since no scenario
     here isolates the original's side within a burst or its garrisoned building's start

4. **Which bolts follow their firer.** When the firer is a vehicle on the map that has no attached
   bolt, the new bolt becomes attached to it. A vehicle carried by a transport is off the map, also
   when it fires from an open-topped transport (glossary, S-0507 claim 14), so its bolts are never
   attached: each starts at the fire point the vehicle has at the shot, and keeps that start while
   the transport moves on. S-0313 claim 5's transport muzzle rule is for infantry passengers only.
   A vehicle passenger's fire point is S-0313 claim 1's, from its own position, facings, weapon
   slot and burst counter; its position is the transport's (S-0507 claim 15), so the bolt starts
   from the vehicle's own fire point placed at the transport's position (Changes for other specs).
   In each drawn frame of the bolt's life, an attached bolt's start is moved to the vehicle's
   current fire point for the same weapon slot, before the view test of claim 5 and whether or
   not the bolt is then drawn; the view test uses the moved start. So a bolt that comes back into
   view after a scroll is drawn from the fire point the vehicle has then. A bolt made while the vehicle already has one
   attached is not attached: its start stays where it was made. A bolt fired by infantry, an
   aircraft or a building is never attached. The attachment ends when the bolt ends (claim 5), or
   when the vehicle is removed (glossary: it no longer exists). A bolt whose vehicle is removed
   keeps the last start it had. Being destroyed does not end the attachment by itself: a destroyed
   vehicle that stays in play for a while, such as one that sinks, keeps its bolt attached, and the
   start follows its fire point until it is removed. Going off the map does not end it either:
   while the vehicle is off the map but not removed, for example after it enters a transport during
   the bolt's life, the start keeps being moved to the fire point that the vehicle's current
   position gives. In an open-topped transport that position is the transport's (S-0507 claim 15).
   In a closed transport no spec moves a carried vehicle's position, so in our engine (design) the
   position stays where the vehicle went off the map and the start stays at the fire point from
   there; the original's result there is open (Open questions). So a carried vehicle's new bolt
   never follows it, while a bolt attached before its vehicle leaves the map keeps following it.
   Our engine uses this rule (design); the original's result is rated low, since no run moves an
   attached vehicle off the map.
   - Backing: `measurable` (scenarios EB2 and EB3; EB2's aircraft run for aircraft, its carried
     run for a carried vehicle's bolt not following it). `unmeasured` for a destroyed vehicle
     that stays in play (no run destroys such a vehicle during its bolt's life), for an attached
     vehicle that goes off the map (no run moves an attached vehicle off the map during its
     bolt's life), and for a carried vehicle's fire point (EB2's carried run checks only that
     the start does not follow, not where it is), and for the start's move in a drawn frame in
     which the bolt is not drawn (EB4's scroll run fires from a firer that does not move, so the
     moved and unmoved starts coincide there; no run moves an attached vehicle while its bolt is
     out of view)
   - Confidence: medium for the start's move in a frame in which the bolt is not drawn
     (`unmeasured`, as above). High (vehicles only, one attached bolt per vehicle, the end never follows), medium
     (the vehicle's removal, a carried vehicle's bolt not following it), low (a carried vehicle's
     fire point, which EB2's carried run does not locate, a destroyed vehicle that stays in play,
     and an attached vehicle that goes off the map). If EB2's carried run is
     reported unreached, the carried vehicle is `unmeasured` and rated low

5. **How long a bolt lasts.** A bolt is drawn in 17 consecutive drawn frames, counting the first
   drawn frame after the shot or fragment launch that made it, and is gone from the 18th. A
   cycle's drawn frame shows the state of the previous tick (S-0201 claim 6), so the tick of that
   shot or launch comes after its own cycle's drawn frame
   and that first drawn frame is the next cycle's. Its life is
   counted in drawn frames, not ticks: in skirmish it is 17 ticks, and during a freeze it still
   burns down. A bolt whose straight line from start to end is wholly outside the battlefield view
   in a drawn frame is not drawn in that frame, but that frame still counts towards its life and
   its phase still goes up. The test is made on the straight line between the screen positions of
   the two ends, not on the drawn strands, so a bolt whose straight line lies just outside the view
   is not drawn at all, even where its upward arc (claim 9) would reach into the view. A
   stand-still makes no drawn frame (S-0201 claim 6), so it does not use up a bolt's life.
   - Backing: `measurable` (scenario EB4; its focus check for the stand-still; its scroll run for the frames out of view, which use
     up the life and raise the phase, and its edge run for the test on the straight line; both
     runs with EB8's sound isolation; its freeze run for the first drawn frame being the first
     one after the shot's tick: with the freeze starting in the next tick, one of the bolt's
     drawn frames comes before the freeze and 16 during it, while a bolt that first showed one
     drawn frame later would show all 17 during it. Which cycle that frame is labelled with
     follows from S-0201 claim 6 and is not a separate observable)
   - Confidence: high (17 drawn frames; the frames out of view counting towards the life); medium
     (the test on the straight line rather than the drawn strands, until EB4's edge run; the frames
     out of view raising the phase, which EB4's scroll run checks only through F2,
     itself medium; that the first drawn frame after the shot's tick is the bolt's first, until
     EB4's freeze run)

6. **Its sparks.** Each bolt comes with one system of the game-wide spark type at the bolt end,
   made with the bolt (S-0616 claim 1), whether or not the bolt is in view or ever drawn
   (claim 5). The sparks do not depend on the shot's outcome. A shot
   fired at the ground has them at the target cell's centre. A shot that kills its target has them
   at the target's point. A shot whose projectile is stopped on the way has them at the bolt end,
   not where the projectile stopped.
   - Backing: `measurable` (scenarios EB1 and EB5) for the bolts of shots; for the bolts of
     shrapnel fragments, claim 8's backing
   - Confidence: high for the bolts of shots; for the bolts of shrapnel fragments, claim 8's
     confidence
   - **Absent spark type, in our engine.** When the game-wide spark type is absent, our engine
     makes no spark system and the importer warns. This is S-0616 claim 1's departure for an
     absent key. The original's result in that case is not established (S-0616 Open questions).
     Design, outside this claim's rating and backing.

7. **No sound of its own.** A bolt plays no sound of its own, and neither do its sparks: no spec
   gives a spark system or its particles a sound. The sounds an electric-bolt shot can bring are
   the damage and death sounds of what it hits (S-0701 claim 20), the weapon's report (S-0701 claim 21) and the start and stop sounds of the animations the shot
   causes, such as its shot animation and the explosion animation its impact creates (S-0701
   claim 22; S-0109 claim 13). A warhead has no sound of its own (S-0701 claim 22).
   - Backing: `measurable` (scenario EB1, with a test weapon whose report is absent)
   - Confidence: high

8. **Shrapnel fragments.** A projectile with a shrapnel weapon launches fragments (S-0306 claim
   21). When that shrapnel weapon has the electric-bolt flag, each fragment launched before the
   projectile's firer has gone also makes a bolt with its spark system. That bolt runs from the
   point where the fragments burst to the fragment's target, with the end rule of the Terms, and
   is never attached. When fragments are launched, how many, and which target each is aimed at
   belong to the fragment rules, which have no owner yet (S-0306 Open questions). So this claim
   fixes that each fragment launched before the firer has gone makes one bolt, that it starts at
   the burst point and ends at its fragment's target, and that it has sparks. Only the choice of
   that target is left to the fragment rules. Which weapon's alternate-colour flag gives the
   bolt its colour, and whether a fragment launched after the projectile's firer has gone still
   makes a bolt, are open (Open questions).
   For depth (F4), the projectile's firer counts as the bolt's firer: when it is a building, the
   fragment's bolt takes the building-source adjustment *a* of S-0518 claim 20, with the burst
   point in place of the fire point, and *a* = 0 for any other firer.
   - Backing: `measurable` (scenario EB6 part a) for one bolt per fragment from the burst point.
     `unmeasured` for never being attached: EB6 does not make the firer a vehicle, the only
     firer whose bolt claim 4 attaches. The building-source adjustment of a fragment's bolt is
     `unmeasured`: EB6's firer is not a building, and the adjustment shows only in which
     images hide the bolt near its start.
   - Confidence: medium (one bolt per fragment from the burst point, with sparks); low (never
     being attached, `unmeasured`); medium (the building-source adjustment, `unmeasured`)

### How it is drawn

9. **Shape.** In each drawn frame the bolt is drawn afresh from its current start, its end, its
   phase and new draws, so its shape is recomputed in every drawn frame. Three strands run from
   the start to the end, made by repeated midpoint displacement (F3). Each piece is split at its
   midpoint until its length counter (F1) is at most 64 leptons, or it has been split 8 times. Each
   split moves the midpoint by a random amount whose jitter parameter halves with every level,
   starting at about 9 % of the bolt length (F3 gives each strand's displacement from it). Strand
   A's own displacement is a draw from −*J* to *J* while the piece's length counter is above 128,
   but −2*J*, 0 or 2*J* where the counter is 128 or less, so its amplitude doubles relative to *J* at that boundary
   although the jitter parameter *J* still halves (F3). The
   first split also shifts the bolt's middle in the ground plane by an amount that sways smoothly
   with the phase, and lifts it by between none and twice that 9 %. At
   levels 0 and 1, where they are split (only level 0 when *D* = 1), strands B and C stay within
   the displacement that F3 gives them from strand A, ⌊R(−*J*ₖ, *J*ₖ) / 2⌋ per axis. Below that
   they jitter on their own, so the three strands coincide at the ends, run close at large scale
   and fray apart in the fine detail. A bolt of length 64 or less is not split: its three strands
   are one straight line, and strand A, drawn last, is the colour that shows. A bolt of length 0 is
   not drawn (unmeasured: no scenario here makes a bolt whose start and end coincide).
   - Backing: `measurable` (scenario EB4 for the three strands, the straight short bolt, which may
     stay unreached, and the upward arc; EB8, with its bolts of several lengths, for the 64-lepton
     limit, the 8 levels, the halving and the range of each level's draws). Strand A's factor 2*J*
     at the splits whose length counter is 128 or less is `unmeasured`: EB8 records the range of
     the draw R(−1, 1), not the displacement it makes, and EB4's capture checks strand A only at
     the first split. Of F2 and F3, a capture tests one thing: strand A's midpoint at the first
     split lies within two pixels of where F2 and F3 put it, with the recorded draws assigned to
     the strands and axes in any order that claim 13 allows (EB4's scroll run). That is the
     conformance test for the sway. Strands B and C within F3's displacement from strand A,
     ⌊R(−*J*ₖ, *J*ₖ) / 2⌋ per axis, at levels 0 and 1, are `unmeasured`: no scenario here
     captures them. F2 beyond that midpoint, the pinning threshold *P* and which strands are
     pinned at which level are `unmeasured` too. Our engine follows them as written (design), and
     conformance does not test them: they only shape the drawing, the random-state recorder shows
     the range of each draw, which is the same whichever way a strand is pinned, and the
     sway takes no draws.
   - Confidence: high (three strands, 64-lepton limit, 8 levels, halving; the 64-lepton limit
     medium if EB8 reports its bound test unreached); medium (the straight
     short bolt, which EB4 tries only where test data can arrange it; low if EB4 reports it
     unreached); medium (the amplitude factor 23/256, which EB8 brackets but does not pin down,
     and the sway as far as EB4's capture checks it); low (strand A's factor 2*J* at the splits
     whose length counter is 128 or less, strands B and C at levels 0 and 1, F2 beyond
     the captured midpoint, *P* and the pinning, all `unmeasured`, and a bolt of length 0, which
     no run reaches)

10. **Colours.** The colours are fixed entries of the general palette (S-0306 Terms), in its
    neutral shade row 26 of 53, not house colours and not a colour stored in the weapon. Strands B
    and C use entry 10, or entry 5 when the weapon has the alternate-colour flag. The main strand A
    always uses entry 15. Which colour each entry shows comes from the user's palette file. For a shot's bolt the
    flag is read from the weapon of the slot that fired (claim 3); for a fragment's bolt (claim 8),
    which weapon's flag is read is open (Open questions). The general palette is never tinted, so
    neither the cell's lighting nor coloured light changes which colour an entry gives. The one
    light that acts on a bolt is the pixel's light-alpha value of claim 11, applied after the entry
    is chosen.
    - Backing: `measurable` (scenario EB4, with its colour runs for a second house and the
      alternate-colour flag) for the entries. `unmeasured` for the cell's lighting
      and coloured light, which EB4 does not vary; that the general palette is never tinted is
      S-0306's (Terms)
    - Confidence: high for the entries, read from the slot that fired as claim 3 rates it; low
      for the lighting and coloured light (both `unmeasured`)

11. **How its lines meet the screen.** Each piece of each strand is a solid line one pixel wide
    between the screen positions of its two points (S-0101 claim 5, each with its own height lift),
    clipped to the battlefield view. A piece's pixels follow the per-pixel rules of S-0514 claim 25
    (not drawn at light-alpha 0, the depth test without a depth write, and the light-alpha scaling
    with its cap at 255) and its depth rule F8, with the depth biases of F4 here in place of F8's
    two biases. That claim's colour factor and its colour without a palette do not apply: a bolt's
    colours are the palette entries of claim 10. An entry becomes a pixel colour through the
    general palette's shade row 26 of 53, the neutral row of S-0104 claim 15. That table is built
    as S-0104 claim 6's untinted table (row *j* is the palette times 2*j*/52, no tint), so row 26
    reproduces the palette file's colour exactly, in the packed 16-bit pixel form of S-0514 claim
    25. The light-alpha scaling then acts on that colour exactly as S-0514 claim 25 says for its
    line kind, which owns the order of the steps and the width of each channel. So a bolt reads
    depth and never writes it, it is not drawn under full shroud, and a light-alpha value (S-0104
    claim 15: the pixel's value after the shroud pieces and any light-glow images covering it
    have acted) other than 127 changes its colour by that scaling. Terrain, cliffs, overlays, building
    images and non-animated terrain objects can hide parts of it, but vehicles, infantry, aircraft
    and other object-pass drawings never do. For a building firer, F4 brings the start nearer, so
    the building's own image does not hide the start of its bolt.
    - Backing: `measurable` (scenario EB7) for cliffs and building images hiding a bolt, vehicles
      not hiding it, and shroud, and EB7 part d for a building firer's own image not hiding the
      start of its bolt. `unmeasured` for overlays, non-animated terrain objects,
      infantry, aircraft and other object-pass drawings: EB7 includes none of them, and they rest
      on the line kind of S-0514 claim 25. The absence of a
      depth write is `unmeasured`: EB7 shows what hides a bolt, not what a bolt would hide from a
      later drawing that reads depth, and no such later reader is isolated. F4's exact depth biases
      and midpoint depths are `unmeasured`: EB7 shows what hides a bolt, not the depth values that
      decide it at a boundary pixel. Conformance asks only what EB7 shows. Beyond that, F4's biases
      and its midpoint depth term are `unmeasured`, and our engine follows F4 as written (design).
    - Confidence: medium (the same line kind as the beam flash, what EB7 shows of the depth
      biases, and the absence of a depth write, which follows S-0514 claim 25's rule for that line
      kind); low (F4 beyond what EB7 shows, `unmeasured`)

12. **Draw order.** Bolts are drawn in the effects group after all five layers (S-0102 claim 2),
    after lasers and before line trails and beam flashes. Among bolts, the most recently made is
    drawn first, so an older bolt appears over a newer one where they cross. Within each piece,
    strand A covers strands B and C wherever their pixels overlap and pass depth. Where two
    pieces of one bolt cross, the piece nearer the bolt's end covers the one nearer its start:
    the pieces are drawn in order from the start towards the end, each with its three strands
    (B, then C, then A) before the next piece. So where strand B or C of a piece crosses strand A
    of a piece nearer the start, B's or C's colour shows.
    - Backing: `measurable` (scenario EB4; S-0102 scenario D13 part b for the effect group; EB4's
      crossing captures for the order of pieces). The order of pieces may stay unmeasured: if no
      capture shows two pieces of one bolt crossing, EB4 reports it unreached.
    - Confidence: high (the effect group); medium (the order among bolts and among strands); low
      (the order among the pieces of one bolt, including crossings between strands of different
      pieces, until EB4 reaches a crossing)

13. **Random draws.** All of a bolt's draws are ranged draws (S-0202 claim 4) from the
    unsynchronised random stream (S-0202 claim 11), and none reaches game state. Making the bolt
    takes one draw, the phase *φ* = R(0, 256), after any choice the shot's report sound makes
    (S-0701 claim 21: an ordinary shot's report takes no draw; a shot fired by occupants takes
    one); that place is design (below).
    Each drawn frame in which the bolt is drawn takes 9 draws per split: 3 for strand A (one per
    world axis), then 3 for each of B and C. With *D* the number of split levels (F3), that is
    9 × (2^*D* − 1) draws per drawn frame. The splits take their draws depth first, parent before
    children. Our engine takes B's draws before C's, the axes in the order *x*, *y*, *z*, and the
    half towards the start before the half towards the end; these three orders are design, and
    any of them conforms (backing below). A drawn frame
    in which the bolt is not drawn (claim 5) takes no draws for it. When several bolts are drawn
    in one drawn frame, each takes all its draws in turn, in the order in which they are drawn
    (claim 12): the most recently made first. The draws of the spark system
    are S-0616's: they come from the scenario random stream, so the sparks reach game state as
    S-0616 claim 2 says, though the bolt does not. Making the system does not depend on whether
    the bolt is drawn, so those draws are the same on every machine. They never appear on the
    unsynchronised stream, so they do not disturb EB8's reading of it. In our engine, within one cycle, the bolt draws of the drawn frame come before
    the per-instance sound draws of that cycle's end pass (S-0701 claim 30). This is design; the
    original's order between the two is open (Open questions).
    - Backing: `measurable` (scenario EB8) for the count, strand A's draws coming first, and the
      splits taken depth first with each parent before its children. EB8 tells A's draws from
      the others at each split whose length counter is at most 128, where A draws R(−1, 1) and B
      and C draw R(−*J*ₖ, *J*ₖ), with *J*ₖ at least 5 at any level that is split (F1). Depth
      first gives a different sequence of levels, and so of ranges, from level by level.
    - Backing: `measurable` (scenario EB8, two-bolt run) for the order between bolts in one drawn
      frame.
    - The number of levels *D* comes from *λ* (Terms). Its height term Δ*z*² is `measurable`
      (scenario EB8's height run, where the planar and the spatial length predict a different
      *D*). Its table root is `unmeasured` here, as S-0204 claim 5 backs it: with S-0205's
      `quirks.chop_rounding` on, the table root and the exact root give a different *D* only when
      ⌊√(Δ*x*² + Δ*y*² + Δ*z*²)⌋ is exactly a bound 65 × 2^*d* and the table root falls short
      there, and everywhere else the two give the same count; with that switch off, F1's rounding
      of the sum can also carry the table root across a bound (S-0204 F1). No scenario here
      arranges either case.
    - B before C, the axis order and the order of the halves are design, outside this claim's
      rating: in each of them the draws that could swap places have the same range (the three
      axes of one strand, strands B and C, and two pieces of one level), so no recording tells
      the orders apart, and a swap only moves where the jagged lines are drawn. Any order
      conforms.
    - Strand A first at the splits whose length counter is above 128, and the phase draw's place
      after the report sound's choice, are design in the same way, outside this claim's rating.
      At those splits A's range is B's and C's, so no recording tells A's place from theirs; and
      the phase draw's place against a report sound's draws would need a report sound with
      several choices, while the draws reach no game state (Terms). Our engine takes A first and
      the phase draw after the report; any order conforms.
    - Confidence: high for the count per split and per drawn frame for a given *D*, strand A
      first at the splits whose length counter is at most 128, and depth first. Medium for the
      order between bolts, until EB8's two-bolt run, and for the height term in *λ*, until EB8's
      height run. Low for the table root's *D* at a bound (`unmeasured`).

## Formulas

**F1, levels.** For a bolt of length *λ* (Terms), the number of split levels is

> *D* = min(8, the least *d* ≥ 0 with ⌊*λ* / 2^*d*⌋ ≤ 64).

A piece at level *k* (the whole bolt is level 0) has the length counter *ℓ*ₖ = ⌊*λ* / 2^*k*⌋ and
the jitter amplitude *J*ₖ = ⌊*J*₀ / 2^*k*⌋, with *J*₀ = ⌊23 *λ* / 256⌋. A piece is split while
*ℓ*ₖ > 64 and *k* < 8. Every piece at a level is split or none is, so the bolt has 2^*D* pieces per
strand. The length counter halves with the level; it is not the length of the displaced piece.
The pinning threshold is *P* = ⌊102 *λ* / 256⌋. Every halving written ⌊*v* / 2⌋ here and
below rounds towards negative infinity.

**F2, sway.** At the first split only, six integers are formed from the phase *φ* (Terms):

> *s*ᵢ = trunc(sinT(*φ* × π / (*i* + 7)) × *J*₀), for *i* = 0 … 5,

with sinT the table sine (S-0105 F1), the argument and the product in double precision and trunc
towards zero; the last bit of these presentation values is not specified (Terms). The same shift is
added to the midpoint of each strand at that split, after the plain midpoint and before the random
jitter of F3:

> Δ*x* = *s*₀ + *s*₃, Δ*y* = *s*₁ + *s*₅, Δ*z* = ⌊(*s*₄ + 2 *J*₀ + *s*₂) / 2⌋.

**F3, midpoint displacement.** Each strand starts as the straight line from the bolt start to the
bolt end. A piece of a strand with end points *p* and *q* at level *k* is split at

> *m* = (⌊(*p*ₓ + *q*ₓ) / 2⌋, ⌊(*p*ᵧ + *q*ᵧ) / 2⌋, ⌊(*p*_z + *q*_z) / 2⌋),

plus F2's shift at level 0, plus a jitter per world axis:

- strand A: R(−*J*ₖ, *J*ₖ) when *ℓ*ₖ > 128, and 2 *J*ₖ × R(−1, 1) when *ℓ*ₖ ≤ 128,
- strands B and C, when *ℓ*ₖ > *P*: their midpoint becomes strand A's displaced midpoint
  plus ⌊R(−*J*ₖ, *J*ₖ) / 2⌋ per axis,
- strands B and C, when *ℓ*ₖ ≤ *P*: their own midpoint plus R(−*J*ₖ, *J*ₖ) per axis.

With *P* close to 0.4 *λ*, the pinned case holds at levels 0 and 1. The two halves *p*–*m* and
*m*–*q* are then split in turn at level *k* + 1. Coordinates are in leptons.

**F4, depth biases.** Each point of a strand carries a depth term *e*: *e* = *a* at the start, with
*a* the building-source adjustment of S-0518 claim 20 for a building firer and 0 for any other,
*e* = 0 at the end, and *e*(*m*) = ⌊(*e*(*p*) + *e*(*q*)) / 2⌋ at each split. A piece from *p* to
*q* has the biases *b*(*p*) = *e*(*p*) − *L*(*p*_z) − 2 and *b*(*q*) = *e*(*q*) − *L*(*q*_z) − 2,
with *L*(*z*) the height lift of S-0101 F2 at the displaced height. S-0514 F8 then says
which of the two biases the piece uses and how clipping changes it.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Bolt life | 17 | drawn frames | claim 5 |
| Strands per bolt | 3 | lines | claim 9 |
| Longest unsplit piece | 64 | leptons (length counter) | F1 |
| Most split levels | 8 | levels | F1 |
| Jitter amplitude factor | 23 / 256 | of *λ* | F1, truncated |
| Pinning threshold factor | 102 / 256 | of *λ* | F1, truncated |
| Coarse-jitter limit | 128 | leptons (length counter) | F3 |
| Sway divisors | 7 to 12 | – | F2 |
| Phase range | 0 to 256 | – | claim 13, R(0, 256) |
| Strand B and C colour | 10 (5 with the alternate-colour flag) | general palette entry, row 26 | claim 10 |
| Strand A colour | 15 | general palette entry, row 26 | claim 10 |
| Draws per split | 9 | ranged draws | claim 13 |
| Depth constant | −2 | depth units | F4 |

## State and ordering

| State | Event | Result |
|---|---|---|
| none | a shot that makes a bolt (claim 2) launches its projectile | bolt made with phase R(0, 256), and its spark system at the bolt end (in our engine only if the game-wide spark type is present, claim 6); attached if claim 4 allows |
| bolt, attached | a drawn frame begins its effects group | start moved to the vehicle's current fire point for the bolt's slot whether or not it is then drawn, then drawn (claim 9) unless claim 5's view test leaves it undrawn |
| bolt, attached | its vehicle is removed | no longer attached, start kept (a destroyed vehicle still in play keeps it attached) |
| bolt | a drawn frame, its line in the view | drawn with new draws, phase + 1, one frame of its life used |
| bolt | a drawn frame, its line wholly outside the view | not drawn, no draws, phase + 1, one frame of its life used |
| bolt | its 17th drawn frame has been used | gone; if it is attached, its attachment ends (claim 4) |

Ordering: a shot's bolt and spark system are made only when its projectile was put on the map
(a void shot makes neither, claim 2), and in our engine any draw of the shot's report sound (S-0701
claim 21) comes before the phase draw (design, claim 13). A bolt is made in the tick of its shot,
and a cycle's drawn frame comes before that cycle's tick (S-0201 claim 6), so a bolt is first drawn
in the next cycle's drawn frame: its draws in a drawn frame come after the draws of every earlier
tick and before those of the same cycle's tick. Draws of different bolts in one drawn frame come
most recently made first (claim 13). The place of a bolt's draws against those of a tick is
`unmeasured` and rated low: the draws are on the unsynchronised stream and reach no game state,
and the order rests on S-0201 claim 6's order of drawn frame and tick, which S-0201 rates
`unmeasured`. The first drawn frame of a bolt is claim 5's, with its backing.

## Edge cases and quirks

- **Slot of the bolt (fixed, no switch).** Our engine takes the bolt's start and colour flag from
  the slot that fired (claim 3); the original takes them from a fresh weapon choice against the
  shot target, a drawing glitch with no gameplay impact where that choice gives another slot
  (low, `unmeasured`; Open questions).
- **A garrisoned building's bolt start (fixed, no switch).** Our engine starts the bolt at the
  muzzle point that the shot was fired from (claim 3); the original, a drawing glitch with no
  gameplay impact, starts it at the muzzle point of the occupant that fires next (low).
- **A bolt's side within a burst (fixed, no switch).** Our engine starts the bolt on the side of
  the shot's own projectile (claim 3); the original, a drawing glitch with no gameplay impact,
  uses the side of the firer's next shot (S-0313 claim 2 with the burst counter *j* mod *B* of
  S-0301 claim 4), and for an attached bolt's later frames the counter's value at that frame
  (low).
- **A short bolt is straight.** A bolt of length 64 or less is drawn as one straight line in strand
  A's colour, entry 15 (claim 9). This follows from F1 and needs no switch. EB4 may leave it
  unreached, since the fire point must be within 64 leptons of the bolt end.
- **Older bolts on top.** Because bolts are drawn newest first (claim 12), an older bolt covers a
  newer one where they cross. It is a drawing detail with no gameplay impact, so it gets no switch.
- **A bolt that leaves the view.** A bolt that is out of view in some drawn frames still uses up
  those frames of its life (claim 5), so scrolling never makes a bolt last longer.

## Import mapping

| Key | Section | Feeds | Form | Absent |
|---|---|---|---|---|
| `IsElectricBolt` | weapon | the electric-bolt flag (claims 1 to 13) | boolean | no |
| `IsAlternateColor` | weapon | strand B and C colour choice (claim 10) | boolean | no |
| `IsLaser` | weapon | the laser flag that wins over the bolt (claim 2, S-0514 claim 23) | boolean | no |
| `IsRadBeam` | weapon | the radiation-beam flag that the bolt wins over (claim 2, S-0514 claim 23) | boolean | no |
| `Spawner`, `DrainWeapon`, `DiskLaser`, `Suicide` | weapon | the firings without a projectile of claim 1 | as mapped by their own specs: S-0301 (spawner), S-0608 (drain), S-0615 (disc laser), S-0519 (suicide) | those specs' rows |
| `IsMagBeam` | weapon | the magnetic-beam flag that the bolt wins over (claim 2; S-0607 claim 21 for its wave) | boolean | no |
| `UseFireParticles`, `UseSparkParticles`, `IsRailgun`, `IsSonic` | weapon | effect flags independent of the bolt (claim 2, S-0301 claim 5) | boolean | no |
| `DefaultSparkSystem` | `[CombatDamage]` | the game-wide spark type at the bolt end (claim 6) | name | S-0616's row |
| `PALETTE.PAL` | none (file name fixed by the original) | the general palette of claim 10 | palette file | S-0306's row |

## Measurement scenarios

All under the measurement setup (glossary): stock data with no add-ons or mods, plus the named test
files that a scenario lists. Frame captures are taken at every drawn frame, at full frame rate, in
skirmish at a fixed game speed.

- **EB1, the shot (claims 1, 2, 3, 6, 7).** A purpose-made map with flat clear ground. Named test
  files: a loose copy of the rules file that gives a test vehicle a weapon with the electric-bolt
  flag and a slow, visible test projectile, and a second variant whose weapon has no report sound.
  Fire (a) at a stationary vehicle, (b) at a building, (c) at the ground by force-fire, (d) at a
  vehicle that the shot kills, (e) and (e′), see below, (f) with the laser flag set as well, and
  (g) at a test vehicle driving at full speed across the line of fire, with a second test warhead
  of very low damage so that it survives. Record the tick of the shot, the projectile's flight, the
  tick and place of the damage, the bolt, the sparks and the sounds heard. Expected: the projectile
  flies as for any weapon and the damage comes when it arrives, not at the shot. The bolt and the
  sparks appear at the shot, at the target's point (or the cell centre in c), and also in d. In f a
  laser and no bolt is drawn. In g the bolt's far end and its sparks stay where the target's point
  was at the shot, in every drawn frame of the bolt's life, while the vehicle drives on. In the
  no-report variant the shot makes no sound other than the start or stop sound of an animation
  it causes, a shot animation's or its impact's explosion animation's (S-0701 claim 22), which
  the record names. Falsified by damage at the
  shot with no projectile, by a bolt without a projectile, by a bolt end that moves with the target
  in g, or, in the no-report variant, by a sound at the shot that is not such an animation's. Parts
  e and e′ use a test building with the same weapon, so that the firer cannot move: (e) an enemy
  test vehicle with no weapon, given no orders, is placed beyond the weapon's range, (e′) a second test weapon with a minimum range
  is used and the vehicle is placed inside it. In each, the building is given the attack order on
  the vehicle, which stays where it is, and the observation ends 600 ticks after the order.
  Expected: no shot, no projectile, no bolt and no sparks in that time. Falsified by any shot or
  bolt. Part h: the test vehicle's type gets the spray attack of S-0313 claim 9 and fires at a
  stationary vehicle. Expected: each bolt and its sparks end at the centre of the cell its shot is
  redirected to, at that cell's height, whatever the firer's target. Falsified by a bolt that ends
  at the firer's target when the redirected cell is another. Part h′: a variant of the test weapon
  that also has the area-fire flag (S-0306 claim 3 a), fired by the stationary test vehicle at an
  enemy vehicle 2 cells away. Expected: each bolt and its sparks end at the centre of the firer's
  own cell, at the terrain height there. Falsified by a bolt that ends at the target vehicle. Part
  i: the test weapon with the electric-bolt flag and the void-shot projectile of S-0306 scenario P5
  run d, an arcing projectile whose launch pitch has no solution, so that the shot is void (S-0301
  claim 15). Expected: no projectile, no bolt and no sparks, and the rearm countdown as S-0301
  claim 15 says. Falsified by a bolt or sparks. If a projectile appears, the shot was not void and
  the part is inconclusive. Part j (line of fire): the arrangement of S-0307 scenario RG8 run a, a
  straight test weapon whose projectile is subject to walls and not to cliffs, with a warhead that
  is not wall-damaging, range 4, and an enemy wall cell of the wall type that run a uses between
  the firer and an enemy vehicle 3 cells away on level ground, so that the range test's
  line-of-fire walk (S-0307 claim 11, S-0306 claim 22) refuses the shot. The firer is the test
  building of part e with that weapon given the electric-bolt flag; the control is the same
  building with the weapon without the flag. Each is given the attack order on the vehicle; observe
  for 600 ticks. Expected: neither fires, and no bolt and no sparks appear. Falsified if the
  flagged building fires. If the control fires, the arrangement does not block it and the part is
  inconclusive. Part k (miss): the test vehicle with a test weapon whose projectile is inaccurate
  and arcing (S-0306 claim 4), and a control with the same weapon without the flag, each
  force-fired 20 times at the same ground cell from the same place. Record each detonation point.
  Expected: the flagged shots scatter as the control's do (detonation points away from the cell
  centre, within the same bound), and every bolt ends at the cell centre whatever the detonation
  point. Falsified if every flagged shot detonates at the cell centre while the control's scatter,
  or a bolt ends at a detonation point away from the centre. Part l (magnetic-beam flag): a
  variant of the test weapon with the electric-bolt and magnetic-beam flags, fired by the test
  vehicle, with no live wave, at a stationary vehicle 4 cells away; the control is the same
  weapon without the electric-bolt flag. Expected: the flagged variant fires its projectile and
  makes the bolt and its sparks, with no magnetic wave. Falsified by a magnetic wave with the
  flagged variant. If the control makes no wave, the part is inconclusive.
- **EB2, following (claim 4).** The test vehicle of EB1 fires while driving across the view at full
  speed, and a test infantry type and a test building with the same weapon fire too. In every
  run of EB2 and EB3 the test weapon's rate-of-fire field is 60 ticks, so each shot or burst
  comes after every bolt of the firer's previous shot or burst has ended (claim 5), and the
  vehicle has no attached bolt from an earlier shot or burst when it fires. A shot or burst begun
  while a bolt of the same firer's earlier shot or burst still shows is not scored. Expected: the
  vehicle's bolt start moves with its fire point in every drawn frame and the end stays put. The
  infantry and building bolts keep their starts. Destroying the vehicle on land during the bolt's
  life, so that it is removed at once, leaves the start where it last was. Carried run: a test
  transport that is open-topped (S-0507) and may carry the test vehicle drives across the view with
  it aboard while the test vehicle fires from inside. Expected: a carried vehicle is off the map
  (claim 4), so its bolts keep their starts while the transport moves. Falsified by a carried
  vehicle's bolt start that follows. If test data cannot make a vehicle fire from inside, the run
  is reported unreached. Aircraft run: a test aircraft type with the same weapon fires at a ground
  target while flying across the view. Expected: its bolt start stays where it was made while the
  aircraft flies on. Falsified by a start that follows the aircraft.
- **EB3, burst (claims 2 and 4).** Set-up: the test weapon with burst 2 and a burst delay of 0,
  on the test vehicle, which drives in a straight line across the view past a stationary target
  about 4 cells away and fires at it while moving. Record: a frame capture of every drawn frame
  from the shot until both bolts have ended, with the screen positions of each bolt's start and
  end and of the vehicle's fire point, and the number of spark systems that appear. Expected:
  two bolts and two spark systems per burst. In each drawn frame the first bolt's start is at
  the vehicle's current fire point for that weapon, while the second bolt's start stays where it
  was made. EB3 does not judge on which side of the vehicle either bolt starts: the original's
  side within a burst is not isolated here (claim 3, Edge cases).
  Falsified if a burst shows another number of bolts or spark systems, if the second bolt's
  start moves, or if the first bolt's start stays where it was made while the vehicle moves.
  Runs: 3 bursts, each after the previous burst's bolts have ended (EB2's rate of fire).
- **EB4, life and shape (claims 5, 9, 10, 12).** The stationary test vehicle fires at targets 1, 3
  and 6 cells away, and, where test data can arrange it, makes a bolt no longer than 64 leptons
  (the fire point must be within 64 leptons of the bolt end; if no test data arranges it, the
  short bolt is reported unreached). Count the drawn frames in which each bolt shows, find the
  pixel colours as general palette entries, and look at how the shape changes. Expected: 17 drawn
  frames. A new shape in each for a bolt longer than 64 leptons. Strands in entries 10 and 15 (5
  and 15 with the alternate-colour flag), where the three strands of one piece overlap, the
  entry-15 strand on top. Three strands that meet at the ends and part in the fine detail.
  The short bolt is one straight line in entry 15's
  colour. Two bolts fired a few drawn frames apart that cross show the older one on top. A test map
  trigger whose pause action starts a timed freeze of a few seconds in the tick after the shot: the
  drawn frame of that tick's cycle, the bolt's first, comes before the freeze starts (S-0201 claim
  6), so expect the bolt's other 16 drawn frames during the freeze, before the next tick.
  Falsified if all 17 show during the freeze, none before it: the bolt would then first show
  one drawn frame later than claim 5 says. Removing
  the input focus (a stand-still, no drawn frames) does not use up its life. Scroll run (claim 5):
  with the random-state recorder reading the unsynchronised stream, fire one shot with the bolt in
  view. The scroll run and the edge run below use EB8's sound isolation: its test weapon, warhead,
  unarmed target, unselected firer with no turret rotation sound, map without ambient sound and
  music off (or music picks recorded), so that no
  sound instance takes a draw from that stream in the window. A run in which any sound is heard is
  void and says which sound it was. If every scroll run is void, its phase result backs nothing
  and stays `unmeasured`; its count of drawn frames still stands. If every edge run is void, its
  no-draw result stays `unmeasured`; its pixel result still stands.
  After *k* drawn frames with the bolt shown (*k* from 3 to 6), scroll the view so that the bolt's
  whole straight line is outside it for *n* drawn frames (*n* from 3 to 8), then scroll back.
  Record in each drawn frame whether the bolt shows, and capture the first drawn frame after the
  return. Choose *k* and *n* so that the two predictions below differ by more than four pixels for
  every pair of assignments of the draws to strands and axes that claim 13 allows, so that their
  two-pixel tolerances (claim 9) do not overlap. If no *k* and *n* in those ranges do, the scroll
  run is inconclusive for the phase and is reported so; its count of drawn frames still stands.
  *Expected:* the bolt shows in drawn frames 1 to *k* and *k* + *n* + 1 to 17 after the shot and is
  gone from the 18th. In the captured frame the midpoint of strand A at the first split lies where
  F2 and F3 put it, from the recorded draws (assigned to the strands and axes in any order that
  claim 13 allows), with the phase raised once for every drawn frame since the shot, the *n* frames
  out of view included. *Falsified if:* the bolt shows in 17 drawn frames in all, or in more than
  17 − *k* − *n* after the return, or the midpoint matches the phase raised only for the frames in
  which the bolt was drawn. Crossing captures (claim 12): in the captures of the 3- and 6-cell
  bolts, where two pieces of one bolt cross, the piece nearer the end is on top, also where strand
  B or C of the piece nearer the end crosses strand A of the other; a bolt with no such crossing
  backs nothing for this order, and if no capture has one the order is reported unreached.
  Colour runs (claim 10): the 3-cell shot is repeated by a test vehicle of a second house whose
  house colour differs from the first's, and by the first vehicle with a variant of the test
  weapon that has the alternate-colour flag. *Expected:* strands in entries 10 and 15 for both
  houses, and in entries 5 and 15 with the alternate-colour flag. *Falsified if:* the strand
  colours change with the firing house, or the variant's strands B and C are not entry 5.
  Edge run (claim 5): with the random-state recorder reading the unsynchronised stream, the
  6-cell shot is fired and the view is then scrolled so that the bolt's straight line lies just
  below the view's lower edge, closer to it than the screen displacement that F2's whole shift (Δ*x*,
  Δ*y* and Δ*z*) gives the bolt's middle from the recorded phase (F2 takes no draws), less the
  largest screen displacement that F3's jitter can add to strand A's middle at the first split. Only drawn frames in which that margin still reaches into the view count.
  *Expected:* in those drawn frames no pixel of the bolt in the view and no draws for it.
  *Falsified if:* pixels of the arc show at the view's lower edge, or the bolt's draws are taken.
- **EB5, a blocked projectile (claims 3 and 6).** Named test data: (a) a test projectile subject to
  walls with a wall-damaging test warhead, fired at a target behind a wall whose overlay type is
  one of the three that S-0307 claim 11 lets such a warhead fire through, so that the shot is fired
  and its projectile is placed to stop at the wall (S-0306 claim 9, and the in-flight
  test of S-0306 table T1 for a projectile that is not invisible); (b) a test projectile subject
  to neither cliffs nor walls, which always passes the line-of-fire test (S-0307 claim 11), fired
  from low ground at a target on high ground behind a cliff edge; and (c) the same projectile fired
  at a target behind a test building that stands between the two ends. A run in which no shot is
  fired is void. Expected: in a the projectile detonates at the wall, while the bolt and the sparks
  reach the target's point. In b and c the bolt's far end and the sparks are at the target's point
  too. Parts of the bolt may be hidden behind the cliff or the building image (claim 11), but where
  the bolt shows on either side of them it lies on one unbroken arc from start to end. Falsified by
  a bolt that ends at the wall, the cliff or the building.
- **EB6, shrapnel (claim 8).** Named test data: a projectile with a shrapnel weapon that has the
  electric-bolt flag and a positive fragment count, fired into a group of targets. A test unit
  type also names the shrapnel weapon as one of its weapons, so that the weapon is read
  (S-0004). Part a: count the bolts at the burst.
  Expected: one bolt per fragment, each starting at the burst point and ending where that
  fragment is aimed (whatever rule picks it), each with sparks. Falsified if part a shows another
  count or start. Part b (record only, for the Open questions): the same shot from a long range,
  with the firer destroyed while the projectile flies; record whether fragment bolts appear and
  whether fragments fly. It judges no claim: without a known firer, fewer fragments launched and
  no bolt for a launched fragment can look alike.
- **EB7, depth and shroud (claim 11).** Set-up: a test map where the stationary test vehicle fires
  the test weapon (a) from low ground at a target on high ground, across a cliff edge that stands
  in front of the bolt's lower part on screen; (b) at a target behind a building of a third house,
  with the building's image between the bolt's middle and the viewer; (c) across a row of three
  vehicles of a third house standing between it and its target, so that the bolt's line on
  screen crosses their images; (d) from a tall test building with the test weapon at a target
  below it. Part (e): a firer of a computer-controlled house, standing in cells that are fully
  shrouded for the recording player, fires at a unit of that player in visible cells, so that the
  bolt runs from fully shrouded cells through partly shrouded ones into visible ones. Record: a
  frame capture of the drawn frames in which each bolt shows. Along each drawn strand (claim 9),
  follow its run of bolt-coloured pixels and record where the run enters and leaves the
  occluding image: whether the strand continues in the bolt's colour across the image or breaks
  off at the image's edge and resumes beyond it. A strand that never crosses the image in a
  frame does not count for that frame, since its jagged path need not meet the straight line
  between the ends. In (e), the colour of the bolt pixels in the
  partly shrouded cells, against the same palette entry's colour where the cells are visible.
  Expected: in (a) and (b) every strand that crosses the cliff or the building's image breaks off
  at its edge and resumes beyond it; in (c) every crossing strand continues in the bolt's colour
  across each vehicle image; in (d) the strands continue across the firing building's own image
  at the start; in (e) no bolt pixel shows in a fully shrouded cell, and bolt pixels in a
  partly shrouded cell have every channel at or below the visible colour's, and at least one
  lower where the cell is darkened. Falsified if a crossing strand breaks off at a vehicle
  image, if no crossing strand ever breaks off at the cliff or the building image, if a strand
  breaks off at the firing building's own image near the start, if a bolt pixel shows under full shroud, or if a partly shrouded cell's bolt pixels
  keep the full visible colour where the cell is darkened. Runs: 2 for each part.
- **EB8, draws (claims 9 and 13).** The no-report test weapon of EB1, which the test file also
  gives no shot animation, with a test warhead of damage 0 that makes no animation and no
  sound, an unarmed target that the hit therefore does not kill, a firer that is not selected and
  gives no voice, a map with no ambient sound, and the random-state recorder reading the
  unsynchronised stream. The firer has no turret, or a turret with no rotation sound, so that no
  turret-rotation sound (S-0701 claim 23) is made in the window, and music is switched off for the
  run. If music cannot be switched off, the music picks in the window (S-0202 claim 11) are
  recorded with their draws, and those draws are left out of the count. Every sound instance takes its draws from that
  stream (S-0701 claims 13, 14 and 30), whether or not it is heard, so every source of a sound
  in the window is removed through data: no sound instance is made from the shot to the bolt's
  last drawn frame, in view or not. The sounds heard are still recorded as in EB1, and a run in
  which any is heard is void and says which sound it was. If every run of a part is void, that
  part backs nothing, and the counts it tests stay `unmeasured`. A stationary firer fires one shot at a
  target about 5 cells away with the bolt in view, and once more with the view scrolled away from
  it. Compute *λ* from the two end points; any *λ* from 1040 to 2079 leptons gives *D* = 5. Then
  repeat the shot in view at other distances: several from 1 to 8 cells, chosen so that the
  predicted *D* takes each value from 2 to 5 at least once, and, on a map with no shroud, once at a
  cell more than 66 cells away by force-fire with a test weapon of that range. There *D* = 8 by the
  cap, 2295 draws per drawn frame, against 4599 if there were no cap and the uncapped *D* were 9
  (more beyond). Bound test (claim 9): at least one shot whose *λ* lies from 64 × 2^*d* to 65 ×
  2^*d* − 1 leptons for some *d* from 2 to 5 (for *d* = 5, 2048 to 2079), where a split rule of
  "below 64" instead of "at most 64" would predict one level more. The end points are varied (a test infantry target at
  another place within its cell, the firer stopped at another point of its path) until the computed
  *λ* falls in one; if none does, the bound test is reported unreached. Falsified if that shot
  takes 9 × (2^(*d* + 1) − 1) draws per drawn frame. Height run (claim 13): the firer on high
  ground behind a cliff several height levels above a target on low ground, at a distance chosen
  from the computed end points so that the planar length (Δ*z* left out) and *λ* predict a
  different *D*. Falsified if the count is the planar length's. Expected for each shot:
  the raw values of one ranged draw R(0, 256) at
  the shot, then in each of the 17 drawn frames with the bolt in view the raw values that 9 ×
  (2^*D* − 1) ranged draws (279 for *D* = 5) with F3's ranges consume (S-0202 claim 4), and none
  while it is out of view. The ranges halve with the level, and strand A's become R(−1, 1) at a
  level whose length counter is at most 128. At such a level, the first three draws of each split
  consume as R(−1, 1) and the next six as R(−*J*ₖ, *J*ₖ). Any other place for A's three draws among
  the nine predicts a different consumption for most recorded sequences, since R(−1, 1) rejects raw
  values that R(−*J*ₖ, *J*ₖ) keeps.
  Two-bolt run (claim 13): two stationary firers fire one shot each, a few drawn frames apart, at
  targets that give *D* = 2 and *D* = 5, the firer whose target gives *D* = 5 first, both bolts in
  view. *Expected:* in each drawn frame in
  which both are drawn, the 27 draws of the newer bolt, then the 279 of the older, each with its
  own F3 ranges. *Falsified if:* the older bolt's draws come first, or the two interleave.
- **EB9, firings without a projectile (claim 1).** Named test data: test firers whose only weapon
  has the electric-bolt flag together with (a) the spawner flag, spawning a test aircraft with an
  ordinary weapon, (b) the drain flag, fired at an enemy building that it can drain (S-0608 claim
  3), (c) the disc-laser flag, and (d) the suicide flag. Each is ordered to attack an enemy vehicle
  or building in view. Expected: no bolt and no sparks at any of these firings, and, in part a, no
  bolt from the aircraft's shots. The ordinary effects of each firing (the spawned aircraft, the
  drain, the disc, the firer's own destruction) still happen. Falsified by any bolt.

## Changes for other specs

- **Glossary, *electric-bolt shot*.** It says the shot is fired instead of launching a projectile.
  The shot launches its projectile like any other (claim 1). Proposed wording: "The lightning-style
  drawn effect, with its sparks, that a weapon flagged as an electric-bolt weapon adds to its
  shots, except as S-0316 claims 1 and 2 say. The shot itself launches its projectile as usual. See
  S-0316." The decision is listed under Questions for the project lead.
- **S-0616 Terms, *Electric bolt*.** Same correction: the bolt is drawn in addition to the
  projectile, and S-0316 owns it. S-0616 Open questions' item on the unowned shot can point here.
- **S-0616 claim 1.** It says each electric-bolt shot makes one spark system. Claim 8 here also
  gives a spark system to each bolt of a shrapnel fragment, which is not a shot (Terms). Claim 1
  could say that each bolt, of a shot or of a fragment, makes one system.
- **S-0202 claim 11.** It gives the electric-bolt jitter as `unmeasured`, because the shot's report
  sound draws from the same stream in the same tick. By S-0701 claim 21 that holds only for a
  shot fired by occupants: an ordinary shot's report takes no draw. A test weapon with no report
  sound, read with the random-state recorder, removes even that case (scenario EB8), so that part
  can become `measurable`.
- **S-0306 Import mapping.** It says the shrapnel weapon and count are not imported there. The
  spec that comes to own the fragment rules imports them; claim 8 here then reads the electric-bolt
  flag of that shrapnel weapon.
- **S-0313 claim 5.** It gives a passenger's fire point for infantry only. A vehicle carried by
  an open-topped transport fires from S-0313 claim 1's fire point computed from its own
  position, facings, weapon slot and burst counter, with its position the transport's (S-0507
  claim 15); claim 4 here relies on this. S-0313 claim 5 could say so.
- **S-0607 claim 21.** It creates a magnetic wave whenever the firer has no live wave. By claim 2
  here, a magnetic-beam weapon that also has the electric-bolt flag makes no wave; the shot itself
  is still fired, as claim 21 already says of a shot that makes no wave. Claim 21's creation
  bullet could list this exception. Proposal, not backed here: the laser and radiation-beam flags
  are likewise ranked above the magnetic-beam flag, so that only the highest of the four
  line-effect flags a weapon has makes its effect; S-0514 and S-0607 would need their own
  scenarios for those pairs.
- **S-0102 claim 2.** No change: it already places bolts after lasers and before line trails.
- **S-0514 claim 23.** Its ranking of line-effect flags ends at the radiation beam. Claim 2 here
  adds that the electric-bolt flag also wins over the magnetic-beam flag. The order of the other
  flags against it is the proposal under S-0607 claim 21 above.

## Open questions

- **Slot.** In which shots the original's fresh weapon choice against the shot target (Edge
  cases) gives a slot other than the one that fired. S-0518 claim 7 gives one candidate: a prism
  master's shot always uses slot 0. Redirected shots are others: the choice is then made against
  the redirected cell (a spray attack's, or an area-fire weapon's own cell). No scenario here
  isolates such a shot.
- **Closed transport.** Where an attached bolt's start goes in the original while its vehicle is
  inside a closed transport (claim 4). No spec gives a carried vehicle's position there; our
  engine keeps it where the vehicle went off the map (design).
- **Bolt and sound draws.** When a cycle's drawn frame draws a bolt and the same cycle's end pass
  also makes per-instance sound draws (S-0701 claim 30), which of the two takes the unsynchronised
  stream first in the original is not known. Scenario EB8 avoids the case: its weapon has no
  report sound and no shot animation, its warhead makes no animation, its firer has no turret
  rotation sound and music is off, so that no sound instance is made in the window, and a run in which any sound is heard is void. Our engine fixes the order by design: within one cycle, the bolt draws of its
  drawn frame are taken before the per-instance sound draws of that cycle's end pass, so that
  identical inputs give identical choices (S-0701).
- **Shrapnel without a firer.** Whether a fragment launched after the projectile's firer has gone
  makes a bolt (claim 8). The original is believed to make fragment bolts only while the firer is
  known, but which fragments are launched once the firer is gone belongs to the fragment rules
  that S-0306 lists as open, so EB6 part b can show that no bolt appears without showing why.
- **Fragment colour.** Which weapon's alternate-colour flag gives a fragment's bolt its colour
  (claims 8 and 10): the shrapnel weapon's or the weapon that fired the projectile.
- **Interior pixels.** Which pixels a piece covers between its ends, as for the beam flash (S-0514
  Open questions).

## Questions for the project lead

- **Glossary, *electric-bolt shot*.** Whether to adopt the proposed wording under Changes for
  other specs: the shot launches its projectile as usual, and the bolt with its sparks is drawn
  in addition (claim 1).
