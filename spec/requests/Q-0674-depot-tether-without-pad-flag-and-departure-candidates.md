---
id: Q-0674
title: Depot tether without the pad flag, departure-candidate tests and the queued-entry immobilized check
status: open
related: [S-0314, S-0215, S-0404, S-0209, S-0211]
answered-by: []
---

## Need
S-0314 revision 39 answers Q-0680 to Q-0683, but three points it raises cannot be built
without guessing. The code is `depot::process_enter` (tether), `depot::exit_cell` and
`econ::release::present_exit_list` (claim 25) and `tick::object_turn` (claim 23's retry).

## Questions
1. **Tether at a depot without the pad flag (claim 17).** Without the pad flag the repeated
   entry request sets no tether, and "completed-entry confirmation also establishes a tether if
   one was not already present". The completed-entry exchange is described as something that
   "can turn a tethered vehicle toward 16384 … and report arrival". For an ordinary repair depot
   (no pad flag), what makes an untethered entering vehicle tethered? Does the
   docking-point-cell arrival exchange (repair at the depot, sleep at the vehicle) tether it, or
   does the vehicle stay untethered through waiting, settling and working? If it stays
   untethered, which of claim 20's docked-vehicle rules (contact kept under a move order, remote
   repair) still apply to it?
2. **"The first contacted vehicle's truncated spatial distance" (claim 17, pad flag).** Is
   this the distance from the foundation centre to the vehicle's own position, to its moving
   destination, or to the vehicle in contact place 0? Does "no moving destination" mean no
   destination at all, or not currently moving?
3. **Locomotion-specific cell-entry test (claim 25).** The door-kind and perimeter rows require
   the vehicle's "locomotion-specific cell-entry test" to give free as well as its cell-entry
   test of S-0209 claim 5 and S-0211. What does that second test check that the first does not
   (for a wheeled, tracked, hovering and ship vehicle)? Can it ever refuse a cell the first
   test calls free?
4. **A foundation size with no listed exit list (claim 25).** With
   `quirks.foundation_exit_lists` on, S-0404's Constants list exit lists for certain foundation
   words only. For a depot whose foundation has no listed word, is the list "present" (and
   what list is it), or absent, so that the perimeter row applies?
5. **Announcement on a failed departure (claim 6).** When the working phase's *done* or
   *not needed* step finds no departure cell and no rally point, is the repair-complete
   announcement still made at that step? The vehicle then returns to settling, whose later
   *not needed* is silent.
6. **Immobilized-state check (claim 23).** The queued-entry retry "is skipped on a turn
   excluded by the vehicle's immobilized-state check". Which states exclude it (a held
   parasite host, an EMP-disabled or deactivated vehicle, a vehicle under a magnetic lift, a
   vehicle whose locomotor is being replaced)? Where does the retry sit relative to an
   attached parasite's step and an open-topped transport's passenger sync in the same turn?

## Possible observation
On a tiny flat skirmish map with an invented repair depot type (no pad flag, a 3 × 3 and a
2 × 2 foundation), order a damaged invented vehicle to enter; log the vehicle's tether state
(through a move order given while it is in waiting, settling and working: does it keep
contact?), and the cell it is sent to when the first candidates are occupied. Repeat with the
pad flag set. For 6, queue a second vehicle behind the first and disable it while the first
leaves.

## Current workaround
- `depot::process_enter` (Q-0680 interim, kept): the vehicle is tethered once it stands in a
  foundation cell of the depot or in the docking point's cell. Removing that tether would leave
  an ordinary depot's vehicle never docked.
- `depot::exit_cell` (Q-0674 interim): the locomotion-specific test adds nothing to the entry
  test (`occupancy::vehicle_outcome` giving clear).
- `econ::release::present_exit_list` (Q-0674 interim): with the switch on, a foundation size
  without a listed word has no list, so the perimeter row applies; with the switch off, the
  ring alternative is the list.
- The repair-complete announcement is made at the working step even when the departure then
  fails.
- `tick::object_turn`: the retry runs after the locomotor movement of every ordinary living
  vehicle turn, before the passenger sync and parasite step; no state skips it.
