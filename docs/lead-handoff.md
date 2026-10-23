# Lead handoff (2026-10-21)

This note is for the project lead's session that takes over, possibly on another computer. It is not
an implementer brief: implementer sessions work from PLAN.md, docs/development.md and the specs, and
should not act on this file.

## Where things stand

| Milestone | State |
|---|---|
| M0 to M6c | Done. |
| M7a (Yuri part 1) | Done 2026-10-18. |
| M7b (Yuri part 2, AI rounds 1 to 3) | Done 2026-10-20 except for M7's arena exit (PLAN.md): the gate passes, and the arena failures are worked on in M8 (A1, A5, the A3 intakes). |
| Visual-fix pass "m7v" | Done 2026-10-20. Its per-item results are committed under docs/; its closing report is `docs/m7v-report.md`. |
| M8 (fidelity, AI depth, polish) | Running, about half done. See [m8-plan.md](m8-plan.md) and the *M8 step ledger* in [development.md](development.md). |
| M9 (multiplayer) | Running alongside M8: N1 to N3 merged. See [m9-plan.md](m9-plan.md) and the *M9 step ledger*. |
| M10 (campaign, optional) | Not started. |

Figures at main 1c7527e:
- save format 69 and AI game format 30;
- the oracle arena fails 49 of 155 pairings (M-0001 33 of 96, M-0002 13 of 24, M-0003 3 of 35);
- 109 approved specs, 3 measurement records and 18 open requests.

### Step status

| Step | State |
|---|---|
| M8 A1, A2, A5, B1, B3, C2 | Merged. A5 missed its performance exit: the AI game is 15 to 20 % slower per tick, over the 10 % limit. C4 follows that up. |
| M8 B2 | Merged. Its exit was missed narrowly; B5 follows it up. |
| M8 A3 (spec intakes) | Recurring. Intakes 1 to 3 are merged (S-0202 r33, S-0616 r16, S-0111 r19, S-0513 r30, S-0519 r15). |
| M8 B5 (tempo after expansion), C3 (accessibility) | Running. |
| M8 B4, C4, C6 | Planned. B4 starts after B5. C4's exit needs the lead's run on an Intel laptop iGPU. |
| M8 C1 (music), C5 part 2 (voxel animations) | Wait for Q-1700, and for Q-1702. C5 part 1 (electric bolts, S-0316 answering Q-1701) is built. |
| M8 A4 (movement feel) | Waits for the lead's play notes or oracle records. |
| M9 N1, N2, N3 | Merged. The web client builds but has not yet been tried in a browser. |
| M9 N4 (desync reports) | Running. |
| M9 N5 to N8 | Planned in order. N8's exit is a 2v2 across networks that the lead plays. |

## Open spec requests

Requests are filed in spec/requests/ and wait for the spec side (see "Two machines" below). Each
request names its interim rule, and docs/development.md lists the interim rules. To list the open ones:
`grep -l '^status: open' spec/requests/Q-*.md`. On 2026-10-21, 18 were open:

- **Visual review and rendering:**
  - Q-1600: shot animation against a turreted voxel vehicle's barrel;
  - Q-1740: the slope height in the slot-animation shroud test.
- **Repair depot:** Q-1610 and Q-1620.
- **M8 plan:**
  - Q-1700: music (blocks C1);
  - Q-1702: voxel animations (blocks C5 part 2; Q-1701, electric bolts, is answered by S-0316);
  - Q-1703: the oracle measurement round.
- **Arena spread (A1):**
  - Q-1710 to Q-1712;
  - Q-1713 and Q-1714: how two arena fights end;
  - Q-1715: no-type entries in the corpse and death-animation lists.
- **Animation playback (A5):** Q-1716 and Q-1717.
- **M9:**
  - Q-1805: house order with several people and random colours;
  - Q-1806: the options menu and game speed in a network game.

Q-1529, Q-1562, Q-1590, Q-1621 and Q-1622 were answered and taken in by M8 intakes 1 to 3
(2026-10-21). Q-1621's file still says `status: open`, although its `answered-by` names S-0519 and
S-0519 revision 15 answers it. The spec side should correct the file: implementers do not edit
existing request files.

A request is answered when its spec's front matter lists it under `requests:` and the request's
status is no longer open. After any spec landing, an implementer catches the code up to the new
revisions (`git log -- spec/`). Then run both quirk coverage tests:
- `glasnost-rules` `every_spec_quirk_has_a_switch`;
- `glasnost-app` `mapview::quirks::tests::every_rendering_spec_quirk_has_a_switch`.

## Waiting on the user

- C4: a frame-rate run on a typical Intel laptop iGPU in Firefox (WebGL2).
- N8: the cross-network 2v2.
- A4: play notes on movement and crowding.
- The optional balance patch: the user was asked whether to move it earlier and has not answered.
  D14 holds: the AI is never weakened for balance.

## Standing rules for the lead

- The user's direction (2026-10-21):
  - work autonomously to the end of M9;
  - Opus for design-heavy steps, Sonnet for well-defined ones, Haiku for trivial ones;
  - push main after every merge and ledger row;
  - keep the status dashboard current and notify the user.
- Only the lead edits the M8 and M9 step ledgers, at merge. Agents give their row text in their
  reports.
- Agent briefs live in `.claude/worktrees/` (untracked): `COMMON-BRIEF-M8.md` plus one brief per step.
- Agents work in their own git worktrees. They never run `cargo clean` and never delete another
  worktree's `target/`. Watch the disk: a worktree's target takes 15 to 30 GB.
- No git history rewrite. Commits are signed off (`git commit -s`). The GitHub repo stays private.
- Never redistribute EA assets. Screenshots and review pages stay local (`local-lane/`, git-ignored).
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

### The spec loops

Since 2026-10-21, two autonomous sessions answer spec requests on the original machine:
- a speccer drafts the specs;
- a scrubber reviews them and pushes `spec:` commits straight to origin/main.

Both run in a bubblewrap sandbox, in their own tmux sessions (`speccer`, `scrubber`).
- **Start:** `~/.local/share/glasnost-loops/start-loops.sh`. The user restarts them after a reboot.
- **Pause:** `touch PAUSE` in the loop's workspace.

The lead session never messages, attaches to or reads those sessions. It only pulls their commits
(`git fetch origin`; merge spec commits fast-forward before merging an agent branch).
