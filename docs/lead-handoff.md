# Lead handoff (2026-10-20)

This note is for the project lead's session that takes over, possibly on another computer. It is not
an implementer brief: implementer sessions work from PLAN.md, docs/development.md and the specs, and
should not act on this file.

## Where things stand

| Milestone | State |
|---|---|
| M0 to M6c | Done. |
| M7a (Yuri part 1) | Done 2026-10-18. |
| M7b (Yuri part 2, AI rounds 1 to 3) | Done 2026-10-20. Gate passes; the measurement arena (M-0001 to M-0003) still fails 55 of 155 comparisons, mostly because our seed-to-seed spread is too small. |
| Visual-fix pass "m7v" | Ran on 2026-10-20 to work through the lead's screenshot review of 2026-10-19 (278 of 280 screenshots seen: 213 OK, 37 issues, 28 unsure). Its per-item results are committed under docs/ ("item results" commits); new screenshots went to the git-ignored `local-lane/visual-fix-3/`. If its last helper work is not on main, the session ran out of usage before merging it: check `git worktree list` on the original machine. |
| M8 | Not started. Plan below. |

## Open spec requests

These are filed in spec/requests/ and wait for the spec side (see "Two machines" below). Each has an
interim rule in docs/development.md, *Visual review 2026-10-19*.

- Q-1529: whether the shrouded-centre-cell protection row reaches a building's slot animations (S-0513). Answered by S-0513 revision 30; taken in by M8 intake 3 (2026-10-21).
- Q-1562: how a particle type's colour list reads malformed triples (S-0202). Answered by S-0202 revision 33; taken in by M8 intake 1 (2026-10-21).
- Q-1590: whether a railgun system without a held particle type makes its laser (S-0616). Answered by S-0616 revision 16; taken in by M8 intake 2 (2026-10-21).

A request is answered when its spec's front matter lists it under `requests:` and the request's
status is no longer open. After any spec landing, an implementer catches the code up to the new
revisions (`git log -- spec/`).

## M8 plan (proposed, not yet briefed)

Proposed split, M8a, M8b and M8c:

- **M8a, fidelity:**
  - arena spread: find the randomness sources the original has and we lack, so the KS comparisons can pass;
  - the Psychic Dominator should actually take units on stock data;
  - turn the remaining `unmeasured` and `measurable` claims into measured ones;
  - movement and crowding feel.
- **M8b, AI depth:**
  - expansion via a second MCV should happen in full games;
  - superweapon saving should not cost tempo (it is blamed for Yuri's drop against Soviet);
  - gate margins that do not hinge on single seeds or quirks;
  - D14 holds: never weaken the AI for balance. Balance is an optional rules patch, like a mod.
- **M8c, polish:**
  - settings, accessibility, low-end iGPU performance, music;
  - specs for electric-bolt sparks and voxel-animation particle systems, which have no spec yet.

The user was asked whether to move the optional balance patch earlier and has not answered.

## Standing rules for the lead

- The user's direction: keep going autonomously through M7 and beyond; Opus for implementers and complex work.
- No git history rewrite. Commits are signed off (`git commit -s`). The GitHub repo stays private.
- Never redistribute EA assets. Screenshots and review pages stay local.
- Implementer briefs contain only milestone and spec IDs, the data path and lead notes. Implementers
  never read the spec workspace, disassembly, YRpp/Ares/Phobos/ModEnc, Chrono Divide or decompiled forks
  (PROVENANCE.md).
- Specs never state stock rule values. Obvious original bugs with no gameplay value are fixed outright.
  Gameplay quirks get a `quirks.*` switch defaulting to the original (PLAN D6). Bad data gets a safe
  deterministic result plus an importer warning.
- Measurements: only observables are committed; no harness code enters this repo.

## Two machines

The clean-room split means the spec workspace (decompilation, spec drafts, the round orchestrator) is
never pushed anywhere. To take over on another computer:

1. Clone this repo.
2. Copy `glasnost-lead-portable.tgz` from the original machine's data-drive backup folder
   (`/var/mnt/data/backups/glasnost/`) and unpack it in the home folder.
   - It holds the spec workspace, with its own `LEAD-HANDOFF.md`: setup steps on a new machine, the
     round mechanics and the prepared rounds.
   - It also holds the implementer briefs and logs, the last scrub reviews, the lead's memory notes and
     the implementer sandbox settings.
   - Keep it private, like the workspace itself.
3. Without the archive, a lead session can still brief and review implementer work, run check, test and
   gate, and file requests, but it cannot answer spec requests.

`docs/m7v-report.md` holds the visual-fix session's closing report once it has ended.
