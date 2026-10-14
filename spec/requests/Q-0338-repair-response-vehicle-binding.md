---
id: Q-0338
title: Which vehicle type is the repair-response vehicle of S-0701 F6
status: answered
related: [S-0701]
answered-by: [S-0701]
---

## Need

S-0701 F6 gives one vehicle type a dedicated repair response. When that vehicle is ordered to
attack with a weapon of negative damage, it plays the game-wide repair-vehicle response first.
Revision 2 says that the exception depends on that vehicle's type name, and not on an imported
field. It withholds the type-name binding (Open questions, Questions for the project lead).
`SoundNames::from_ruleset` in `glasnost_audio::names` has to mark the type as the repair-response
vehicle, and it can only do that once the binding is known.

## Questions

1. Under which clean-room import-mapping convention is the repair-response vehicle identified?
2. Is the type matched by its type name without regard to case, as for other type names
   (S-0003 claim 14)?
3. Does a type that a mod registers under the same name also count, whatever its weapons and
   passengers?

## Possible observation

Not needed if the binding is supplied by the import-mapping convention. Otherwise, follow S-0701
scenario D2's repair-response part with the passenger-configurable vehicle. Then repeat it with a
test vehicle that has a negative-damage weapon but a different type name.

## Current workaround

No type is the repair-response vehicle, so F6's dedicated repair response never plays. An attack
order uses the attack response list of F6. The F6 rule itself is in place and is tested with a
type marked by hand (`events_tests::repair_vehicle_response`). Documented in
docs/development.md, "Audio (M5b)".
