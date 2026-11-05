//! Damage resolution (S-0302): what one hit does to one object, the shot
//! damage of a firer (F1), the expected damage (F8) and the reactions of a
//! hit (thresholds, the damage effect and building fires, credit, fear and
//! anger).

use glasnost_map::CellXY;
use glasnost_num::{F64, Rounding};

use crate::combat::data::{Ability, WarheadId, WeaponId};
use crate::combat::veterancy::{has_ability, type_combat};
use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, World};

/// One hit (S-0302 Terms, claim 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    /// Delivered damage d₀.
    pub amount: i32,
    /// Warhead (absent for some scripted sources and for the hits of a
    /// debris piece of a type with no warhead; S-0302 claim 1).
    pub warhead: Option<WarheadId>,
    /// Distance r in leptons from the detonation point (0 without one).
    pub distance: i32,
    /// The attacker.
    pub attacker: Option<ObjectId>,
    /// The attacking house.
    pub house: Option<HouseId>,
    /// The ignore-defences marker.
    pub ignore_defences: bool,
    /// The no-escape marker.
    pub no_escape: bool,
}

/// The damage-state result of a hit (S-0302 claim 12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitResult {
    /// A row of claim 8 stopped the hit, or it healed.
    Unaffected,
    /// Health fell without crossing a threshold, or the target went berserk.
    Unchanged,
    /// Health crossed half the strength.
    NowYellow,
    /// Health crossed strength × red ratio, or a cyborg was crippled.
    NowRed,
    /// Health reached 0.
    NowDead,
    /// A delayed kill started or was hit again (claim 16).
    PostMortem,
}

/// An object's strength (its type's strength field; S-0302 Terms).
pub fn strength(w: &World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id) else { return 0 };
    let r = w.rules();
    match o.kind {
        Kind::Building => r
            .buildings
            .get(o.type_index as usize)
            .map_or(0, |b| b.strength),
        Kind::Terrain => r
            .terrain
            .get(o.type_index as usize)
            .map_or(0, |t| t.strength),
        _ => r.unit(o.kind, o.type_index).map_or(0, |u| u.strength),
    }
}

/// The double of an integer.
fn d(n: i64, mode: Rounding) -> F64 {
    F64::from_i64(n, mode)
}

/// The shot damage D_s of a weapon fired by an object (S-0302 claim 2,
/// F1), fixed at the moment of firing. A garrisoned building's multiplier
/// M_g (S-0508) and a bunkered vehicle's M_b (S-0606) apply; the
/// open-topped transport's M_t is a seam.
pub fn shot_damage(w: &World, firer: ObjectId, weapon: WeaponId) -> i32 {
    let r = w.rules();
    let mode = r.mode;
    let wr = r.combat.weapon(weapon);
    // spec: S-0302 F1 (E₀ = 0 for a sonic-wave or fire-particle weapon)
    if wr.sonic || wr.fire_particles {
        return 0;
    }
    let dmg = wr.damage;
    // spec: S-0302 F1 (E₂ = trunc(RZ₅₃(E₁ × M_g)) for a garrisoned building, whatever the sign of D)
    let garrison = |e1: i32| {
        if crate::structures::garrison::garrisoned(w, firer) {
            d(i64::from(e1), mode)
                .mul(r.combat.general.damage_mult[0], mode)
                .to_i32_common()
        } else {
            e1
        }
    };
    // spec: S-0302 F1 (E₃ = trunc(RZ₅₃(E₂ × M_b)) for a bunkered unit, whatever the sign of D; S-0606 claim 9)
    let bunker = |e2: i32| {
        if crate::structures::bunker::bunkered(w, firer) {
            d(i64::from(e2), mode)
                .mul(r.combat.general.damage_mult[1], mode)
                .to_i32_common()
        } else {
            e2
        }
    };
    // spec: S-0302 F1 (E₀ = D for D ≤ 0: the multipliers leave it unchanged)
    if dmg <= 0 {
        return bunker(garrison(dmg));
    }
    let Some(o) = w.obj(firer) else { return dmg };
    let fh = o
        .owner
        .and_then(|h| w.houses.get(usize::from(h.0)))
        .map_or(d(1, mode), |h| h.combat.firepower);
    let fo = o.combat.firepower_mult;
    // spec: S-0302 F1 (E₀ = trunc(RZ₅₃(RZ₅₃(F_h × F_o) × D)))
    let e0 = fh
        .mul(fo, mode)
        .mul(d(i64::from(dmg), mode), mode)
        .to_i32_common();
    // spec: S-0302 F1 (E₁ = trunc(RZ₅₃(E₀ × V_c)) for a firer with the veteran firepower bonus)
    let e1 = if has_ability(w, firer, Ability::Firepower) {
        d(i64::from(e0), mode)
            .mul(r.combat.general.veteran_combat, mode)
            .to_i32_common()
    } else {
        e0
    };
    bunker(garrison(e1))
}

/// The house armour factor A_h for a target (S-0302 claim 5): the factor of
/// the owner's country for the target's kind (infantry, vehicles,
/// aircraft, other buildings, defence buildings), `kind_of` giving the
/// object whose kind counts.
fn house_armour(w: &World, owner_of: ObjectId, kind_of: ObjectId) -> F64 {
    let r = w.rules();
    let mode = r.mode;
    let one = d(1, mode);
    let Some(owner) = w.obj(owner_of).and_then(|o| o.owner) else {
        return one;
    };
    let Some(country) = w.houses.get(usize::from(owner.0)).map(|h| h.country) else {
        return one;
    };
    let Some(c) = r.combat.countries.get(country as usize) else {
        return one;
    };
    let Some(k) = w.obj(kind_of) else { return one };
    // spec: S-0302 claim 5 (the factor for the kind: infantry, vehicles, aircraft, defence buildings, other buildings)
    let i = match k.kind {
        Kind::Infantry => 0,
        Kind::Vehicle => 1,
        Kind::Aircraft => 2,
        Kind::Building => {
            if r.buildings[k.type_index as usize].building.defense {
                4 // spec: S-0302 F2 (defence buildings: the fifth factor)
            } else {
                3 // spec: S-0302 F2 (other buildings: the fourth factor)
            }
        }
        Kind::Terrain => return one,
    };
    c.armour[i]
}

/// The defended amount d₃ of S-0302 F2 for a hit that does not ignore
/// defences and has d₀ ≥ 0. `prone` applies the prone factor; the armour
/// factor and multiplier are those of `armour_of` (the target, or the firer
/// for the expected damage, F8).
fn defended(
    w: &World,
    target: ObjectId,
    armour_of: ObjectId,
    amount: i32,
    prone_factor: Option<F64>,
) -> i32 {
    let r = w.rules();
    let mode = r.mode;
    // spec: S-0302 F2 (d₁ = max(trunc(RZ₅₃(d₀ × P)), 1) for a prone infantry target with d₀ > 0)
    let d1 = match prone_factor {
        Some(p) if amount > 0 => d(i64::from(amount), mode)
            .mul(p, mode)
            .to_i32_common()
            .max(1),
        _ => amount,
    };
    let ah = house_armour(w, target, armour_of);
    let ao = w
        .obj(armour_of)
        .map_or(d(1, mode), |o| o.combat.armour_mult);
    // spec: S-0302 F2 (d₂ = trunc(RZ₅₃(d₁ / RZ₅₃(A_h × A_o))))
    let d2 = d(i64::from(d1), mode)
        .div(ah.mul(ao, mode), mode)
        .to_i32_common();
    // spec: S-0302 F2 (d₂′ = trunc(RZ₅₃(d₂ / V_a)) for a target with the veteran armour bonus)
    let d2p = if has_ability(w, target, Ability::Stronger) {
        d(i64::from(d2), mode)
            .div(r.combat.general.veteran_armor, mode)
            .to_i32_common()
    } else {
        d2
    };
    // spec: S-0302 F2 (d₃ = max(d₂′, 1))
    d2p.max(1)
}

/// The falloff and versus step of S-0302 F3 case (vi) (S-0303 F3) for a
/// positive defended amount `d3` at distance `r`.
// spec: S-0302 F3 (R = trunc(RZ₅₃(s × 256)))
pub fn falloff_versus(w: &World, warhead: WarheadId, d3: i32, r: i32, v: F64) -> i32 {
    let rules = w.rules();
    let mode = rules.mode;
    let h = rules.combat.warhead(warhead);
    // spec: S-0302 F3 (f = RZ₂₄(d₃), a = RZ₂₄(RZ₅₃(d₃ × p)) from the integer d₃, not the narrowed f; R = trunc(RZ₅₃(s × 256)))
    let d3d = d(i64::from(d3), mode);
    let f = d3d.to_f32(mode).to_f64();
    let a = d3d.mul(h.percent_at_max, mode).to_f32(mode).to_f64();
    let big_r = h.spread.mul(d(256, mode), mode).to_i32_common();
    // spec: S-0302 F3 (g = d₃ when R = 0 or a = f)
    let g = if big_r == 0 || a.eq_num(f) {
        d3
    } else {
        // spec: S-0302 F3 (g = trunc(RZ₅₃(RZ₅₃(RZ₅₃(RZ₅₃(f − a) × (R − r)) / R) + a)))
        f.sub(a, mode)
            .mul(d(i64::from(big_r) - i64::from(r), mode), mode)
            .div(d(i64::from(big_r), mode), mode)
            .add(a, mode)
            .to_i32_common()
    };
    // spec: S-0302 F3 case (vi) (m₀ = min(trunc(RZ₅₃(max(g, 0) × v)), D_max))
    let m = d(i64::from(g.max(0)), mode).mul(v, mode).to_i32_common();
    m.min(rules.combat.general.max_damage)
}

/// F3 of S-0302: the modified amount m₀ for a hit, given the amount after
/// the defences (`d3`, which is d₀ for a hit that ignores defences).
fn modified(w: &World, hit: &Hit, d3: i32, r: i32, class: usize) -> i32 {
    // spec: S-0302 F3 case (i) (a hit that ignores defences: d₀)
    if hit.ignore_defences {
        return d3;
    }
    // spec: S-0302 F3 case (ii) and claim 6 (an inert game or no warhead: 0, such as a debris piece of a type with no warhead, S-0317 claim 7)
    let Some(h) = hit.warhead else { return 0 };
    if w.map_flags.inert {
        return 0;
    }
    if d3 < 0 {
        // spec: S-0302 F3 cases (iii) and (iv) (a negative amount: itself below 8 leptons, else 0)
        return if r < 8 { d3 } else { 0 };
    }
    if d3 == 0 {
        return 0; // spec: S-0302 F3 case (v)
    }
    let v = w.rules().combat.warhead(h).versus(class);
    falloff_versus(w, h, d3, r, v)
}

/// The modified amount m₀ (S-0302 claims 5 and 6, F2 and F3) a hit of a
/// positive `amount` with `warhead` at distance `r` would deal `target`,
/// before claim 8's stopping tests: an estimate that changes nothing, for
/// a player's foresight of an area hit.
pub fn hit_estimate(w: &World, target: ObjectId, amount: i32, warhead: WarheadId, r: i32) -> i32 {
    let Some(o) = w.obj(target) else { return 0 };
    if amount <= 0 {
        return 0;
    }
    // spec: S-0302 F2 (the prone factor for a prone infantry target)
    let prone = o.kind == Kind::Infantry && o.infantry.as_ref().is_some_and(|s| s.prone);
    let prone_factor = prone.then(|| w.rules().combat.warhead(warhead).prone);
    let d3 = defended(w, target, target, amount, prone_factor);
    let class = type_combat(w, target).map_or(0, |tc| tc.armour);
    let hit = Hit {
        amount,
        warhead: Some(warhead),
        distance: r,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    };
    modified(w, &hit, d3, r, class)
}

/// The expected damage of a weapon fired by `firer` at `target` (S-0302
/// claim 18, F8).
pub fn expected_damage(w: &World, firer: ObjectId, weapon: WeaponId, target: ObjectId) -> i32 {
    let r = w.rules();
    let wr = r.combat.weapon(weapon);
    // spec: S-0302 F8 (0 for a sonic-wave or fire-particle weapon; the damage field when it is 0 or less)
    if wr.sonic || wr.fire_particles {
        return 0;
    }
    if wr.damage <= 0 {
        return wr.damage;
    }
    let e1 = shot_damage(w, firer, weapon);
    // spec: S-0302 F8 (A_h and A_o of the firer, quirks.expected_damage_firer_armour; off: the target's)
    let armour_of = if r.quirks.expected_damage_firer_armour {
        firer
    } else {
        target
    };
    let d3 = defended(w, target, armour_of, e1, None);
    let class = type_combat(w, target).map_or(0, |tc| tc.armour);
    let hit = Hit {
        amount: e1,
        warhead: wr.warhead,
        distance: 0,
        attacker: Some(firer),
        house: None,
        ignore_defences: false,
        no_escape: false,
    };
    modified(w, &hit, d3, 0, class)
}

/// Whether a vehicle stands inside the vehicle factory it is in contact
/// with (S-0302 claim 8, stage 1).
fn inside_factory(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Vehicle {
        return false;
    }
    let Some(f) = o.contact.and_then(|c| w.obj(c)) else {
        return false;
    };
    if f.kind != Kind::Building
        || !w.rules().buildings[f.type_index as usize]
            .building
            .weapons_factory
    {
        return false;
    }
    crate::objects::foundation_cells(f.pos.cell(), f.foundation).contains(&o.pos.cell())
}

/// Applies one hit to one object (S-0302 claims 5 to 20) and its reactions
/// (claims 12 to 16; S-0107 claim 16; S-0309 claim 16; S-0311 claims 7 to
/// 10; S-0312 when it kills), then the crush response (S-0215 claim 39).
pub fn apply(w: &mut World, target: ObjectId, hit: Hit) -> HitResult {
    // spec: S-0216 claim 8 (Removal before the groups: no object damages an idle test house's objects; design)
    if w.of_idle_house(target) {
        return HitResult::Unaffected;
    }
    let result = apply_hit(w, target, hit);
    // spec: S-0215 claim 39 (a hit that has been applied and has not killed the vehicle, unaffected and unchanged included, berserk hits too; now dead and post-mortem give none; after every other effect of the same hit)
    if !matches!(result, HitResult::NowDead | HitResult::PostMortem) {
        crate::crush::respond_to_hit(w, target, hit.attacker);
    }
    result
}

/// [`apply`] without the crush response.
fn apply_hit(w: &mut World, target: ObjectId, hit: Hit) -> HitResult {
    let Some(o) = w.obj(target).cloned() else {
        return HitResult::Unaffected;
    };
    if o.kind == Kind::Terrain {
        return terrain_hit(w, target, hit);
    }
    // spec: S-0504 claim 10 (sinking is no immunity: damage delivered to a sinking vessel takes the ordinary rules)
    // spec: S-0315 claims 6 and 7 and S-0614 claim 12 (a hit on a host counts before it changes the host's health, stopped hits included; not inside the vehicle factory)
    if !inside_factory(w, target) {
        crate::parasite::host_hit(w, target, &hit);
    }
    let rules = w.rules_arc();
    let mode = rules.mode;
    let q = rules.quirks;
    let tc = type_combat(w, target).cloned().unwrap_or_default();
    let wh = hit.warhead.map(|h| rules.combat.warhead(h).clone());
    let mut hit = hit;
    // spec: S-0302 claim 17 (an infantry unit in the air hit by a warhead of infantry-death kind 9: d₀ = 0)
    let mutated = o.kind == Kind::Infantry
        && w.height_above_surface(target) > 0
        && wh.as_ref().is_some_and(|h| h.inf_death == 9);
    if mutated {
        hit.amount = 0;
    }
    let plus = !hit.ignore_defences && hit.amount >= 0;
    let ign = hit.ignore_defences;
    let building = o.kind == Kind::Building;
    let b = building.then(|| &rules.buildings[o.type_index as usize].building);
    let same_type_owner = hit
        .attacker
        .and_then(|a| w.obj(a))
        .is_some_and(|a| a.kind == o.kind && a.type_index == o.type_index && a.owner == o.owner);
    // spec: S-0302 claim 8 (stage 1 rows: no side effect at all)
    let stage1 = (building && hit.attacker == Some(target) && !tc.damage_self)
        || (b.is_some_and(|b| b.laser_fence) && !ign)
        || (b.is_some_and(|b| b.bridge_repair_hut) && tc.immune)
        || (inside_factory(w, target) && !ign)
        || (tc.type_immune && same_type_owner && plus)
        || (building && o.health <= 0)
        // spec: S-0302 claim 8 (stage 1: the target is temporarily invulnerable, + hits; S-0513 claim 26)
        || (crate::superweapons::protected(w, target) && plus)
        // spec: S-0302 claim 8 (stage 1: the target is being erased, + and − hits; the leaving stage of S-0513 claim 39, the temporal beam is S-0514's seam) and S-0302 r40 claim 8 (a superweapon warp member whose beam was locked before the departure is not being erased from its departure update and takes ordinary hits, up to a later lock, from which this protection applies again: S-0513 claim 39a, in `being_erased`)
        || (crate::econ::power::being_erased(w, target) && !ign);
    if stage1 {
        return HitResult::Unaffected;
    }
    // The amount after the defences of claim 5 (d₀ for a hit that ignores defences or heals).
    let prone = o.kind == Kind::Infantry && o.infantry.as_ref().is_some_and(|s| s.prone);
    let prone_factor = prone.then(|| wh.as_ref().map_or(d(1, mode), |h| h.prone));
    let d3 = if mutated && !q.airborne_mutation_hit_raised {
        // spec: S-0302 Edge cases (airborne_mutation_hit_raised off: the 0 is not raised to 1)
        0
    } else if plus {
        defended(w, target, target, hit.amount, prone_factor)
    } else {
        hit.amount
    };
    // spec: S-0302 claim 10 (readiness loss for every hit that no stage 1 row stops; quirks.readiness_from_defended_damage)
    if q.readiness_from_defended_damage {
        readiness_loss(w, target, d3);
    }
    let allied_attacker = hit
        .attacker
        .and_then(|a| w.obj(a))
        .is_some_and(|a| w.allied(a.owner, o.owner));
    // spec: S-0302 claim 8 (stage 2 rows: radiation, psychic, poison immunities; allies); *Hits without a warhead* (none of them holds for a hit without one)
    let stage2 = wh.as_ref().is_some_and(|h| {
        (h.radiation && tc.immune_radiation)
            || (h.psychic && tc.immune_psionic_weapons)
            || (h.poison && tc.immune_poison)
            || (!h.affects_allies && allied_attacker)
    });
    // spec: S-0302 claim 8 (stage 2 bunker rows, + and − hits: a tank bunker holding a bunkered unit hit by a bunker-penetrating warhead; a bunkered unit hit by a warhead that does not penetrate bunkers)
    // spec: S-0302 claim 8, *Hits without a warhead* (revision 39: neither bunker row holds for a hit without a warhead, which counts as neither penetrating bunkers nor not penetrating them)
    let penetrates = wh.as_ref().map(|h| h.penetrates_bunker);
    let bunker_row = !ign
        && ((crate::structures::bunker::is_bunker(w, target)
            && crate::structures::bunker::occupied(w, target)
            && penetrates == Some(true))
            || (crate::structures::bunker::bunkered(w, target) && penetrates == Some(false)));
    if stage2 || bunker_row {
        return HitResult::Unaffected;
    }
    // spec: S-0302 claim 10 (stage 3: a berserk warhead changes no health); claim 8, *Hits without a warhead* (a hit without one has no berserk effect)
    if let Some(h) = wh.as_ref().filter(|h| h.psychedelic) {
        let house_allied = hit.house.is_some_and(|hh| w.allied(o.owner, Some(hh)));
        if house_allied || tc.immune_psionics || building {
            return HitResult::Unaffected;
        }
        // spec: S-0302 Formulas (berserk duration: F3's cases at r = 0 with d after the defences, or d₀ ignoring them)
        let class = tc.armour;
        let t = if hit.ignore_defences {
            let mut h2 = hit;
            h2.ignore_defences = false;
            modified(w, &h2, hit.amount, 0, class)
        } else {
            modified(w, &hit, d3, 0, class)
        };
        let _ = h;
        // spec: S-0302 claim 10 and S-0610 claim 4 (entry: T, target dropped, hunt queued; a further berserk hit sets the remaining time to the new T; quirks.berserk_zero_duration)
        // spec: S-0302 r41 claim 10 (the fall-back of S-0215 claim 16 is judged on the behaviour current when the hit lands, so before the entry queues hunt; it needs a known attacker)
        let falls = hit.attacker.is_some() && crate::process::no_threat_applies(w, target);
        if crate::psychic::enter(w, target, t) {
            berserk_reactions(w, target, &hit, falls);
        }
        return HitResult::Unchanged;
    }
    // spec: S-0302 claim 6 (the modified amount)
    let m0 = modified(w, &hit, d3, hit.distance, tc.armour);
    // spec: S-0302 claim 7 (an undemolishable building loses at least 1; quirks.undemolishable_heal_is_damage)
    let undemolishable = b.is_some_and(|b| !b.can_c4);
    let m = if undemolishable {
        if q.undemolishable_heal_is_damage {
            m0.max(1)
        } else if m0 == 0 {
            1
        } else {
            m0
        }
    } else {
        m0
    };
    // spec: S-0302 claim 8 (stage 4 rows: no health on a non-building, immune, an amount of 0)
    let stage4 = (!building && o.health <= 0) || (tc.immune && !ign) || m == 0;
    if !q.readiness_from_defended_damage && !stage4 {
        // spec: S-0302 Edge cases (readiness_from_defended_damage off: the final amount, not when claim 8 stops the hit)
        readiness_loss(w, target, m);
    }
    if stage4 {
        damage_effect_reaction(w, target, HitResult::Unaffected, hit.warhead);
        // spec: S-0516 claim 12 (a surviving hit that reports unaffected, the final amount 0 or the armour among them, uncloaks)
        crate::cloak::machine::hit_taken(w, target, HitResult::Unaffected, 0);
        return HitResult::Unaffected;
    }
    let s = strength(w, target);
    let h = o.health;
    let mut lost = 0;
    let result = if m < 0 {
        // spec: S-0302 F4 (m < 0: H′ = min(H − m, S); the result is unaffected)
        let h2 = (i64::from(h) - i64::from(m)).min(i64::from(s)) as i32;
        set_health(w, target, h2);
        HitResult::Unaffected
    } else {
        // spec: S-0302 F4 (m′ = min(m, H), H′ = H − m′)
        let mp = m.min(h);
        lost = mp;
        let h2 = h - mp;
        let res = threshold(w, s, h, m, mp, h2);
        set_health(w, target, h2);
        res
    };
    let result = if result == HitResult::NowDead {
        killing_hit(w, target, &hit, &tc)
    } else {
        result
    };
    if result != HitResult::NowDead && result != HitResult::PostMortem {
        damage_effect_reaction(w, target, result, hit.warhead);
        // spec: S-0302 claim 12 and S-0516 claim 34 (a) (the disguise loss and the 2m′ disruption countdown)
        crate::cloak::disguise::hit_taken(w, target, result, lost);
        // spec: S-0516 claim 12 (a surviving hit, healing included, uncloaks; quirks.heal_reveals_cloaked)
        crate::cloak::machine::hit_taken(w, target, result, lost);
        // spec: S-0314 Terms (hostile-hit mark: a surviving hit past claim 8's stopping tests, attacker known, its house not allied from the owner's side)
        let attacker_house = hit
            .attacker
            .map(|a| w.obj(a).map_or(hit.house, |a| a.owner));
        if let Some(ah) = attacker_house
            && !w.allied(o.owner, ah)
            && let Some(ob) = w.obj_mut(target)
            && ob.health > 0
        {
            ob.combat.hostile_hit = true;
        }
    }
    // A presentation event for the sound reactions (S-0701 claim 20); no state, no draw.
    w.effects.push(crate::combat::effects::Effect::Hit {
        id: target,
        of: crate::econ::TypeRef {
            kind: o.kind,
            index: o.type_index,
        },
        owner: o.owner,
        pos: o.pos,
        result,
        attacker: hit.attacker,
        warhead: hit.warhead,
        health: (h, w.obj(target).map_or(0, |o| o.health)),
    });
    reactions(w, target, &hit, h, result);
    result
}

/// The reactions that follow a berserk hit which took effect (S-0302 r41
/// claim 10), after the hit's own damage step, none of whose reactions (anger,
/// retaliation, scatter, trigger event, damage effect, disguise loss) a
/// berserk hit makes: the no-threat fall-back (S-0215 claim 16; the hunt of
/// the berserk entry is replaced by what it queues), the change of fear of an
/// infantry unit (S-0107 claim 16) and the announcement that a harvester is
/// under attack (S-0701 claim 28). The crush response (S-0215 claim 39)
/// follows in [`apply`]. Request B of S-0208 claim 15 is the seam of the
/// scatter from damage, built for no hit (`docs/m8-claims/intake100.md`).
fn berserk_reactions(w: &mut World, id: ObjectId, hit: &Hit, falls: bool) {
    // spec: S-0302 r41 claim 10 (a team member that the hit makes berserk has left its team by then: no teams in our engine, so the team exception of the fall-back never holds it back)
    if falls {
        // spec: S-0215 claim 16 (the fall-back to rest, claims 13 to 15) and S-0610 claim 4 (a behaviour the fall-back queues replaces the queued hunt)
        crate::process::fall_back(w, id, false);
    }
    let Some(o) = w.obj(id) else { return };
    // spec: S-0302 r41 claim 10 and S-0107 claim 16 (the table of fear applies to a berserk hit, with the health ratio as it stands; a known attacker only selects its rows)
    if o.kind == Kind::Infantry && o.health > 0 {
        crate::combat::fear::damaged(w, id, hit.attacker.is_some());
    }
    let Some(o) = w.obj(id) else { return };
    // spec: S-0302 r41 claim 10 and S-0701 claim 28 (a berserk hit on a harvester is announced like any hit that takes health and leaves it alive; Q-2626 for the other sound reactions)
    let (of, owner, pos) = (
        crate::econ::TypeRef {
            kind: o.kind,
            index: o.type_index,
        },
        o.owner,
        o.pos,
    );
    w.effects.push(crate::combat::effects::Effect::Sound {
        sound: crate::combat::effects::Sound::HarvesterAttacked,
        of: Some(of),
        house: owner,
        pos: Some(pos),
    });
}

/// Sets an object's health.
fn set_health(w: &mut World, id: ObjectId, h: i32) {
    if let Some(o) = w.obj_mut(id) {
        o.health = h;
    }
}

/// The thresholds of S-0302 claim 12, F5.
fn threshold(w: &World, s: i32, h: i32, m: i32, mp: i32, h2: i32) -> HitResult {
    let r = w.rules();
    let mode = r.mode;
    // spec: S-0302 F5 (now dead ⇔ H′ = 0)
    if h2 == 0 {
        return HitResult::NowDead;
    }
    // spec: S-0302 F5 (X = RZ₅₃(S × ρ); now red ⇔ X < H and H − m′ < X, exact comparisons)
    let x = d(i64::from(s), mode).mul(r.general.condition_red, mode);
    let hd = d(i64::from(h), mode);
    let h2d = d(i64::from(h2), mode);
    if x.lt(hd) && h2d.lt(x) {
        return HitResult::NowRed;
    }
    // spec: S-0302 F5 (now yellow ⇔ m < H and H ≥ Y and H − m < Y, Y = ⌊S / 2⌋; quirks.yellow_state_at_half)
    let yellow = if r.quirks.yellow_state_at_half {
        (m < h) && h >= s / 2 && h - m < s / 2
    } else {
        // spec: S-0302 Edge cases (yellow_state_at_half off: strength × yellow ratio, as X)
        let y = d(i64::from(s), mode).mul(r.general.condition_yellow, mode);
        y.lt(hd) && h2d.lt(y)
    };
    let _ = mp;
    if yellow {
        HitResult::NowYellow
    } else {
        HitResult::Unchanged
    }
}

/// Readiness loss (S-0302 claim 10, F6).
fn readiness_loss(w: &mut World, id: ObjectId, dd: i32) {
    let Some(tc) = type_combat(w, id).cloned() else {
        return;
    };
    if !tc.readiness {
        return;
    }
    let mode = w.rules().mode;
    let q = w.rules().quirks;
    let s = strength(w, id);
    let Some(o) = w.obj(id) else { return };
    let a = o.combat.ammo;
    // spec: S-0302 F6 (A′ = max(trunc(RZ₅₃(A − RZ₅₃(A_max × RZ₅₃(κ × RZ₅₃(d / S))))), 0))
    let quotient = d(i64::from(dd), mode).div(d(i64::from(s), mode), mode);
    let loss = d(i64::from(tc.ammo), mode).mul(tc.readiness_factor.mul(quotient, mode), mode);
    let mut a2 = d(i64::from(a), mode).sub(loss, mode).to_i32_common().max(0);
    if !q.readiness_heal_over_max && a2 > tc.ammo && a2 > a {
        // spec: S-0302 Edge cases (readiness_heal_over_max off: at most the type's field)
        a2 = tc.ammo.max(a);
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.combat.ammo = a2;
        // spec: S-0301 claim 17 (the reload countdown restarts at once)
        ob.combat.reload_left = Some(crate::combat::fire::reload_delay(&tc, a2));
    }
}

/// The damage effect of S-0302 claim 13 and the building fires.
fn damage_effect_reaction(
    w: &mut World,
    id: ObjectId,
    result: HitResult,
    warhead: Option<WarheadId>,
) {
    let Some(o) = w.obj(id).cloned() else { return };
    let mode = w.rules().mode;
    let s = strength(w, id);
    let yellow_ratio = w.rules().general.condition_yellow;
    // spec: S-0214 Terms (health ratio: health / strength in floating point)
    let ratio = d(i64::from(o.health), mode).div(d(i64::from(s), mode), mode);
    let at_or_below = ratio.le(yellow_ratio);
    if matches!(result, HitResult::NowYellow | HitResult::NowRed) {
        // spec: S-0616 claim 1 (an object holds at most one damage-smoke system and makes a new one only when its place is empty)
        if at_or_below && o.combat.damage_smoke.is_none() && o.kind != Kind::Building {
            let tc = type_combat(w, id).cloned();
            let list = tc
                .as_ref()
                .map(|t| t.smoke_systems.clone())
                .unwrap_or_default();
            let n = list.len() as i32;
            // spec: S-0302 claim 13 (a draw over 0 … n − 1 when n > 0 and the height is above −10 leptons)
            if n > 0 && w.height_above_surface(id) > -10 {
                let half = w.rules().quirks.ranged_draw_half_span;
                let k = w.rng.ranged(0, n - 1, half);
                // spec: S-0302 claim 13 (a draw of k picks the (k + 1)-th eligible entry from the end)
                let name = list[(n - 1 - k) as usize].clone();
                let offset = tc.map_or([0; 3], |t| t.smoke_offset);
                crate::particles::damage_smoke(w, id, &name, offset);
            }
        }
        if o.kind == Kind::Building && result == HitResult::NowYellow {
            // spec: S-0616 claim 1 (after each hit that reports now yellow, S-0302 claim 12, the natural smoke's T is multiplied by 1.5; a now red hit does not multiply it)
            crate::emitters::natural_smoke_yellow(w, id);
        }
        if o.kind == Kind::Building && warhead.is_some_and(|h| w.rules().combat.warhead(h).sparky) {
            building_fires(w, id);
        }
    } else if !at_or_below && let Some(sys) = o.combat.damage_smoke {
        // spec: S-0302 claim 13 and S-0616 claim 1 (a hit that leaves the ratio above the yellow ratio marks the damage-smoke system; the object keeps it in its place until it leaves play)
        crate::particles::mark(w, sys);
    }
}

/// The cells of a building's foundation in the order of S-0302 claim 13:
/// rows by map y, cells of a row by map x; the 8-cell 3 × 3 word leaves
/// out the cell (2, 1). The foundation words are not imported as words
/// (only the size), so every foundation is the full rectangle here.
pub fn foundation_order(top_left: CellXY, f: (i32, i32)) -> Vec<CellXY> {
    // spec: S-0302 claim 13 (row by row by map y, within a row by map x)
    crate::objects::foundation_cells(top_left, f)
}

/// Building fires (S-0302 claim 13): one draw per foundation cell, each
/// followed by its loop draw for a fire of the first or second kind.
// spec: S-0302 claim 13 (fires of the first and second kind take a loop draw)
fn building_fires(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    let (fw, fh) = o.foundation;
    let half = w.rules().quirks.ranged_draw_half_span;
    let fires = w.rules().combat.general.damage_fire_types.clone();
    for c in foundation_order(o.pos.cell(), o.foundation) {
        // spec: S-0302 claim 13 (each cell's draw over 0 … W + H + 5)
        let k = w.rng.ranged(0, fw + fh + 5, half);
        // spec: S-0302 Constants (bands: 1 to 5 the first fire, 6 to 8 the second, 9 the third)
        let kind = match k {
            1..=5 => Some(0),
            6..=8 => Some(1),
            9 => Some(2),
            _ => None,
        };
        if let Some(i) = kind {
            if i < 2 {
                // spec: S-0302 claim 13 (a fire of the first or second kind: a loop draw over 1 … 3, directly after its cell's draw)
                let _loops = w.rng.ranged(1, 3, half);
            }
            // spec: S-0302 Edge cases (a missing fire entry starts no fire, a departure)
            if let Some(name) = fires.get(i).cloned() {
                let p = crate::world::Pos::centre(c, o.pos.z);
                crate::combat::effects::anim(w, &name, p);
            }
        }
    }
}

/// The reactions of a hit outside S-0302: fear (S-0107 claim 16), anger
/// (S-0309 claim 16), the no-threat fall-back (S-0215 claim 16),
/// retaliation (S-0309 claim 18).
fn reactions(w: &mut World, id: ObjectId, hit: &Hit, health_before: i32, result: HitResult) {
    let Some(o) = w.obj(id).cloned() else { return };
    let dealt = health_before - o.health;
    if dealt != 0
        && let Some(a) = hit.attacker
    {
        crate::combat::acquire::anger(w, id, a, dealt);
    }
    if o.kind == Kind::Infantry && o.health > 0 && dealt > 0 {
        crate::combat::fear::damaged(w, id, hit.attacker.is_some());
    }
    // spec: S-0215 claim 16 (damage from a known attacker that the actor survives: rated medium, "any damage that changes health and leaves the actor alive"; a heal is not damage, Q-2586 interim) and Q-2586 interim (after the hit's own effects, anger and fear, before retaliation; judged on the behaviour current when the hit lands); S-0215 claim 39 (the crush response, in `apply`, still follows)
    if dealt > 0 && o.health > 0 && hit.attacker.is_some() {
        crate::process::no_threat_fall_back(w, id);
    }
    if dealt > 0
        && o.health > 0
        && result != HitResult::Unaffected
        && let Some(a) = hit.attacker
    {
        crate::combat::acquire::retaliate(w, id, a);
    }
}

/// The killing hit (S-0302 claims 14 to 16): cyborg crippling, delayed
/// kills, credit and the death effects of S-0312.
fn killing_hit(
    w: &mut World,
    id: ObjectId,
    hit: &Hit,
    tc: &crate::combat::data::TypeCombat,
) -> HitResult {
    let Some(o) = w.obj(id).cloned() else {
        return HitResult::NowDead;
    };
    let rules = w.rules_arc();
    let mode = rules.mode;
    // spec: S-0302 claim 15 (a cyborg that is not prone, hit by damage that does not ignore defences, is crippled)
    if o.kind == Kind::Infantry
        && !hit.ignore_defences
        && rules.unit(o.kind, o.type_index).is_some_and(|u| u.cyborg)
        && !o.infantry.as_ref().is_some_and(|s| s.prone)
    {
        let s = strength(w, id);
        // spec: S-0302 F5 (H′ = max(trunc(RZ₅₃(S × 0.25)), 1))
        let h = d(i64::from(s), mode)
            .mul(crate::rules::nearest_double("0.25"), mode)
            .to_i32_common()
            .max(1);
        set_health(w, id, h);
        crate::infantry::request(w, id, crate::infantry::Seq::GoProne, true);
        if let Some(name) = rules.combat.general.infantry_explode.clone() {
            let p = o.pos;
            crate::combat::effects::anim(w, &name, p);
        }
        return HitResult::NowRed;
    }
    let delay = hit
        .warhead
        .map(|h| rules.combat.warhead(h))
        .filter(|h| h.causes_delay_kill);
    if o.kind == Kind::Building
        && rules.buildings[o.type_index as usize].building.delay_kill
        && let Some(h) = delay
    {
        // spec: S-0302 claim 16 (a delayed kill: 1 health, countdown K, the shorter kept)
        let k = delay_kill_countdown(
            w,
            h.delay_kill_frames,
            h.delay_kill_at_max,
            h.spread,
            hit.distance,
        );
        // spec: S-0312 claim 29 (a delayed-kill building's destruction raises come on the initial lethal hit, before its health is restored to 1 and the countdown is kept or started)
        crate::combat::death::destruction_raises(w, id, hit);
        let now = w.tick;
        let again = o.combat.delay_kill_until.is_some();
        if let Some(ob) = w.obj_mut(id) {
            if !again {
                ob.combat.charge_planter = None;
            }
            ob.health = 1;
            let until = now.wrapping_add(k);
            ob.combat.delay_kill_until = Some(match ob.combat.delay_kill_until {
                Some(u) if u.wrapping_sub(now) <= k => u,
                _ => until,
            });
            ob.combat.killed = true;
        }
        // spec: S-0302 claims 1 and 16 (killed at the delayed-kill hit: projectiles it fired before carry neither attacker nor attacking house)
        for p in w.projectiles.values_mut() {
            if p.attacker == Some(id) {
                p.attacker = None;
                p.house = None;
            }
        }
        // spec: S-0302 claim 16 (the kill is credited at the hit; again at a later hit with quirks.delay_kill_credited_again)
        if !again || rules.quirks.delay_kill_credited_again {
            credit(w, id, hit);
        }
        return HitResult::PostMortem;
    }
    // spec: S-0504 claim 13 and S-0312 claim 32 (a further killing hit on a sinking vessel takes the ordinary path: destruction
    // registered again and the death effects; claim 10's conditions leave it at 1 health, otherwise 0 health, still sinking)
    if o.combat.delay_kill_until.is_some() && !rules.quirks.delay_kill_credited_again {
        // spec: S-0302 Edge cases (delay_kill_credited_again off: no further credit)
    } else {
        credit(w, id, hit);
    }
    let _ = tc;
    crate::combat::death::kill(w, id, hit);
    HitResult::NowDead
}

/// Whether a killed object is dead and no longer takes part as a living
/// one: killed (S-0302 Terms) with no health left. A building in a
/// delayed-kill countdown is killed but keeps its health (S-0302 claim 16:
/// it does not die yet, stays in play, can be hit, healed and fires); an
/// infantry unit in its death sequence and a building awaiting removal
/// (S-0312 claim 21) have 0 health.
pub fn dead(o: &crate::world::Object) -> bool {
    o.combat.killed && o.health <= 0
}

/// The expiry of a building's delayed-kill countdown (S-0302 claim 16,
/// Q-0081 parts 1 and 2; S-0312 claim 3), in the building's own turn.
pub fn delay_kill_expiry_turn(w: &mut World, id: ObjectId) {
    let now = w.tick;
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(until) = o.combat.delay_kill_until else {
        return;
    };
    // spec: S-0302 claim 16 (the building's first turn at or after t + K)
    if now.wrapping_sub(until) < 0 {
        return;
    }
    // spec: S-0302 claim 16 (only while it is still alive and in play, and neither being erased nor under an electromagnetic pulse; both read their seams)
    if o.health <= 0
        || !o.on_map
        || crate::econ::power::being_erased(w, id)
        || crate::econ::power::under_pulse(w, id)
    {
        return;
    }
    let rules = w.rules_arc();
    let hut = o.kind == Kind::Building
        && rules.buildings[o.type_index as usize]
            .building
            .bridge_repair_hut;
    if hut {
        // spec: S-0302 claim 16 (a bridge repair hut clears the countdown and the recorded planter, with no expiry hit, and demolishes its bridge)
        if let Some(ob) = w.obj_mut(id) {
            ob.combat.delay_kill_until = None;
            ob.combat.charge_planter = None;
        }
        // spec: S-0511 claim 16 (the hut demolishes its bridge; no gate applies)
        crate::bridges::hut::demolish(w, id);
        return;
    }
    // spec: S-0302 claim 16 and claim 1's table (whole current health, distance 0, the demolition warhead, ignoring defences, no no-escape marker, no attacking house; the attacker is a recorded charge planter, and none is ever recorded in our engine)
    // spec: S-0519 claim 14 and S-0302 claim 16 (the expiry hit's attacker is the recorded charge planter while it exists)
    let planter = o.combat.charge_planter.filter(|&p| w.obj(p).is_some());
    if let Some(ob) = w.obj_mut(id) {
        ob.combat.charge_planter = None;
    }
    let hit = Hit {
        amount: o.health,
        warhead: rules.combat.general.c4_warhead,
        distance: 0,
        attacker: planter,
        house: None,
        ignore_defences: true,
        no_escape: false,
    };
    apply(w, id, hit);
}

/// The delayed-kill countdown K of S-0302 F7.
pub fn delay_kill_countdown(w: &World, f: i32, alpha: F64, spread: F64, r: i32) -> i32 {
    let rules = w.rules();
    let mode = rules.mode;
    let rr = if rules.quirks.delay_kill_whole_cell_spread {
        // spec: S-0302 F7 (R′ = 256 × trunc(s))
        // spec: S-0312 F6 (V = 256 × trunc(S) with quirks.delay_kill_whole_cell_spread on, R of S-0302 F3 off)
        256 * spread.to_i32_common()
    } else {
        // spec: S-0302 Edge cases (delay_kill_whole_cell_spread off: R of F3)
        spread.mul(d(256, mode), mode).to_i32_common()
    };
    let fd = d(i64::from(f), mode);
    // spec: S-0302 F7 (K = trunc(RZ₅₃(F + RZ₅₃(RZ₅₃(RZ₅₃(RZ₅₃(α × F) − F) / R′) × r))))
    alpha
        .mul(fd, mode)
        .sub(fd, mode)
        .div(d(i64::from(rr), mode), mode)
        .mul(d(i64::from(r), mode), mode)
        .add(fd, mode)
        .to_i32_common()
}

/// Credits a destruction (S-0302 claim 14; S-0311 claims 7 to 10).
pub(crate) fn credit(w: &mut World, id: ObjectId, hit: &Hit) {
    let attacker_owner = hit.attacker.and_then(|a| w.obj(a)).and_then(|a| a.owner);
    // spec: S-0302 claim 14 (the attacker when the attacking house is absent or its owner; otherwise the house)
    let (object, house) = match (hit.attacker, hit.house) {
        (Some(a), None) => (Some(a), attacker_owner),
        (Some(a), Some(h)) if attacker_owner == Some(h) => (Some(a), Some(h)),
        (_, Some(h)) => (None, Some(h)),
        (None, None) => (None, None),
    };
    // spec: S-0414 claims 21 and 22 (the result screen's kills, losses and destruction score: to the credited object's house, or to the attacking house that S-0302 claim 14 credits alone)
    let to = match (object, house) {
        (Some(a), _) => crate::econ::stats::Credit::Object(a),
        (None, Some(h)) => crate::econ::stats::Credit::House(h),
        (None, None) => crate::econ::stats::Credit::Nobody,
    };
    crate::econ::stats::destruction(w, id, to);
    crate::combat::veterancy::destruction_credited(w, id, object, house);
}

/// A hit on a terrain object (S-0302 claim 20).
fn terrain_hit(w: &mut World, id: ObjectId, hit: Hit) -> HitResult {
    let Some(o) = w.obj(id).cloned() else {
        return HitResult::Unaffected;
    };
    let rules = w.rules_arc();
    let t = rules
        .terrain
        .get(o.type_index as usize)
        .cloned()
        .unwrap_or_default();
    // spec: S-0302 claim 20 (stopped unless the warhead has the wood flag and the type is not immune)
    let Some(h) = hit.warhead else {
        return HitResult::Unaffected;
    };
    if !rules.combat.warhead(h).wood || t.immune {
        return HitResult::Unaffected;
    }
    // spec: S-0302 claim 20 (claims 6, 11 and 12 with the terrain type's armour class; F2 does not apply)
    let m = modified(w, &hit, hit.amount, hit.distance, t.armour);
    if m == 0 {
        return HitResult::Unaffected;
    }
    let s = t.strength;
    let h0 = o.health;
    if m < 0 {
        set_health(
            w,
            id,
            (i64::from(h0) - i64::from(m)).min(i64::from(s)) as i32,
        );
        return HitResult::Unaffected;
    }
    let mp = m.min(h0);
    let h2 = h0 - mp;
    let res = threshold(w, s, h0, m, mp, h2);
    set_health(w, id, h2);
    if h2 == 0 {
        crate::combat::death::terrain_destroyed(w, id);
    }
    res
}
