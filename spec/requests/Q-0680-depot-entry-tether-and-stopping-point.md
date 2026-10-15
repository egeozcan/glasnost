---
id: Q-0680
title: When a vehicle approaching a repair depot becomes tethered, and where it stops on an off-centre docking point (S-0314 claims 17 and 18)
status: answered
related: [S-0314, S-0215, S-0504, S-0207]
answered-by: [S-0314]
---

## Need
S-0314 claim 17 and S-0215 claim 29 describe the completed entry exchange: the vehicle turns
toward 16384 facing units and, once no longer moving, tethered to the depot and still in
*enter*, reports arrival. Claims 20 to 23 then split vehicles in contact into *approaching*
(contact, no tether) and *docked* (tethered). No claim says at which moment of the approach the
tether is made. The engine needs it in the vehicle's *enter* processing (`depot::process_enter`)
and for every claim 20 decision (an order keeps or drops the contact by it).

Claim 16 also refuses acceptance to "a vehicle already standing on this very depot", and S-0215
claim 29 makes a vehicle without a tether fall back to rest on any non-accepting reply to its
repeated entry request. Read together, an approaching vehicle whose path crosses a foundation
cell of the depot before it reaches the docking point would be refused at its next *enter*
processing unless it is tethered by then.

Separately, claim 18's docking point is the foundation centre plus the imported displacement,
which need not be a cell centre (for example the centre of a 2 × 2 foundation is a cell corner).
A driving vehicle in our engine ends its move at a cell centre (S-0207), so for such a point the
waiting check of claim 6 (strictly below 100 leptons) may never pass.

## Questions
1. At which moment of the approach does a vehicle with depot contact become tethered: when it
   enters any cell of the depot's foundation, when it reaches the cell of its docking point, when
   it comes to rest there, or at some other observable moment?
2. Does the "already standing on this very depot" refusal of claim 16 apply to the repeated entry
   request of an approaching vehicle in *enter* (S-0215 claim 29), or only to a new click and the
   queued-entry retry of claim 23?
3. Where does a driving vehicle come to rest when its docking point is not the centre of a cell:
   at the docking point itself, at the centre of the cell that holds it, or elsewhere? Does the
   ship locomotor differ?

## Possible observation
H16 (a, c and e) with a test depot whose docking displacement is not a cell centre, recording
position, contact and tether each tick of the approach.

## Current workaround
Interim rule, recorded in docs/development.md: the vehicle is tethered in its *enter* processing
once it stands in a foundation cell of the depot or in the docking point's cell; the
standing-on-the-depot refusal is applied to the click and the queued-entry retry only. The
vehicle drives to the docking point's cell and ends at that cell's centre, as every driving move
does. The synthetic tests use docking points at cell centres.
