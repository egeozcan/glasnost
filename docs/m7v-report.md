# m7v final report (visual-fix pass, 2026-10-20)

The session's closing report, copied from its log by the lead's wrap-up script.

Waiting on the map lane (bkn8exqhg) and the two agents.

That was an interim notice: the slots agent is still running its workspace tests. One thing I noticed: the sandbox's write list no longer includes that agent's worktree (`.claude/worktrees/agent-aaf9437e44e76a16f`), only the protection agent's. If the slots agent's next write fails because of that, I'll report it rather than work around it. Still waiting on the map lane and both agents.

The all-owners map lane has finished; its pictures are in `local-lane/visual-fix-3/map-all-owners`. The skirmish-view map run is now going in the background.

The slots agent had ended its turn while its tests were still running, so I told it to carry on. It is to finish its tests, check and gate, rebase onto main and then report.

The protection agent is still waiting on its gate. I'll merge each agent as it reports.

Both map runs passed. The skirmish-view pictures (16 files) are in `local-lane/visual-fix-3/map-skirmish-view`. I'm waiting for the slots and protection agents to report back before merging.

The iron curtain and force shield work is merged into main. Both effects are now drawn, and the agent's two pictures show them. The one part still missing is the protection look on buildings' slot animations.

- **Protection merge:** the agent's final checks (workspace tests, `cargo xtask check` and `cargo xtask gate`) passed in its worktree before the merge. I haven't re-run them on main yet; I'll run all three at the end, as planned.
  - The save format moved from 65 to 66, because the simulation now keeps every step of the brightness envelope.
  - It filed two requests: Q-1527 (a colour selector outside the defined range) and Q-1528 (protection on a building's bib and door, and on units while they cloak).
  - The pictures are `local-lane/visual-fix-3/agent-protection/sw-iron-curtain.png` and `sw-force-shield.png`.
- **Merge conflict:** `docs/development.md` conflicted. I kept both sides of the interim table and added an iron curtain / force shield row to the item results.
- **Worktree:** I deleted the branch, but removing the worktree's folder under `.git/worktrees` failed with "Device or resource busy", as before. I'll run `git worktree prune` at the end.
- **Spec changes:** three new specs landed on main while I waited: S-0306 Projectiles, S-0107 Infantry sprite sequences and S-0610 Psychic weapons. I'll check the code against them in the end-of-task spec pass. S-0306 may answer Q-1570, the projectile drawing request.
- **Maps:** both map runs passed. The all-owners pictures are in `local-lane/visual-fix-3/map-all-owners/`, and the skirmish-view pictures (16 files) are in `local-lane/visual-fix-3/map-skirmish-view/`.

Two agents are running now. The shot-animation agent started in the freed worktree; it covers the gattling item. The slots agent was resumed to finish its tests, checks and rebase. The beam and layer agents start as these two finish, at most two at a time.

The slots work is merged as well. In the lane pictures, the absorber's levels and the bunker's walls now change with the building's state.

The depot's repair arm still does not show. No spec says what starts it, so the slots agent filed request Q-1550 and nothing starts it for now. I added rows for the three slot-animation items to the results table in `docs/development.md` and committed them.

Two agents are running now:
- **Shot animations:** this covers the gattling item.
- **Beams:** prism, chrono erasure and magnetic lift.

A new spec revision, S-0513 r20, answers both of the protection agent's requests, Q-1527 and Q-1528. I've written a follow-up brief for it. That brief also covers the protection look on building slot animations, which neither earlier agent took on.

The layers agent and this follow-up start as the two running agents finish, two at a time.

The slots worktree's folder under `.git/worktrees` also hit "Device or resource busy" on removal, and git could not lock `.git/config`. I'll run `git worktree prune` at the end.

The shot-animation work is merged into main and answers Q-1523.

- **What it does:** each ordinary shot now plays one animation from the weapon's list, chosen by the shot's facing. On a vehicle the animation follows the firer.
- **No placeholder:** the red muzzle placeholder is no longer drawn in the game client.
- **New request:** the agent filed Q-1580 for how the target line is drawn. For now nothing is drawn for it.
- **Checks:** `cargo test --workspace` passes in its worktree after the rebase. `cargo xtask check` and `cargo xtask gate` passed before the rebase but were not re-run after it, so the final run on main has to cover them. No golden or gate hash changed.
- **Open question from the pictures:** in the gattling shots, the flash sits low on the hull or body rather than at the barrel tips. Two possible causes:
  - The animation starts where the projectile starts. The S-0313 fire point used for that may be off.
  - Our launch point may be missing an offset.

  I'll look at this when I take the final pictures.

I added a gattling row to the item results table and committed it. Removing the worktree again failed with "Device or resource busy"; the branch is deleted.

Two agents are running:
- **Beams:** prism, chrono erasure and magnetic lift.
- **Layers:** the jumpjet drawn under its factory, particle drawing by S-0616 r11 (gas, chaos and berserk smoke), and the V3 impact animation that is drawn but not seen.

The protection follow-up for S-0513 r20 starts when the first of these two finishes.

The layers agent found that the jumpjet-factory picture is a depth effect, not a layer problem, and that the spec says it should look this way.

- **The picture:** the jumpjet is already drawn after the factory. Its depth, though, is anchored to the ground point beneath it, which lies inside the factory's foundation. So the factory parts in front of that point correctly hide the tail (S-0102 claims 10 and 11). The layer work will not change this picture.
- **The layer switch:** under the spec's default switch, layers change only at the moments S-0102 claim 4 names (`quirks.layer_change_on_events_only`). A jumpjet takes off at almost zero height, so it never leaves the ground layer. I checked that S-0102 defines this switch, and approved the agent's plan:
  - implement the switch as a presentation setting;
  - file Q-1560 asking whether a jumpjet's climb to cruise height counts as one of those moments;
  - record the interim rule in the docs.

The agent is carrying on with the particle drawing and the V3 impact depth. The beams agent is still running.

The beams agent is still running: its workspace tests are going in the background, and it will report when they finish.

The sandbox's write list has dropped the beams agent's worktree, as it did for the slots agent earlier; the slots agent got its write access back later. If the beams agent's next write fails because of this, I'll tell you rather than work around it.

The layers agent is still working.

Both agents are now waiting on their own workspace test runs and will report when those finish.

The sandbox's write list has also changed, and this affects me as well. It no longer includes either agent's worktree or `~/glasnost/.git`.

While that lasts, I can't commit or merge anything on main. The agents may also be unable to commit in their worktrees.

I'm not working around it. If a commit or merge fails when the agents report, I'll tell you which path was refused. Restoring that write access is your call.

The beams agent is re-running its workspace tests and will report when they finish. The layers agent is also still waiting on its own test run. I'll merge each one once it sends its final report.

The beams agent is still waiting on its re-run of the workspace tests, and the layers agent is waiting on its own test run. Nothing to merge yet.

The layers agent is still waiting on its test run. Write access to `.git` and to the layers worktree is back; the beams worktree is still missing from the write list. Both agents are still running.

Both agents are still waiting on their test runs, so there is nothing to merge yet.

The sandbox's write access has dropped again. `.git` and both agent worktrees are off the write list, so nobody can commit or merge right now. I'll report it if a commit or merge fails.

Both agents (beams, and layers with particles and impact depth) are still running their workspace tests. There is nothing to merge yet.

The layers agent is now waiting on its `cargo xtask gate` run, and the beams agent is still running its workspace tests. I'll merge each one once it sends its final report.

Both agents are now on their final `cargo xtask gate` runs. I'll merge each one when its report arrives.

The layers work is merged into main, and the protection follow-up agent has started in its place.

**Layers merge**
- **Checks:** the layers agent reports that the workspace tests, `cargo xtask check` and `cargo xtask gate` all pass.
  - Only the duel-m7a golden changed, because each particle now keeps its own translucency and colour state.
  - The save format went from 66 to 67.
- **What it changed:**
  - Mobile objects are now drawn layer by layer (S-0102 claims 1 to 7).
  - Particles are drawn by S-0616 revision 12. This replaces the green discs under gas and chaos smoke.
  - The agent says S-0616 r12 also answers Q-1520 and Q-1540; the lead needs to close both.
- **Two items turned out to be right as drawn, by the depth rules:**
  - **The jumpjet tail inside the factory:** the jumpjet's depth is anchored to its ground point, which lies inside the factory's foundation.
  - **The V3 explosion that is never seen:** it is created below the height of the construction yard it hit, and the yard's shadow images sit in front of it.
- **Requests filed:**
  - Q-1560: when a jumpjet changes layer.
  - Q-1561: whether an explosion below the surface should be hidden like this.
  - Q-1562: how badly formed colour lists are read.
- **Not done yet:**
  - Searchlights (S-0616 claim 31) are not drawn, because the spec leaves their look open.
  - The railgun laser is not drawn, because the beam drawing it needs is not merged yet.
  - No lane scene shows chaos smoke.

I added rows for these items to the item results table and committed them.

**Running now**
- **Beams agent:** it is on its final `cargo xtask gate` run. The sandbox's write list currently leaves out its worktree, so its commit or rebase may fail; if it does, I'll report it rather than work around it.
- **Protection follow-up agent:** this is S-0513 r20, which answers Q-1527 and Q-1528, plus the protection look on building slot animations.

Removing the layers worktree's folder failed with "Device or resource busy" again. The branch is deleted; I'll run `git worktree prune` at the end.

The beams agent is still on its final `cargo xtask gate` run. Its worktree is writable again, so it can commit and rebase. The protection follow-up agent is also still running.

The beams agent is re-running its `cargo xtask gate` check, and the protection follow-up agent is also still running. I'll merge each one once it sends its final report.

The beams agent finished, but merging its work into main conflicted. It had rebased before the layers merge landed, so I aborted the merge and sent it back to rebase onto current main.

**What the beams agent did:**
- Prism beams, chrono and radiation beam flashes, the magnetic lift wave and the half see-through body of a unit being erased are now drawn from S-0518, S-0514 and S-0607.
- On its own branch, before main moved, all three checks passed: `cargo test --workspace`, `cargo xtask check` and `cargo xtask gate`.
- It filed no requests. The cases the specs leave open have interim rules in the docs.

**Why the merge failed:** the conflicts are in four places in `crates/app/src/mapview/live.rs` and in the docs table.

**What I asked for:**
- Resolve the conflicts, keeping both the beams and the per-layer drawing with particles.
- Draw the railgun laser with the new beam code if S-0518 gives its settings; otherwise leave it undrawn and say why in the docs.
- Re-run the three checks.

**Things it reported:**
- `prism-beams.png` is taken at tick 11, before the first shot, so it shows no beam. A trace frame at about tick 40 does show the beams; the final picture needs a later moment.
- No lane scene fires a radiation-beam weapon. Its picture came from a temporary edit to the chrono scene, which the agent reverted.
- Something outside the agent's commands deleted its worktree's `target/` folder during a gate run, and that run failed. A clean rerun passed.

The protection follow-up agent is still running. Once both finish, I'll merge them, take the final screenshots and run the full checks on main.

The beams agent has rebased onto current main, resolved the conflicts and committed the railgun laser drawing. It is now running `cargo test --workspace`, then `cargo xtask check` and `cargo xtask gate`.

The protection follow-up agent is still working. I'll merge both once they report.

The beams agent has passed `cargo xtask check` and is now running `cargo xtask gate`. The protection follow-up agent is still working. Both report when they finish.

The protection follow-up for S-0513 r20 is merged into main.

- **Requests:** Q-1527 and Q-1528 are now handled in the code. The agent filed Q-1529, whether shroud also removes the force-shield colour from a building's slot animations.
- **Checks:** it reports that `cargo test --workspace`, `cargo xtask check` and `cargo xtask gate` passed on its branch.

The lead has since answered Q-1560 and Q-1561 in new spec revisions:

- **S-0102 r24** spells out exactly when a jumpjet, an aircraft or a rocket changes draw layer.
- **S-0506 r13** says a missile's explosion sits exactly at the nose point, even below ground, with depth adjustment −15.

I started an intake agent to bring the code up to both. It will retake the jumpjet-factory and V3 impact pictures, to see whether the explosion now shows.

The beams agent is on its last `cargo xtask gate` run before I can merge it.

The lead's handoff note (`docs/lead-handoff.md`) asks for this pass's closing report in `docs/m7v-report.md`, so I'll write it there as well as in my final message.

The protection worktree's folder again could not be deleted ("Device or resource busy"); its branch is gone. I'll run `git worktree prune` at the end.

The beams agent is re-running the prism scene, now with the picture taken later so the beams are visible, and will read it when it finishes.

The intake agent for S-0102 r24 and S-0506 r13 is still running.

The beams agent's worktree has dropped out of the sandbox's write list again. If its next commit or rebase fails because of that, I'll tell you rather than work around it.

Both agents are still running. The intake agent (S-0102 r24 and S-0506 r13) is waiting for one of its own background runs to finish. The beams agent is re-running the prism scene with a later picture so the beams show.

The beams work is merged into main, so every agent except the intake one is now done and merged.

**Beams merge**
- **What is drawn now:**
  - prism beams;
  - the chrono and radiation beam flashes;
  - the half see-through body of a unit being erased;
  - the magnetic lift wave;
  - the railgun laser.
- **Checks on its final commit:** `cargo test --workspace` passed all 261 test binaries, and `cargo xtask check` and `cargo xtask gate` passed with every golden matching.
- **Pictures:**
  - The prism scene now takes its picture at tick 39, after the support beams fire.
  - The chrono and lift pictures show the flash and the wave.
  - The radiation-beam picture came only from a temporary scene edit, which the agent reverted. No permanent lane scene fires that weapon.
- **New request:** Q-1590, whether a railgun with no held particle type still draws its laser. Nothing is drawn in that case for now.
- **Not done yet:**
  - Voxel parts are made see-through one by one rather than as one image.
  - There is no electric bolt and no fog on the wave.
- **Formatting:** `cargo fmt --check` still flags a few test files on main that no agent touched.

I added rows for the prism, chrono-erasing and lift items to the item results table and committed them.

**Running now**
- **Intake agent** for S-0102 r24 and S-0506 r13: when a jumpjet changes draw layer, and the V3 explosion's placement at depth −15.
- **Final screenshots:** the m6a lane is shooting into `local-lane/visual-fix-3/final/m6a`. This run also compiles and tries the radar-type change I haven't committed yet.

The beams worktree's folder could not be deleted either ("Device or resource busy"). Its branch is gone, and I'll run `git worktree prune` at the end.

The radar lane change is committed. It now shoots every building type that carries the radar flag in the rules (S-0407 claim 6), as `radar-side<s>-<i>.png` under `local-lane/visual-fix-3/final/m6a/`; `<s>` is the type's base-planning side from the rules, and `<i>` its index. That avoids stock type names in the source. The single side-2 picture (`radar-side2-1.png`) will become `yuri-radar.png`.

The final m6a shots are taken. The m6b, m7a and fix lanes are now running in the background into `local-lane/visual-fix-3/final/`.

The m6c and anim lanes wait for the intake agent, because its S-0102 r24 and S-0506 r13 work changes the jumpjet and V3 pictures.

The intake agent (S-0102 r24 and S-0506 r13) is still waiting on its own background run. The m6b, m7a and fix lanes are still taking the final screenshots.

The sandbox's write list again leaves out `.git` and the intake agent's worktree. Until that changes, neither of us can commit or merge; if a commit fails on that, I'll say so here.

The intake agent's gate run passed, and it is now waiting on its test run after rebasing onto main. The m6b, m7a and fix lanes are still taking final screenshots. I'll merge the intake work once it reports, then run the m6c and anim lanes.

The intake agent's post-rebase test run is done, and it is now waiting on its last `cargo xtask gate` run. The final m6b, m7a and fix screenshots are still being taken in the background.

The final m6b, m7a and fix screenshots are taken, and those I've checked show what their names say:

- **mind-overload:** the link lines and sparks over the controller, with no green circles.
- **lift-seized:** the blue magnetic wave between the lifter and the seized vehicle.
- **sw-force-shield:** the shielded tower is tinted blue, and the units around it are untouched.
- **bunker:** the walls are up while it is occupied and come down after the units leave.

The gattling shot is not right yet. The flash is drawn low on the hull, below and to the right of the barrel tips. I started an agent to trace one shot through the fire-point and drawing steps against S-0313 and S-0301 claim 22. It will fix the step that deviates from the spec, or file a request if the spec says nothing.

When the intake agent's work is merged, I'll take the m6c, anim and map final shots.

There are five review items I haven't recorded yet:
- mind-second;
- gattling-building;
- the aircraft-attacking attitude shot;
- cloaked-as-owner;
- the attitude ship-sinking shot.

I'll check them against the final shots and add them to the item results table.

The m6c and anim lanes, the muzzle agent, the depot agent and the spec-triage agent are all still running. Nothing is finished yet. I'll pick the work back up when their notifications arrive. After that I'll do the merges, the per-item pictures, the three checks and the `docs/m7v-report.md` report.

The chrono-erasing picture shows the target turned translucent with sparkles. There is no beam on this tick; the beam is only drawn on the firing tick. The prism picture shows the support beams joining the towers. The muzzle, depot and spec-triage agents are still running, and I'll continue when they report.

