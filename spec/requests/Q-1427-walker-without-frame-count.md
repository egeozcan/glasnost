---
id: Q-1427
title: Removal of a burning walker whose sprite frame count is unknown or too small
status: open
related: [S-0615, S-0109]
answered-by: []
---

## Need
S-0615 claim 11 removes an expiring walker when its stage is at least ⌊N / 2⌋ − 1, with N its
sprite's image-frame count. The simulation reads N from the art data; when the art gives none (no
art section, or a frame count the import could not find), or N / 2 − 1 is at most the starting
stage 8R + 1, the rule removes the walker at its first expiring update. The spec's open questions
list zero running frames but not this case. Code: `crates/deterministic/sim/src/walker.rs`, `turn`.

## Questions
1. A burning-walker type whose sprite has fewer frames than 2 (8R + 2): is it removed at the first
   update after expiry starts, as the formula gives, and does it deliver any damage over time?
2. With no frame count known, what count does the game use?

## Possible observation
S-0615 scenario FW3 with a walker type whose sprite has 8R + 4 frames, and with a type whose art
section is missing.

## Current workaround
N is 0 when not known, so the walker is removed at its first expiring update with no damage.
