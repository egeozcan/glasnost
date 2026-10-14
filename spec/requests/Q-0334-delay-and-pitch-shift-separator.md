---
id: Q-0334
title: What separates the two numbers of a sound entry's delay and pitch shift
status: open
related: [S-0701]
answered-by: []
---

## Need

S-0701 claim 2 reads the *delay* and *pitch shift* keys as "one or two integers *a* [*b*]", with
*b* = *a* when only one is given. For the sample list and the control and placement words, the
claim says which characters separate the items: spaces, tabs or newlines, and not commas. It says
nothing about the separator between the two integers. The sound data reader in
`glasnost_audio::catalog` has to split these values somehow.

## Questions

1. Which characters separate the two numbers of the delay and pitch shift values? Spaces, tabs,
   commas, or some combination?
2. What do these values give:
   - `40,80`
   - `40, 80`
   - `40 x 80`
   - three numbers, as in `10 20 30`
3. Does a number with trailing text, such as `40ms 80`, count as its leading digits, or does it
   make the value unreadable?

## Possible observation

Give one entry a pre-delay with `Delay=400,800` and another with `Delay=400 800`. Play each many
times and record the start latencies (S-0701 scenario E1). Also give a pitch shift written
`FShift=-50,-50` and check whether the sample plays at half speed.

## Current workaround

`catalog::pair` splits these values on spaces, tabs and commas. It reads the first two pieces as
integers through S-0003's integer form. A missing or unreadable second number repeats the first.
An unreadable first number leaves the value in force. Further pieces are ignored. Documented in
docs/development.md, "Audio (M5b)".
