//! Making a shot (S-0301): the delay between shots and how it is chosen,
//! bursts, ammunition and reloading, and the house factors set when the
//! houses are set up.

use glasnost_num::F64;

use crate::combat::state::HouseCombat;
use crate::rules::{Kind, SimRules};

/// A new object's ammunition: its type's initial ammunition (S-0301 claim
/// 10). How it relates to the maximum A belongs to the production specs;
/// request Q-0060 asks it, and meanwhile the initial field is used as it
/// stands (−1: unlimited).
pub fn initial_ammo(r: &SimRules, kind: Kind, type_index: u32) -> i32 {
    match kind {
        Kind::Building => r
            .buildings
            .get(type_index as usize)
            .map_or(-1, |b| b.combat.initial_ammo),
        Kind::Terrain => -1,
        // spec: S-0301 claim 10 (a firer starts with its type's initial ammunition)
        _ => r
            .unit(kind, type_index)
            .map_or(-1, |u| u.combat.initial_ammo),
    }
}

/// A house's combat factors, fixed when the houses are set up: every house
/// uses the normal difficulty row (S-0301 claim 3, S-0302 claim 3; glossary
/// *difficulty skill block*).
pub fn house_combat(r: &SimRules, country: u32, houses: usize) -> HouseCombat {
    let mode = r.mode;
    let g = &r.combat.general;
    let one = F64::from_i64(1, mode);
    let (rof, fp) = r
        .combat
        .countries
        .get(country as usize)
        .map_or((one, one), |c| (c.rof, c.firepower));
    HouseCombat {
        // spec: S-0301 F2 (h = RZ₅₃(ℓ × κ) in a skirmish)
        rof_factor: g.row_rof.mul(rof, mode),
        // spec: S-0302 claim 3 and F1 (F_h = RZ₅₃(φᵣ × φ_c))
        firepower: g.row_firepower.mul(fp, mode),
        anger: vec![0; houses],
        enemy: None,
        score: 0,
        kills: 0,
        losses: 0,
        self_heal: [0, 0],
    }
}
