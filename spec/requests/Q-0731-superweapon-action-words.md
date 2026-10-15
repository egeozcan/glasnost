---
id: Q-0731
title: Which aim action words exist, and whether an empty action field is no action
status: answered
related: [S-0512, S-0513]
answered-by: [S-0512]
---

## Need
S-0512 claim 23 and Import mapping say:
- the `Action` field is an action word, matched case-insensitively;
- `None` maps to no action;
- an unrecognised word sets no action.

They name only `Nuke`. Our importer must decide two things:
- which words are recognised, so that an unknown word can set no action;
- what an empty value does.

S-0513 claim 2's shared-action quirk compares actions, so this matters there too.

## Questions
1. Which action words does the game recognise?
2. Does an empty `Action=` value set no action, or keep the previous layer's action?

## Possible observation
Use named rules files that give a type an unknown word, an empty value and each known word in
turn. Record whether a sidebar click offers targeting, and which record a click fires (S-0513
scenario SW1 d).

## Current workaround
Interim rule:
- Every word other than `None` and the empty word is kept as its own action, lower-cased, so
  two types with the same word share an action.
- `None` and an empty value are no action.
