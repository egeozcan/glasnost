//! The houses' part of the tick (S-0203 row 7): each house's own update,
//! one house after another in house order (S-0203 claim 8). The order of the
//! topics inside one house's update follows the specs that own them.

use crate::world::{HouseId, World};

/// Row 7 of S-0203: every house's update, in house order.
pub fn part(w: &mut World) {
    // spec: S-0414 claim 16 (the final tick's results are committed at the start of the houses' part)
    crate::econ::victory::commit(w);
    // spec: S-0203 claim 8 (the houses one after another in house order)
    for hi in 0..w.houses.len() {
        house_update(w, HouseId(hi as u8));
    }
}

/// One house's update.
fn house_update(w: &mut World, h: HouseId) {
    // Power, then radar (S-0405 claim 7, S-0410 claim 2): the radar recheck
    // after an event sees the power worked out in the same update (S-0410
    // scenario R1, the power changes at (d) and (e)).
    let low_before = crate::econ::power::low_power(w, h);
    if crate::econ::power::house_update(w, h) {
        // spec: S-0410 claim 2 (a change of the power balance asks for a recheck)
        crate::econ::radar::mark_due(w, h);
    }
    let low_changed = crate::econ::power::low_power(w, h) != low_before;
    // spec: S-0512 claim 2 (b) and claim 5 (a low-power change worked out in this house update: the availability check; then completion, before this update's checks of claim 2 (a))
    crate::superweapons::charge::house_update(w, h, low_changed);
    crate::econ::radar::house_update(w, h);
    // spec: S-0414 claim 16 (a pending surrender's destruction and the defeat test after power and the ordinary radar recheck, which do not see that destruction, and before the tech-tree recheck, which does)
    crate::econ::victory::house_update(w, h);
    // spec: S-0414 claim 18 (the owner-house raise of S-0011 claim 35 sees this update's surrender destruction and defeat test, after power and radar, before the tech-tree recheck; defeated houses keep raising)
    crate::triggers::house_raise(w, h);
    // spec: S-0217 claim 2 (a test-only computer-controlled player house's check comes after the raise of its tags in the same house update, a rule of our engine; S-0203 row 7, in the row's house order)
    crate::computer::house_update(w, h);
    // spec: S-0407 claim 12 (a recheck in the first house update after an event; S-0404 claim 9 and S-0411 claim 2: the removal orders in that house update)
    let recheck = w
        .houses
        .get(usize::from(h.0))
        .is_some_and(|x| x.econ.tech.recheck_due);
    super::tech::house_update(w, h);
    // spec: S-0512 claim 2 (a) (the superweapon checks at every recheck, in the house update where it happens)
    crate::superweapons::charge::after_recheck(w, h, recheck);
    // spec: S-0513 claim 32 (the force shield's fading sound, counted in house updates)
    crate::superweapons::protect::fade_update(w, h);
    // spec: S-0411 claim 9 (the automatic release order)
    super::sidebar::release_orders(w, h);
    // spec: S-0314 claim 15 (the automatic-repair hold is released by the start tick's house update: human and AI houses, and the neutral and special houses' delay 0)
    if let Some(house) = w.houses.get_mut(usize::from(h.0)) {
        house.combat.repair_hold = false;
    }
}
