---
id: Q-3118
title: What happens to a crate place whose crate another rule removed before it expires
status: answered
related: [S-0520, S-0524, S-0303]
answered-by: [S-0520]
---

## Need
S-0520 claim 1 says that removing a crate through pickup or expiry frees its place, and claim 4
says that each expiry removes one crate and adds at most one random crate. A crate (Terms) is a
cell holding a crate overlay. Other rules can remove a crate's overlay without a pickup or an
expiry: S-0524 claim 9's restoration of a missing starting building removes every overlay on its
foundation cells, and S-0303 claim 19 removes an overlay whose type has the explodes flag (a
mod can give that flag to a crate overlay type). No spec says what then happens to the crate's
place. In our engine the place stays occupied, pointing at a cell with no crate, until its
lifetime ends (`crates::update` in `crates/deterministic/sim/src/crates.rs`, issue #284).

By the time it expires, the cell may hold another overlay: a wall, a resource, or a newer crate
that holds a place of its own.

## Questions
1. When a rule other than pickup or expiry removes a crate's overlay, is the crate's place freed
   at that moment, or does it stay occupied until its lifetime ends?
2. If it stays occupied: at its expiry, with no crate on its cell, does the expiry still make its
   replacement random placement (claim 4), so that a crate that was already gone is replaced?
3. If it stays occupied and the cell holds a newer crate with a place of its own when it expires:
   does the expiry remove that newer crate (leaving the newer place pointing at an empty cell), or
   leave it alone?
4. Does the answer differ for a cell that an overlay of another kind (a wall, a resource) took
   over in the meantime?

## Possible observation
S-0524 scenario for action 130 with the crate setting on: a crate on a foundation cell of a lost
starting building, removed by the restoration; count the crates on the map, the replacement draws
and the overlay on that cell at the crate's expiry tick, once with the cell left empty, once with
a wall built on it and once with a later crate placed on it.

## Current workaround
The place stays occupied until its lifetime ends, as before. At its expiry the cell's overlay is
removed only when the cell holds a crate (`crates::is_crate`), so a wall or a resource on it is
left alone (S-0520 Terms, claim 4: an expiry removes a crate). The replacement placement is still
made. A newer crate with its own place on that cell is still removed by the older place's expiry
(question 3).
