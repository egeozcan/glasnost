# S-0111 r25: claim ledger, M8 step A3 intake 48

Takes in S-0111 r21 to r25 (spec commits 98f1420, b9ebdaa, 574ef85), which answer Q-2071 and Q-2072.
No simulation change; no format, golden or arena change.

| Claim | Status | Tests |
| --- | --- | --- |
| 11 spawned-aircraft row (first f pips frame 1, rest 0; f = slots docked or reloading; packed from pip 0) | tested: `pip_scale::spawn_row` packs; `live.rs` counts `Docked` and `Reloading` slots (S-0506 claim 4) | `claim_11_spawn_and_occupant_rows` |
| 11 occupant row (pip k = occupant at list position k, own type's frame, else 6; a dead place closed up, S-0508 claim 23) | tested: unchanged code (list order, empty 6); the closing up comes from the sim's list. Its comment now cites the spec | `claim_11_spawn_and_occupant_rows`, `a_building_occupant_row` |
| 12 / Import mapping: unknown PipScale word gives kind none, row length 0, importer warns (none itself does not) | tested: already so; comments now cite the spec | `pip_scale_warnings` (Cart `sideways` warns), `named_values` |
| 15 negative wrap: no ammunition pip; importer warns | tested: `ammo_row` returns no pips for w < 0; `pip_scale_checks` warns | `claim_15_ammo_row`, `pip_scale_warnings` |
| OV10 b/c, OV15 | seam: measurable by the lead, not run |  |
| Cross-spec hand-off to S-0506 R5 | seam: none for the view; it reads slot states, not pips |  |

Q-2071 and Q-2072 interims are retired. No new requests (Q-2106 to Q-2110 unused).
