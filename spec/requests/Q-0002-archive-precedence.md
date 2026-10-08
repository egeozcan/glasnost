---
id: Q-0002
title: Which copy of a file the game uses when several archives hold it
status: open
related: []
answered-by: []
---

## Need
The virtual file system mounts every archive in the install folder and the
archives nested in them, then looks files up by name. The same name often
occurs in several archives, so lookups need an order. PLAN.md decision D5
already fixes one case (the single highest-precedence copy of the expansion
rules file wins and copies are never merged), but the general order is not
specified. Code gap: `glasnost_vfs::precedence` (the default order is a
placeholder of our own) and, later, the importer's choice of rules, art and
map files.

## Questions
1. When two archives in the install folder hold the same file name, which
   copy does the game use? Does that depend on the archive file names
   (base-game versus expansion archives, numbered expansion archives, map
   packages, cache archives), on load order, or on something else?
2. How do archives nested inside other archives rank against their parent and
   against other top-level archives?
3. Do loose files in the install folder take priority over files inside
   archives? Does that hold for every file type, or only some (rules files,
   sprites, maps)?
4. When one archive lists the same name twice, which entry is used?
5. Are some archives used only in some modes (single player, skirmish,
   multiplayer, a particular theater) so that their files are not visible
   otherwise?
6. Does the base game's set of archives (without the expansion) take part when
   the expansion is played, and if so, below or above the expansion's
   archives?

## Possible observation
Place two harmless, different copies of one small text or palette file in two
archives (and once as a loose file), start a skirmish, and see which copy
shows; repeat with the archive names swapped.

## Current workaround
`PlaceholderPrecedence`: mount order (top-level archives sorted by
case-insensitive file name, each followed depth-first by its nested
archives); the first archive holding the name wins; within one archive the
first entry in file order wins; loose files are never used. Callers can plug
in another order (`NamedPrecedence`). Nothing in the importer may depend on
the placeholder.
