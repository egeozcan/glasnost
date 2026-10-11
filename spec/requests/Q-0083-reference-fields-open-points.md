---
id: Q-0083
title: Open points of the reference fields that create types (art-file fields, endless creation, numbered weapons above 18)
status: open
related: [S-0004, S-0306, S-0007, S-0308]
answered-by: []
---

## Need
`convert::layers` now implements S-0004 claim 13 and its Import mapping
tables: every creating field of every section kind and game-wide section
creates the types it names, in the rows of claim 4, and those types read their
sections by claim 6. Four points are not settled by the approved specs, and
the importer carries a placeholder for each (docs/development.md, "Open
requests and their placeholders").

## Questions
1. **Where a projectile's trailer animation is read.** S-0004 claim 13 and
   its Import mapping list the trailer animation of projectiles (`Trailer`)
   among the fields read from the art file. S-0306's Import mapping lists
   `Trailer` under "rules, projectile". Which file does the game read a
   projectile's trailer animation from? Does the same answer hold for an
   aircraft's `Trailer`, which S-0004 also places in the art file?
2. **Which art section the three art-file fields of other kinds are read
   from.** S-0004 says a building's convert-to-overlay field is read "in the
   section named by the building's `Image`". For aircraft and projectiles it
   says only "the art file".
   - Is it the section named by the image name in force once the type has read
     its rules section in that layer (the type's own name when no layer gives
     an image), for all three kinds?
   - On a snow map, does the arctic suffix of S-0007 claims 9 and 10 apply to
     this section name? If so, which types a building creates through its
     convert-to-overlay field would depend on the theater.
3. **A field that names its own kind with a name longer than 24 bytes.** By
   claim 8, such a name never finds the type created from it. Take a type
   created during its own kind's pass (claim 6) whose section names that long
   name again, for example an animation whose art section `[X]` (24 bytes)
   has `Next=` followed by those 24 bytes plus more text. Then each new copy
   reads the same section and creates the next copy. What does the game do:
   does it stop at some point, refuse to start, or hang or crash?
4. **Numbered weapon fields above 18.** S-0004 claim 13 says that a type with
   a turret count of 1 or more takes effect with numbered weapon fields 1 to
   *c*, its weapon count. S-0308's departure reads only the first 18 numbered
   fields into weapon slots. For registration, does a type with a weapon count
   above 18 create the weapons that `Weapon19` and later fields name (and
   their elite forms), as S-0004's text says?

## Possible observation
- Question 1: a test map whose projectile gets a trailer animation only in the
  rules section, and a loose art copy that gives it one only in its art
  section. Which trailer is drawn?
- Question 2: as scenario G3, but the building's rules section sets its image
  to a second art section that holds the convert-to-overlay field, played once
  on a temperate map and once on a snow map with the arctic flag set.
- Question 3: the animation case above, in a loose art copy.
- Question 4: a vehicle with turret count 1 and weapon count 19 whose
  `Weapon19` names a new weapon that only it names, with a section whose
  warhead names a new overlay-creating animation (any index-visible effect of
  claim 12).

## Current workaround
1. The trailer animations of aircraft and projectiles are read from the art
   file, as S-0004 says. `convert::sim` keeps reading the projectile's
   `Trailer` from the rules section for its record, as S-0306 says.
2. All three art-file fields are read from the art section named by the
   type's image in force after the rules section was read (the stored name
   when no layer gives an image). The arctic suffix is not applied.
3. A departure that stops the loop. A creation is skipped, with an importer
   warning, when the type would read its section in the same layer and some
   type in the chain of creators of the referring type was created in this
   layer, is of the same kind and was created by the same name. So the loop
   stops after one copy per layer.
4. Numbered fields 1 to *c* create weapons, as S-0004 says. Only the keys
   present in the section are considered, so a very large *c* costs nothing.
