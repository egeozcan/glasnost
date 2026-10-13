---
id: Q-0100
title: Which words the factory field of a building type takes, and how they are compared
status: answered
related: [S-0403, S-0404, S-0407, S-0213, S-0214, S-0003]
answered-by: [S-0003]
---

## Need
S-0403 (Terms, Import mapping), S-0404 (Import mapping), S-0407 (claim 1,
Import mapping), S-0213 (Import mapping) and S-0214 (Import mapping) map the
building type key `Factory` to "the kind of object it makes: infantry,
vehicles, aircraft or buildings", with the parse form "type kind" or
"type-kind name (S-0003)". S-0003 defines no such form, and no approved spec
gives the words the value is compared with. The importer
(`convert::sim_econ`) therefore cannot tell which kind a building's factory
field names, so no imported building is a factory building: production,
primary factories and the rally rules have no factory buildings on real data.

## Questions
1. Which value words make a building a factory building of infantry, of
   vehicles, of aircraft and of buildings? (Stated as the words the importer
   must recognise, not as a stock value of any type.)
2. How is the value compared with them: letter case, the length of text read
   (S-0003 claim 1 or 16 limits), leading or trailing spaces?
3. What does a value that matches none of them give: not a factory, or the
   value in force kept (S-0003 claim 16's rule for named values)?
4. Does any other word name a kind that is not one of the four (for example
   a kind with no production line), and what does a building with it do?

## Possible observation
None needed for question 1 if it is read from the original's value
vocabulary; for questions 2 and 3, a test rules layer that gives a barracks
type its factory word in other letter case, with a trailing space, and a
nonsense word, then checks whether infantry can be trained.

## Current workaround
The importer leaves `Factory` unconverted and warns once per building type
that gives it; hand-written rulesets (tests, the synthetic ruleset) use our
own words `infantry`, `vehicle`, `aircraft` and `building` in the field
`factory`. On an imported install no building is a factory building until
this is answered.
