---
id: Q-2172
title: A pad-bound aircraft started while the house's others are out, and the crash of the one that comes back
status: answered
related: [S-0404, S-0407, S-0501, S-0213, S-0417]
answered-by: [S-0407]
---

## Need

The lead's playtest of 2026-10-31 (M8 A4 round 6, note 35, `docs/m8-claims/a4-play-notes.md`)
reports: "I can buy new black eagles and harriers when others are in the air. So the existing
ones die when trying to return base. The option is successfully blocked when the existing ones
aren't flying." The lead expected the build refused while the house owns as many pad-bound
aircraft as it has pads.

Our engine follows the approved claims as written, so this is what they give:

- S-0404 claim 3 applies a build order for an aircraft only if a factory "has a free dock".
- A pad-bound aircraft sent out on an attack breaks off its pad contact (S-0213 table T3 b; a
  pad-bound type is not tethered, S-0417 claim 11). Its pad then counts as a free dock.
- The house-wide pad limit of S-0407 claim 16 (with S-0501 claim 30) is a queue-time test. It
  is made only for a copy queued behind another item, not when an idle line starts an item.
- An aircraft that comes back and finds no free pad crashes (S-0501 claim 46).

So a house whose pads are all empty because its aircraft are out can build another on every pad,
and each extra one that returns to a full set of pads is lost. Test
`claim_16_pad_capacity_is_tested_only_at_queue_time` pins this today.

The person who plays reads this as a bug. Our "Pathing and other fixes" layer may set a switch
only to an off position that an approved spec states (docs/m8-claims/c7-play-fixes.md,
*Admission*). No spec states one here, so the layer cannot offer the behaviour the lead expected.

## Questions

1. Confirm by observation: in the original, with every pad-bound aircraft of a house out on an
   attack (pads empty), does a click on the aircraft icon start a new one at once? Is the
   house-wide pad count of S-0407 claim 16 really not tested when an idle line starts the item?
2. Confirm that an aircraft returning to a house whose pads are all occupied crashes (S-0501
   claim 46), rather than waiting, circling or landing elsewhere, when the house has built past
   its pad count this way.
3. If both hold, is this a quirk the spec should name with a switch, and what is its off
   position? The behaviour the lead expected is the house-wide test (owned pad-bound aircraft,
   counting those in flight and those in production, against the house's pads) applied also
   when an idle line starts an aircraft, so the build is refused while the count is at or above
   the pads.

## Possible observation

A house with one airfield-type building of 4 pads and 4 pad-bound aircraft. Order all four to
attack a far target. While they fly, click the aircraft icon: record whether production starts,
and how many aircraft the house can have in production or flight at once. Let them return: record
what the fifth does when it finds every pad taken.

## Current workaround

None needed: the engine follows S-0404 claim 3, S-0407 claim 16 and S-0501 claim 46 as written.
If the spec names a switch with an off position, the play-fixes layer can admit it under C7's
rules.
