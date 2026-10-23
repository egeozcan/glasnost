---
id: Q-1715
title: What a no-type entry does in a corpse list or a death-animation list
status: answered
related: [S-0107, S-0003]
answered-by: [S-0107]
---

## Need
M8 step A1 built S-0107 claim 17's corpse pick and row 3 (a type's own death-animation list),
which an arena game reaches at every infantry death (`docs/m8-claims/a1-arena-spread.md`). The
lists are read as S-0003 says: empty items are skipped (claim 12), and `none` or `<none>` means
"no type" (claim 14). S-0107 claim 17 row 3 says "entry 0 when entry *w* is empty", so a
death-animation list seems to keep a position for an entry that names no type. For the corpse
lists the spec does not say, and the length decides which entry the pick's draw selects.

Code: `crates/deterministic/sim/src/combat/death.rs` (`corpse`, row 3 in `infantry_death`), the
importer rows of `DeadBodies` and `DeathAnims` in `crates/convert/src/sim.rs` and
`crates/convert/src/sim_combat.rs`.

## Questions
1. In a type's death-animation list, does a `none` or `<none>` entry keep its position, so that
   the entries after it keep their death-kind indices, and is it the "empty" entry of row 3 (the
   unit then shows entry 0)?
2. In a corpse list (a type's own or the general one), does a `none` or `<none>` entry count
   towards the list length the pick's draw is reduced by? If it counts and is picked, is any
   corpse shown?
3. In either list, an entry that names an animation type the rules do not list: does it count
   towards the length (corpse list) or keep its position (death-animation list), and what does
   it show when chosen?

## Possible observation
An infantry type with `DeadBodies=A,none,B` (A and B two distinct corpse animations) killed many
times with different seeds: whether a death leaves no corpse about a third of the time. Likewise
`DeathAnims=A,none,B` with warheads of death kinds 1 and 2: whether kind 2 shows B.

## Current workaround
Death-animation list: every entry keeps its position; `none` and `<none>` (any case) count as
empty, so entry 0 is shown. Corpse lists: `none` and `<none>` entries are left out, as for the
other animation lists the importer reads (S-0003 claim 14), so they do not count towards the
length. In both lists an entry that names an animation type the rules do not list shows nothing
when chosen, and a corpse pick that chooses it still takes its draw.
