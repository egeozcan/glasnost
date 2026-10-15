---
id: Q-0712
title: What happens to a dropped object's deck occupancy record
status: answered
related: [S-0511, S-0209]
answered-by: [S-0511]
---

## Need

S-0511 claim 9 moves every object listed in a lost cell's deck layer "to the ground layer of the
same cell". Claim 11 then checks each blown-up cell "whose deck-layer occupancy record (S-0209) is
not empty", and its scenario B6 expects that check to pass when a stationary deck unit was in
the cell. So the dropped unit's marks (an infantry standing position, a vehicle's present flag)
seem to stay in the deck record at least until claim 11's check. The spec does not say what
becomes of them afterwards. If they stayed for ever, a later repair (claim 19) would put the deck
back with stale marks that no unit owns.

The code in `crates/deterministic/sim/src/bridges/blow.rs` (`drop_from_deck`, `transfer_marks`) and `fall.rs` needs the rule.

## Questions

1. After a drop, do the dropped object's marks stay in the cell's deck record, and until when?
2. When the object has landed in the ground layer, does it mark the ground record of its cell (as
   an object entering it), and from which moment?
3. After the bridge is repaired, is the deck record of a cell whose object fell from it empty?

## Possible observation

S-0511 scenario B6 with a stationary deck infantry unit: after its fall, repair the bridge and
order a vehicle onto that deck cell. A vehicle refused or delayed as if the cell were occupied
shows a stale deck record.

## Current workaround

At the drop the object's list entry moves from the deck list to the ground list; its marks stay
in the deck record until the attempt's claim 11 checks are done, and then move to the ground
record of the same cell. The choice is marked in the code with this request's ID.
