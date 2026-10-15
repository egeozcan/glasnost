---
id: Q-0649
title: Whether the whole-map reveal at game start leaves the border cells' surrounded marks clear
status: answered
related: [S-0409]
answered-by: [S-0409]
---

## Need

S-0409 (revision 11) claim 3 now says the start-of-game whole-map reveal marks as surrounded
each cell whose eight map neighbours are explored, with a non-legal neighbour using claim 14's
shared explored state, "so a border cell can be marked surrounded when that shared state is
explored". The consequences paragraph of the re-covering claim (the paragraph that begins
"Consequences. A cell explored by a sight or a plain reveal is always marked surrounded") still
lists "the reveal of claim 3 and of defeat, which leaves the cells on the border of the map's
diamond unmarked: S1 row 10" among the reveals that leave the mark clear. The two passages
disagree for border cells when the shared state is explored. The code
(`glasnost_sim::econ::shroud`) follows claim 3.

## Questions

1. After the whole-map reveal of claim 3 (and the reveal at defeat), is a border cell of the map's
   diamond marked surrounded when the shared off-map explored state is explored, as claim 3 says,
   or always left unmarked, as the consequences paragraph and S1 row 10 say?
2. If claim 3 is right, will the consequences paragraph, S1 row 10 and the Terms be revised to match?

## Possible observation

S-0409 scenario S1 row 10: a game with the lobby shroud setting off, then re-covering near the
map's border. Note whether the border cells erode at the first re-covering step.

## Current workaround

The code follows claim 3: the shared state decides a non-legal neighbour.
