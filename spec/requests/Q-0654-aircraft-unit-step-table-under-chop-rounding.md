---
id: Q-0654
title: Which result a 1-lepton aircraft step has at facings just past south and just past west under round-toward-zero
status: open
related: [S-0501, S-0205, S-0208, S-0105]
answered-by: []
---

## Need

S-0501 revision 10, F3, gives a table of the coordinate change of a 1-lepton forward step by body
facing. It says the table follows from F3 "with S-0105's table values". The opening note of the
spec says its scenarios use every quirk switch in its default state. That includes
`quirks.chop_rounding` (S-0205 claim 5): every arithmetic result is rounded toward zero.

Built literally from S-0208 F3, S-0105 F1 and S-0205 claim 5, two rows of the table come out
differently under that default. Our engine agrees with the table at every facing only when
`quirks.chop_rounding` is off.

| Facings | S-0501 F3 table | Our engine, `chop_rounding` on (default) | Our engine, off |
|---|---|---|---|
| 32767 to 32772 | (0, +1) | (−1, +1) | (0, +1) |
| 49148 to 49155 | (−1, 0) | (−1, −1) | (−1, 0) |

At every other facing the two modes agree with each other and with the table. That includes the
stall range 16391 to 32766, so `quirks.aircraft_unit_step_stall` is not affected.

Why, from the spec's formulas, starting at (5000, 5000) with *s* = 1:

- Facing 32767: θ = 16384 × *k*. S-0105 F1 gives table index σ = 6144 for the sine, so sin = −1.
  The cosine is τ(8192), the exception −2.4492127076447545 × 10⁻¹⁶. The x component is
  1 × τ(8192), slightly negative. Rounded toward zero, 5000 + that component is
  4999.9999999999990905…, the double just below 5000. Truncation then gives 4999. Rounded to
  nearest, the sum is 5000 and the coordinate does not change. The y component is exactly +1.
  Facings 32768 to 32772 have the same σ and the same result.
- Facing 49148: σ = 4096, cos = τ(6144) = −1, and sin = τ(4096) = 1.2246063538223773 × 10⁻¹⁶.
  The y component −1 × τ(4096) is slightly negative. Rounded toward zero, 5000 + that component
  is again 4999.9999999999990905…, so y truncates to 4999. Rounded to nearest, it stays 5000.
  Facings 49149 to 49155 have the same σ.

This is not limited to 1-lepton steps. Any step at those facings, of any length, moves the
coordinate one extra lepton under the default. It changes AC24 (b3) in our engine. From the north,
11 leptons west of the column, the aircraft drifts west on each step at facings 32767 to 32772.
It still hovers at *d* = 166 at a facing between east and south, but at facing 29778 and
47 leptons west of the column, not at about 32076 with the side distance 11. With
`chop_rounding` off, (b3) matches the spec: facing 32076, side distance 11. (b1), (b2) and (b4)
give the same result in both modes.

## Questions

1. Under the default round-toward-zero arithmetic, does a 1-lepton step at a body facing from 32767
   to 32772 move the aircraft one lepton south only, or one lepton south and one lepton west?
   And at 49148 to 49155, one lepton west only, or one lepton west and one lepton north?
2. Does the original apply round-toward-zero to the addition x + *s* × cos θ in the aircraft's
   forward step (S-0205 claim 5's scope)? Or is that step, or this addition, computed in a way
   that leaves a coordinate unchanged when the component is a tiny negative value?
3. Which governs if they disagree: S-0501 F3's table, or S-0208 F3 with S-0105 F1 under S-0205
   claim 5? Does AC24 (b3) expect the facing about 32076 and the side distance 11 under the
   default?

## Possible observation

Fly AC24 (b3), from the north and 11 leptons west of the flight point's column, and log the
position each tick. A westward drift of one lepton per tick while the body facing is 32767 to
32772 confirms our engine. A side distance that stays 11 confirms the table. A move order due
south and one due west would also settle it: log whether x changes on the southward flight and
whether y changes on the westward flight.

## Current workaround

The engine follows S-0208 F3, S-0105 F1 and S-0205 claim 5 literally: under the default the
coordinate takes the extra lepton. No code change. The test `f3_unit_step_by_facing` checks the
full table with `chop_rounding` off, and with it on checks every facing except these two ranges.
`claim_5_unit_step_hover_between_east_and_south` does not check (b3)'s facing or side distance
under the default. `claim_5_unit_step_hover_from_the_north_without_chop` checks them with
`chop_rounding` off.
