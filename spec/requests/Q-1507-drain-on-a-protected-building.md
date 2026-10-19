---
id: Q-1507
title: Does force-shield or iron-curtain protection stop a drain link
status: answered
related: [S-0608, S-0513]
answered-by: [S-0608]
---

## Need
S-0513 claim 27 lists what protection refuses besides damage (mind control, domination, erasure,
warp, crushing, locomotor seizing, flinging, demolition charges, parasite weapons). It has no row
for a draining weapon. S-0608 claim 3 lists the firing check's drain refusals (the own current
building, a building already drained, a type that is not drainable) and has no row for a
protected target either. S-0513 claim 28 lets a player house fire at a protected object. Read
together, a drainer can link to a building under a force shield or the invulnerability effect,
and a link that already exists continues when protection starts. Neither spec says so in words,
and the backing of claim 27's list (scenario SW10 part c) does not name a drain. Our engine
decides this in `crates/deterministic/sim/src/drain.rs` (the attempt) and
`crates/deterministic/sim/src/combat/check.rs` (the firing check).

## Questions
1. A drainer fires its draining weapon at an enemy power plant that is under a force shield.
   Does the link form, so that the plant's house loses the plant's output?
2. The same with the invulnerability effect on the plant instead of a force shield.
3. A drainer is already linked to a building when that building's house launches a force
   shield over it. Does the link continue through the protection, end at once, or end when
   something else happens?
4. If a link cannot form on a protected building: does the drainer get the *illegal target*
   answer (it never fires), or does it fire and the attempt fail (and then, where does the
   drainer go)?

## Possible observation
Two houses. House B has a power plant; house A has a drainer near it. Launch B's force shield
over the plant, then order the drainer to attack the plant. Record whether it links, and B's
power. In a second run, link first, then launch the shield. Repeat both with the invulnerability
effect.

## Current workaround
The literal reading: protection refuses no drain. The firing check gives no *illegal* answer for
a protected target (S-0608 claim 3 and S-0513 claim 28), the attempt forms the link (S-0608
claim 5), and an existing link holds while the building is protected. Test:
`drain_pairs::a_shielded_plant_is_drained_under_the_q1507_interim` in
`crates/deterministic/sim/tests/pairs_m7b.rs`.
