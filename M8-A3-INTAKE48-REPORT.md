# M8 A3 intake 48: S-0111 r25 (Q-2071, Q-2072)

Built (view and importer only; no sim change, no format/golden/arena change):
- Claim 11 spawn row: `pip_scale::spawn_row` packs the f full pips (slots Docked or Reloading) from pip 0. Test `claim_11_spawn_and_occupant_rows`.
- Claim 11 occupant row: code already matched (list position k, own frame, else 6); comments now cite the spec. Sim already closes up the list (S-0508 claim 23).
- Claim 15 negative wrap: `ammo_row` returns no pips for w < 0; importer warns on negative PipWrap. Tests `claim_15_ammo_row`, `pip_scale_warnings`.
- Import mapping unknown PipScale word: already kind none plus warning; comments now cite the spec.
- Ledger docs/m8-claims/intake48-S-0111.md; intake46 rows and Q-2071/Q-2072 notes updated.
Seams: OV10 b/c and OV15 not run (lead measures). Requests: none (Q-2106..2110 unused).
Checks: fmt, clippy -D warnings, cargo test --workspace and glasnost-app --lib green; xtask check fails only on the 16 expected stock-guard problems. Arena/sweeps not run (no sim change).

Ledger row: A3 intake 48: S-0111 r25 in. Spawn pips packed from pip 0, negative PipWrap shows no ammo pips and warns, unknown PipScale and occupant order cite the spec; Q-2071 and Q-2072 interims retired. View and importer only, no format change. Ledger docs/m8-claims/intake48-S-0111.md.
