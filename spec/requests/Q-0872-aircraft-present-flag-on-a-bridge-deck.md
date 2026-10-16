---
id: Q-0872
title: Which cell layer holds the object-present flag of an aircraft that rests with the deck mark set
status: open
related: [S-0501, S-0209, S-0105]
answered-by: []
---

## Need

S-0501 claim 43 (revision 15) and S-0209 claim 13 (revision 36) let an ordinary aircraft land on a high bridge, take the aircraft deck mark and be listed in the bridge-deck layer's occupant list. S-0209 claim 1 says an aircraft on the ground sets the object-present flag of its cell, and S-0501 scenario AC27 asks the recorder to read the occupancy flags separately from the lists, but no claim says which layer's occupancy record carries the flag for a deck-marked aircraft at rest (the ground record, as for every landed aircraft so far, or the deck record), nor which record it clears at take-off.

The code is `air::set_mark` and `air::clear_mark`, which use the ground record of the aircraft's cell; the aircraft's listing alone follows the mark (`air::sync_listing`). The flag decides whether a landing, a placement or a vehicle entry to that cell is refused (S-0209 claims 3 to 6, S-0501 F7).

## Questions

1. When an aircraft rests on a bridge deck with the mark set, is its object-present flag set in the ground record, the deck record, or both?
2. At take-off from the deck, which record's flag is cleared, and when (the first climbing tick, or when the listing height becomes positive)?
3. Does a vehicle on the ground below the bridge, or on the deck, see that flag?

## Possible observation

S-0501 scenario AC27, recording both occupancy records of the landing cell at each tick, then driving a vehicle into the cell on the ground below and onto the deck.

## Current workaround

The flag stays in the ground record, set at touchdown and cleared when the aircraft leaves the ground, whether or not it is marked (`air::set_mark`, `air::clear_mark`). Marked `Q-0872 interim` in `docs/development.md`.
