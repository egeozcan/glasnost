//! Fear in combat (S-0315; S-0107 claim 16): how damage raises an infantry
//! unit's fear, and the panic of a fraidycat that fires its last round.

use glasnost_num::F64;

use crate::combat::data::Ability;
use crate::combat::veterancy::has_ability;
use crate::rules::Behaviour;
use crate::world::{ObjectId, World};

/// The fear level after surviving damage (S-0107 claim 16, F5).
// spec: S-0107 F5 (i = ⌊i₀ / 2⌋ above the yellow ratio)
pub fn damaged(w: &mut World, id: ObjectId, known_attacker: bool) {
    let rules = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(u) = rules.unit(o.kind, o.type_index) else {
        return;
    };
    let Some(s) = o.infantry.as_ref() else { return };
    let fear = s.fear;
    // spec: S-0107 claim 16 (fearless: the type is fearless or the unit has the fearless ability)
    let fearless = u.fearless || has_ability(w, id, Ability::Fearless);
    let fraidy = u.fraidycat;
    let mode = rules.mode;
    let strength = u.strength;
    // spec: S-0107 F5 (i₀ = 50 at or below red, 25 above; halved above yellow; strength 0: above both)
    let (above_red, above_yellow) = if strength == 0 {
        (true, true)
    } else {
        let h = F64::from_i64(i64::from(o.health), mode)
            .div(F64::from_i64(i64::from(strength), mode), mode);
        (
            h.gt(rules.general.condition_red),
            h.gt(rules.general.condition_yellow),
        )
    };
    let i0 = if above_red { 25 } else { 50 }; // spec: S-0107 Constants (fear increments 50, 25)
    let i = if above_yellow { i0 / 2 } else { i0 };
    let up = (fear + i).min(300); // spec: S-0107 F5 (new fear = min(fear + i, 300))
    // spec: S-0107 claim 16 (the table of new fear levels)
    let new = if known_attacker && fear < 100 {
        if fraidy && (!fearless || rules.quirks.fraidycat_fear_ignores_fearless) {
            300 // spec: S-0107 claim 16 (a fraidycat hit by a known attacker below 100: 300)
        } else if fearless {
            fear
        } else {
            100 // spec: S-0107 claim 16 (below 100, not fraidycat, not fearless: 100)
        }
    } else if fearless {
        fear
    } else {
        up
    };
    if let Some(st) = w.obj_mut(id).and_then(|o| o.infantry.as_mut()) {
        st.fear = new;
    }
}

/// A fraidycat that fires a shot leaving it with no ammunition panics
/// (S-0315 claim 2).
pub fn last_round(w: &mut World, id: ObjectId) {
    let rules = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(u) = rules.unit(o.kind, o.type_index) else {
        return;
    };
    if !u.fraidycat || o.combat.ammo != 0 {
        return;
    }
    let fearless = u.fearless || has_ability(w, id, Ability::Fearless);
    // spec: S-0315 claim 2 (quirks.fraidycat_ammo_panic_ignores_fearless; off: nothing for a fearless unit)
    if fearless && !rules.quirks.fraidycat_ammo_panic_ignores_fearless {
        return;
    }
    let cur = o.beh.seen();
    if let Some(ob) = w.obj_mut(id) {
        if let Some(st) = ob.infantry.as_mut() {
            st.fear = 300; // spec: S-0315 claim 2 (fear set to 300)
        }
        // spec: S-0315 claim 2 (from attack or hunt, guard is queued)
        if matches!(cur, Some(Behaviour::Attack | Behaviour::Hunt)) {
            ob.beh.queue(Some(Behaviour::Guard));
        }
    }
}
