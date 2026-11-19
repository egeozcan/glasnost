---
id: Q-3257
title: A balloon-hover bomber that hovers over its target without facing it
status: open
related: [S-0307, S-0310, S-0502, S-0204]
answered-by: [S-0502]
---

## Need

Issue 445 (own AI): an AI house's airship-type bomber hovers over an enemy base and does nothing.
A synthetic trace (`crates/deterministic/ai/tests/air_bomber.rs`) rebuilds it. The bomber is a
balloon-hover jumpjet vehicle without the turret flag. Its only weapon has a vertical projectile
and lacks the omni-fire and fire-while-moving flags. It gets one attack order on a 2×2 enemy
power plant, by hand or from our AI. The AI's order is a plain attack, and the lasting firing
check answers *can fire* or *not facing*, never *illegal* or *impossible*. The bomber still never
fires in these cases:

- **Half-cell reach, type with the mobile-fire flag, from any side.**
  1. S-0307 claim 18 makes the plant its destination and keeps it.
  2. Our engine supplies the plant's position (its top-left cell centre) as the jumpjet's
     movement point. The balloon-hover type keeps that exact point (S-0502 claim 16).
  3. The bomber arrives there (claim 12) and hovers with its body facing its last heading.
  4. The check aims at the plant's reference point, the foundation centre (S-0204 claim 4;
     S-0310 claim 1). That is 128 leptons east and 128 south of the position, so the aim error
     is above 2048 and the answer is *not facing*.
  5. With a destination, a vehicle without the turret flag gets no reply to *not facing*
     (S-0310 claim 9, last *not facing* row), so its body never turns to the plant. It stays
     for the rest of the run: attack, destination the plant, hovering, *not facing*.
- **Two-cell reach, type with the mobile-fire flag, coming from the east.**
  1. The heading turns towards the plant's position, not its reference point.
  2. Once in reach, the aim error is already above 2048: 2062 at the first in-reach tick.
  3. It then arrives and stalls as above.

  From the west, north and south, the same bomber fires on its way in.
- **Type without the mobile-fire flag, either reach.** The answer is *moving* for the whole run
  (S-0310 claim 7, first condition: the destination is kept). The balloon-hover reading changes
  only the second condition.

Our engine follows the claims as written here, so we find no broken claim. What is unclear is
whether the game really leaves such a bomber idle, or whether some rule we lack makes it aim or
fire.

The point the jumpjet gets for an object destination is our engine's choice: the object's
position. No claim we found names that point for a building. S-0502 claim 16 speaks of "a
destination point". S-0307 claim 18 says the destination becomes "the target itself".

## Questions

1. When a balloon-hover jumpjet vehicle is given a building as its destination (S-0307 claim
   18), which point does its locomotor fly to: the building's position (top-left cell centre),
   its reference point (S-0204 claim 4), or another point? For a non-building target, is it the
   target's position?
2. Over its target, hovering, with the target still its destination, and with a weapon that is
   not omni-fire, does such a vehicle without the turret flag ever turn its body towards the
   target?
   - S-0310 claim 9 gives no reply to *not facing* while it has a destination.
   - Claim 7's balloon-hover reading ("its locomotor is moving it" instead of "it has a
     destination") is limited to one condition. Does that reading also apply to claim 9's
     *not facing* rows, so that a hovering balloon-hover vehicle (S-0502 claim 20: not moving
     now) turns its body?
   - Or does something else, such as its own turning while hovering, make it face the target?
3. A balloon-hover vehicle type without the mobile-fire flag keeps its target as destination
   while attacking (S-0307 claim 18). Is it refused with *moving* for as long as the attack
   lasts (S-0310 claim 7, first condition), so that it never fires?

## Possible observation

Use S-0307 RG13's setup with a 2×2 enemy building as the target. Give the balloon-hover vehicle
a vertical weapon of reach 0.5 cell, without the omni-fire, fire-while-moving and turret flags.
Run it once from the west and once from the east, and once more with the mobile-fire flag off.

Each tick from the attack order, record:

- position, body facing and jumpjet state;
- destination kind and identity, and the firing check's answer;
- the shots.

See whether it fires and where it hovers: over the building's top-left cell, over its centre,
or elsewhere. See whether its body ever turns while it hovers.

## Current workaround

None in our engine. The simulation is unchanged.

- **The bomber.** It stays over its target as described above.
- **Our AI.** It gives one attack order to an idle bomber that sees an enemy building near it. It
  gives nothing more while the bomber holds that attack, because the bomber is not idle. A later
  AI batch could move a stalled bomber off its target and attack again, once this is answered.
