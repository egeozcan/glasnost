//! The own AI and drainers in the air (M7b AI round 3; PLAN §4.7).
//!
//! A flying drainer over a building (S-0608) can be shot only by a weapon
//! whose projectile is anti-air (S-0306 claim 23) and whose warhead's
//! versus value against the drainer's armour is not zero (S-0304 claim 1).
//! In the stock lane a house drained for 9 000 ticks had nothing that could
//! hit the drainer. A person who sees enemy drainers in the air and has
//! fewer units and defences able to hit them than there are drainers adds
//! one: the cheapest unit or defence type he can start now whose weapons
//! can, one at a time. The economy defence (`Brain::defend`) sends only
//! units the firing check lets attack the drainer. A ruleset with no such
//! type leaves the brain as it was.
//!
//! Every order is one a person gives with the mouse. It reads only the
//! house's own objects, the enemies it has seen and the rules a person may
//! read. It draws no random value.

use glasnost_sim::combat::data::WeaponId;
use glasnost_sim::combat::{veterancy, weapon};
use glasnost_sim::econ::{TypeRef, production, tech};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, World};

use crate::drain::is_drainer;
use crate::scripted::{Script, all_types};

/// The weapon slots of type `t`, ordinary and elite.
fn slots(w: &World, t: TypeRef) -> Vec<WeaponId> {
    let r = w.rules();
    let tc = match t.kind {
        Kind::Building => r.buildings.get(t.index as usize).map(|b| &b.combat),
        Kind::Terrain => None,
        _ => r.unit(t.kind, t.index).map(|u| &u.combat),
    };
    tc.map(|tc| {
        tc.slots
            .iter()
            .chain(&tc.elite_slots)
            .flatten()
            .copied()
            .collect()
    })
    .unwrap_or_default()
}

/// Whether type `t` has a weapon that can hit an object of armour class
/// `armour` in the air: an anti-air projectile (S-0306 claim 23), a
/// warhead whose versus value for the class is not zero (S-0304 claim 1)
/// and damage above zero (a healing weapon heals).
pub fn hits_air(w: &World, t: TypeRef, armour: usize) -> bool {
    let r = w.rules();
    slots(w, t).into_iter().any(|wp| {
        // spec: S-0306 claim 23 (an airborne target needs an anti-air projectile)
        let aa = r.combat.projectile_of(wp).is_some_and(|p| p.aa);
        // spec: S-0304 claim 1 (a versus value of exactly zero makes the target illegal)
        let versus = r
            .combat
            .warhead_of(wp)
            .is_some_and(|h| !h.versus(armour).eq_num(Default::default()));
        aa && versus && r.combat.weapon(wp).damage > 0
    })
}

/// The seen enemy drainers in the air (S-0307 Terms: airborne by height),
/// with their armour classes.
pub fn drainers_in_air(script: &Script, w: &World) -> Vec<(ObjectId, usize)> {
    script
        .seen_enemies(w)
        .into_iter()
        .map(|(id, _)| id)
        .filter(|&id| is_drainer(w, id) && weapon::airborne_by_height(w, id))
        .filter_map(|id| veterancy::type_combat(w, id).map(|tc| (id, tc.armour)))
        .collect()
}

/// The house's objects on the map, units and buildings, and the items on
/// its lines, whose type can hit one of `armours` in the air.
fn able(script: &Script, w: &World, armours: &[usize]) -> usize {
    let hits = |t: TypeRef| armours.iter().any(|&a| hits_air(w, t, a));
    let on_map = script
        .own(w)
        .into_iter()
        .filter_map(|id| w.obj(id))
        .filter(|o| o.health > 0 && hits(tech::type_of(o)))
        .count();
    let lined: usize = w
        .econ
        .lines
        .lines
        .iter()
        .filter(|l| l.house == script.house)
        .map(|l| {
            l.item.iter().filter(|i| hits(i.t)).count()
                + l.queue.iter().filter(|&&q| hits(q)).count()
        })
        .sum();
    on_map + lined
}

/// The type to build against drainers in the air: while the house sees
/// more of them than it has units, defences and line items able to hit
/// them, the cheapest unit or defence type it can start now that hits
/// one of them (on equal price the first in type order), when its line is
/// free.
pub(crate) fn build_type(script: &Script, w: &World) -> Option<TypeRef> {
    let threats = drainers_in_air(script, w);
    if threats.is_empty() {
        return None;
    }
    let armours: Vec<usize> = threats.iter().map(|&(_, a)| a).collect();
    if able(script, w, &armours) >= threats.len() {
        return None;
    }
    all_types(w)
        .into_iter()
        .filter(|&t| armours.iter().any(|&a| hits_air(w, t, a)))
        .filter(|&t| crate::startable(w, script.house, t))
        .filter(|&t| !script.busy(w, t))
        .min_by_key(|&t| (production::cost_field(w, t), t.kind, t.index))
}
