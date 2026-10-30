---
id: Q-1988
title: How does the player make, add to and select a control group?
status: open
related: [S-0111, S-0606, S-0415, S-0214, S-0213, S-0701]
answered-by: [S-0213]
---

## Need

Several specs assume control groups exist. S-0111 claim 17: "A unit or building in one of the
local player's ten control groups shows the digit (γ + 1) mod 10 for group γ (0 to 9)", and its
scenario OV12 puts units "in groups 1, 5 and 10 of the keyboard". S-0606 claim 7: the bunkered
vehicle "leaves any control group it belonged to". S-0415 claim 6: the deployed structure "takes
over the deploying vehicle's ... control group". No spec says how the player makes a group, adds
to it or selects it. S-0213's key commands (table T2) do not list them. Play testing (M8 A4
round 5, note 31) asked for control groups, so the client needs input for them.

## Questions

1. Which inputs make a group of the current selection, add the selection to a group, and select
   a group? Are they fixed keys (for example a modifier plus a digit) or player bindings?
2. Can an object be in more than one group? When an object is put in a group, does it leave the
   group it was in?
3. Selecting a group: does it replace the selection, in which order (claim 12's front/end rule,
   group order, or creation order), and which selection response plays (S-0701 claim 17)? Is
   there an input that adds a group to the selection instead?
4. Does selecting the same group twice in quick succession move the view to it? If so, within
   what interval, and to which point (the first object, the group's middle, or something else)?
5. Which objects can be in a group: own units and buildings only? Does an object leave its group
   when it changes owner (capture, mind control, S-0607), when it is killed, when it enters a
   transport or building, or when it is deactivated? Does it come back when it returns?
6. Does making a group of an empty selection clear the group?
7. Do the group keys act while the game is paused, and while a superweapon or placement is armed?

## Possible observation

Select two tanks and use each candidate key combination with digit 1; note which one makes the
group (S-0111 OV12 shows the digit). Select one of them, put it in group 2, and see whether the
digit over it changes from 1 to 2 and whether group 1 still selects it. Select group 1 twice
quickly with the view elsewhere and note whether the view moves, and where to. Let a group member
be mind-controlled and select the group again.

## Current workaround

Our own design (`Play::group_assign` and `Play::group_recall` in `crates/app/src/game/play.rs`,
`Session::group_key` in `crates/app/src/game/session.rs`, the digit keys in
`crates/app/src/game/client.rs`, marked with this request):

- Ctrl+digit makes the group of the selection's own objects; Shift+digit adds them to it. Each
  object is in one group at most, so it leaves any other group.
- The digit alone replaces the selection with the group's own objects that can still be selected,
  by claim 12's rule; the first one speaks. An empty group leaves the selection alone. A second
  press of the same digit within 500 ms (our own interval, the same as S-0214's double press)
  also centres the view on the group's first object.
- A member that is gone or no longer the player's leaves the group when the group is next
  selected. A bunkered vehicle leaves its group (S-0606 claim 7).
- A digit bound to a command key keeps its command; the group keys give way.
- Not yet done: the group digit over the units (S-0111 claim 17), which waits for the pip scale,
  and the transfer to the deployed structure (S-0415 claim 6).
