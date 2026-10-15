---
id: Q-0730
title: The exact words a superweapon type's kind and link fields accept
status: open
related: [S-0512, S-0513]
answered-by: []
---

## Need
S-0512 Import mapping lists the twelve superweapon kinds in kind-number order, for the `Type` and
`PreDependent` fields, by description ("multi-missile kind", "iron-curtain kind" and so on). It
does not give the literal words the rules files use. It also does not say how they are matched
(case, hyphens, spaces). Our importer (`superweapons::kind_of_word`) must turn the field's text
into a kind.

## Questions
1. What literal word does each of the twelve kinds use in the `Type` and `PreDependent` fields?
2. Is the match case-insensitive? Are other spellings, with or without hyphens or spaces,
   accepted?
3. What does an unknown word do: give no kind, or keep the previous layer's kind?

## Possible observation
Use named rules files that give one type each spelling variant of a kind word. For each, record
whether the icon appears, which ready announcement plays and what the launch does.

## Current workaround
Interim rule (docs/development.md, *Simulation (M6b: superweapons)*):
- Take the table's descriptions with "kind" dropped and hyphens, spaces and underscores
  stripped, for example `ironcurtain`, `lightningstorm` and `americanparachutedrop`.
- Match them without regard to case.
- An unknown word gives no kind, and the importer warns.
