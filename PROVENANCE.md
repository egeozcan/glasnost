# Provenance

This file describes how knowledge about the original game's behaviour reaches this repository. The project uses a documented two-team process and makes no certified clean-room claim.

In short: a **spec side** studies the original game and writes plain-language behaviour specs. Every spec passes a **scrub** before it enters `spec/`. The **implementation side** works only from `spec/`, `measurements/` and this repository's own code.

## 1. Roles

| Role | Who | May read | Never reads |
|---|---|---|---|
| **Spec agent** | An AI agent session working in a separate, private workspace (`~/glasnost-spec-dirty`, never pushed anywhere) | The stock, hash-checked game executable and its decompilation; YRpp, Ares and Phobos source and docs; ModEnc; OpenRA; XCC; the project lead's earlier modding repository | This repository's code (it reads only `spec/`, for the template, topics, glossary, existing specs and requests). It **writes** only draft specs and proposed measurements, in its own workspace; its raw notes never leave it. |
| **Scrubber** | Automated lint, then a fresh AI session with no access to the spec workspace, then the project lead's sign-off | The draft spec text and the lint report | Raw notes, decompiler output, the spec workspace |
| **Oracle operator** | The project lead, plus scripted harnesses | The running stock game, the measurement harness, the spec side | — (does not write engine code) |
| **Implementer sessions** | Isolated AI (Claude Code) sessions in this repository only | `spec/`, `measurements/`, this repository's code and history, public format docs, OpenRA and XCC source (GPL-3, for file formats and loaders, with attribution) | The spec workspace; the project lead's modding repository; YRpp, Ares and Phobos (source and docs); ModEnc; disassembly or decompilation of the game; the Chrono Divide client; decompiled browser ports of the game |

The project lead is the spec owner, oracle operator and reviewer/approver. The lead does not write engine code, and there is no second human implementer. Future human contributors work under the implementer rules and state any prior exposure to the sources in the right-hand column (see [CONTRIBUTING.md](CONTRIBUTING.md)).

Banned on **both** sides: the Chrono Divide client and the decompiled browser ports of the game (huangkaoya/redalert2, OpenRA2/RA2Web, ammaarreshi's port).

### How the lead talks to implementer sessions

- Every instruction cites a spec ID (`S-xxxx`), a measurement ID (`M-xxxx`) or a spec request (`Q-xxxx`).
- No "do it the way the original does internally", and no numbers without a citation.
- Reviews judge code against tests and behaviour, not against memory of the original's internals.
- Any exception is logged in `spec/incidents/` (section 8).

## 2. Spec style

A spec describes **what the game does**, not how the original program is built. The test: two different correct implementations must both satisfy the spec. If only a copy of the original's structure satisfies it, rewrite it.

### Allowed

- Observable behaviour, in our own vocabulary (see `spec/glossary.md` once it exists).
- Formulas in mathematical notation, with the rounding rule stated.
- Constants with units: ticks, leptons (256 per cell), per-mille, credits.
- State machines as tables of state, event and result.
- Ordering stated as an observable consequence.
- Edge cases and the original's bugs, described as behaviour. These become named quirk switches that default to the original behaviour.
- Rules-file key names, **only** when describing how the importer maps the user's files into our format.
- For each claim: a confidence level and its backing (`measured` with an M-ID, `measurable` with a proposed scenario, or `unmeasured`).

### Forbidden

- Addresses, offsets, vtable slots, struct layouts, field order or sizes.
- Class, function, field or enum names from the executable, YRpp, Ares or Phobos.
- Pseudo-code, C-like syntax, or step lists that follow one function's control flow.
- Decompiler output, or quotes from it, even "for context".
- Organisation by code module. Specs are organised by game topic (combat, movement, production, economy, AI-visible rules, and so on).
- Text copied from manuals, strategy guides or wiki prose. Describe facts in your own words.
- Stock rules values. Specs refer to imported values by role ("the weapon's reload time"), never by the stock number.

### Examples

(Every name and number below is invented.)

| Bad | Good |
|---|---|
| "`WidgetClass::DoThing` at address `0x…` calls `FrobFactor()` then sets `this->Gauge`." | "When a shot hits, the target loses health equal to the warhead's damage multiplied by the warhead's modifier for the target's armour class." |
| "`if (frame % 15 == 0) { ammo++; }`" | "A unit with partial ammunition regains one round every *reload interval* ticks, counted from the moment it stopped firing." |
| "The routine loops over the list and breaks on the first match." | "If several targets qualify, the closest one is chosen; ties go to the one that entered range first." |
| "Damage is computed as `(int)(dmg * mult)`." | "damage = ⌊base × modifier⌋, truncated toward zero after the multiplication; the result is at least 1 unless the modifier is 0." |
| "The combat routine at `0x…` handles both shots before the death check." | "If two units kill each other in the same tick, both die: damage from all shots in a tick is applied before any unit is removed." |

## 3. Scrub pipeline and sign-off

1. **Draft.** The spec agent writes a draft in its own workspace, with its raw notes kept separate. Only the draft text moves on.
2. **Lint.** An automated check rejects:
   - any `0x` hexadecimal number (short values are written in decimal, file-format magic numbers excepted), and any bare 6-to-8-digit hex pattern;
   - C-like tokens (`->`, `::`, `if (`, braces, lines ending in `;`);
   - every identifier on the denylist of executable, YRpp, Ares and Phobos names. The denylist itself lives only in the spec workspace.
3. **Scrubber review.** A fresh AI session with no access to the spec workspace reads the draft. It rewrites anything code-shaped into behavioural prose, applies the two-implementations test, and rejects drafts that fail.
4. **Sign-off.** The project lead reads the scrubbed spec and approves it. The approval is recorded in the spec's front matter (`status: approved`, `approved-by`, `approved-on`).
5. **Commit.** The approved spec is committed to `spec/<topic>/S-xxxx-short-name.md` in its own commit, with message `spec: add S-xxxx <title>` (or `spec: revise S-xxxx to rN`).

Revisions follow the same pipeline. Editing an approved spec without a new scrub and sign-off is not allowed.

## 4. Spec IDs and file format

- IDs are `S-` plus four digits (`S-0001`), never reused. The spec side allocates them in blocks of 100 by area (for example `S-00xx` import and INI semantics, `S-03xx` combat), so gaps in the numbering are normal. A retired spec keeps its ID with `status: retired`.
- One spec per file: `spec/<topic>/S-0001-short-name.md`.
- Each spec has a `revision` number. Code cites the ID only; the spec's history shows which revision is current.

Front matter (YAML), see [spec/TEMPLATE.md](spec/TEMPLATE.md):

```yaml
id: S-0001
title: Ticks per second at each game speed
topic: simulation          # simulation | combat | movement | production | economy | ...
revision: 1
sources-class: [decomp, measurement]   # any of: decomp, measurement, modenc, openra, xcc, format-doc, design
status: approved           # draft | scrubbed | approved | retired
approved-by: project lead
approved-on: 2026-10-08
measurements: [M-0001]     # M-IDs that back this spec; may be empty
requests: []               # Q-IDs this spec answers
supersedes: []             # S-IDs this spec replaces, if any
```

`sources-class` says what kind of source the facts came from, never the source itself. `design` marks decisions of our own that do not describe the original (for example our rules format).

## 5. Measurement IDs and file format

- IDs are `M-` plus four digits (`M-0001`), never reused.
- One record per file: `measurements/<topic>/M-0001-short-name.md`, see [measurements/TEMPLATE.md](measurements/TEMPLATE.md).
- A record holds the scenario, the game settings, the method, the sample size, the result and its tolerance, and the specs it confirms or contradicts.
- **The measurement wins.** A spec that contradicts a measurement goes back to the spec side for revision.
- When a measured value is the same thing as a stock rules value (for example a weapon's fire interval), the record commits the scenario, method and the *relation* it confirms, and keeps the value itself in a local-only file (`value-visibility: local`). Values that are not rules data (tick rate, ordering, a win rate under stated settings) may be committed (`value-visibility: committed`).

## 6. How implementers cite IDs

- Every behavioural constant, formula, ordering rule and quirk switch in simulation code carries a citation on the line or the item it governs:

  ```rust
  // spec: S-0042
  // measure: M-0007
  ```

- Defaults in the rules schema carry the same tags in their doc comments.
- Tests that check a behaviour cite the same IDs.
- CI lints for uncited constants in the simulation crates and for IDs that do not exist in `spec/` or `measurements/`.
- If no spec covers what you need, do not guess. File a spec request in `spec/requests/` (see [spec/requests/README.md](spec/requests/README.md)).

## 7. Oracle wall

The project lead's earlier modding repository contains a benchmark and unit-arena harness that drives the original game. It may be used as a measurement instrument under these rules:

- It runs the **stock** game with stock data, with every gameplay mod and AI change turned off.
- Only **observables** cross into this repository, as M-records: win rates, times, tick counts, credits, unit counts, cell positions, health readings.
- No code, scripts, addresses, configuration or notes from that repository enter this one.
- Implementer sessions never read that repository or its output logs; they read only the M-records.

## 8. Incidents

An incident is any forbidden material reaching this repository: a binary or library identifier, an address, decompiler output, code-shaped spec text, a stock rules value, EA asset bytes, or an implementer session that read a forbidden source.

1. **Stop.** No further work builds on the affected code or spec.
2. **Revert** the commits that brought the material in. If it was pushed, rewrite the public history to remove it where practical.
3. **Re-spec** if a spec was the carrier: the spec goes back through the full scrub pipeline.
4. **Re-implement** the affected code in a **fresh** implementer session that has not seen the reverted code or the tainted material.
5. **Log** it in `spec/incidents/I-xxxx.md`: date, what was found, how it got in, what was reverted, the replacement commits, and any rule change made to prevent a repeat.

## 9. Tooling that enforces the wall

- `.claude/settings.json` denies implementer sessions read and edit access to the spec workspace, the scrub staging folders and the lead's modding repository, common disassembler commands, and web access to the extension projects' documentation hosts and ModEnc. Machine-specific absolute paths go in the untracked `.claude/settings.local.json`.
- File access denies are the main guard. Shell-command denies match command text and are best-effort only. Domain denies cannot block a single GitHub repository without blocking all of GitHub, so implementers must also follow the reading rules in [CLAUDE.md](CLAUDE.md).
