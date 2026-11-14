//! Target acquisition (S-0309): automatic searches, candidate tests, the
//! threat score, retaliation, anger and the enemy house, and the expected
//! remaining health.

use glasnost_num::geom::root_floor;
use glasnost_num::{F64, Rounding};

use crate::combat::check::{Answer, check};
use crate::combat::data::Ability;
use crate::combat::veterancy::{has_ability, type_combat};
use crate::combat::weapon::{armed, mean_damage, versus, would_use};
use crate::rules::{Behaviour, Kind};
use crate::world::{AttackMoveMemory, HouseId, HouseRole, ObjectId, Target, World};

/// τ_r, the retaliation threshold: the single nearest 0.01 (S-0304
/// Constants).
pub fn tau_r() -> F64 {
    // spec: S-0304 Constants (τ_r = 5368709 / 2²⁹)
    F64::from_parts(false, 5368709, -29, Rounding::TowardZero)
}

/// τ_a, the acquisition threshold: the single nearest 0.02 (S-0304
/// Constants).
pub fn tau_a() -> F64 {
    // spec: S-0304 Constants (τ_a = 5368709 / 2²⁸)
    F64::from_parts(false, 5368709, -28, Rounding::TowardZero)
}

/// Whether an object's owner is human-controlled for the unit rules: every
/// player house in our engine (S-0309 Terms, *Houses in our engine*) except
/// an explicitly marked test-only computer-controlled player house (S-0309
/// Terms, *Test-only exception*); the neutral and special houses are not.
pub fn player_owned(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.owner)
        .and_then(|h| w.houses.get(usize::from(h.0)))
        // spec: S-0309 Terms (Test-only exception: a marked player house takes the computer-owner rules of claim 25)
        .is_some_and(|h| h.role == HouseRole::Player && h.human)
}

/// ρ₃ between the reference points of two objects (S-0309 Formulas).
pub fn rho3(w: &World, a: ObjectId, b: ObjectId) -> i64 {
    let (p, q) = (w.reference_point(a), w.reference_point(b));
    let (dx, dy, dz) = (
        i64::from(p.x - q.x),
        i64::from(p.y - q.y),
        i64::from(p.z - q.z),
    );
    let r = w.rules();
    root_floor(
        (dx * dx + dy * dy + dz * dz) as u64,
        r.quirks.table_root_distance,
        r.mode,
    ) as i64
}

/// The anger increment of a hit and the enemy house (S-0309 claim 16, F8):
/// the victim owner's anger towards the attacker's owner grows by
/// conv((d / S) × C), then the enemy house is worked out again.
pub fn anger(w: &mut World, victim: ObjectId, attacker: ObjectId, dealt: i32) {
    let owner = w.obj(victim).and_then(|o| o.owner);
    let s = crate::combat::damage::strength(w, victim);
    let c = type_cost(w, victim);
    anger_of(w, owner, s, c, attacker, dealt);
}

/// [`anger`] with the victim's owner, strength `s` and type cost `c` taken
/// by the caller, for a victim that the hit's kill handling may have
/// removed or changed before the anger step (#173).
pub fn anger_of(
    w: &mut World,
    victim_owner: Option<HouseId>,
    s: i32,
    c: i32,
    attacker: ObjectId,
    dealt: i32,
) {
    let (Some(vo), Some(ao)) = (victim_owner, w.obj(attacker).and_then(|o| o.owner)) else {
        return;
    };
    // spec: S-0216 claim 8 (Removal before the groups: no house treats an idle test house as an enemy; design)
    if w.of_idle_house(attacker) {
        return;
    }
    let mode = w.rules().mode;
    // spec: S-0309 F8 (Δ = conv((d / S) × C), each a double-precision step)
    let delta = F64::from_i64(i64::from(dealt), mode)
        .div(F64::from_i64(i64::from(s), mode), mode)
        .mul(F64::from_i64(i64::from(c), mode), mode)
        .to_i32_common();
    let Some(h) = w.houses.get_mut(usize::from(vo.0)) else {
        return;
    };
    if let Some(a) = h.combat.anger.get_mut(usize::from(ao.0)) {
        // spec: S-0309 F8 (anger values are 32-bit integers; an addition wraps)
        *a = a.wrapping_add(delta);
    }
    recompute_enemy(w, vo.0 as usize);
}

/// A house's anger when its alliance with another house changes (S-0309
/// claim 16): made an enemy of `other`, its anger towards it grows by 1
/// point; made an ally, that anger becomes 0. The enemy house is then
/// worked out again (which clears it if the new ally was it).
pub fn alliance_anger(w: &mut World, house: HouseId, other: HouseId, ally: bool) {
    let Some(a) = w
        .houses
        .get_mut(usize::from(house.0))
        .and_then(|h| h.combat.anger.get_mut(usize::from(other.0)))
    else {
        return;
    };
    *a = if ally {
        0 // spec: S-0309 claim 16 (made an ally: the anger towards the new ally becomes 0)
    } else {
        // spec: S-0309 claim 16 and F8 (made an enemy: the anger grows by 1 point, a wrapping 32-bit addition)
        a.wrapping_add(1)
    };
    // spec: S-0309 claim 16 (each such change is followed by the same recomputation)
    recompute_enemy(w, usize::from(house.0));
}

/// Works a house's enemy house out again (S-0309 claim 16): the house,
/// other than itself, not allied and not defeated, with the greatest anger
/// above 0, the earlier in house order on ties (S-0414 owns defeat).
pub fn recompute_enemy(w: &mut World, house: usize) {
    let Some(h) = w.houses.get(house) else { return };
    let mut best: Option<(usize, i32)> = None;
    for (i, &a) in h.combat.anger.iter().enumerate() {
        let allied = h.allied_with(crate::world::HouseId(i as u8));
        // spec: S-0309 claim 16 (greatest anger above 0; ties: the earlier house in house order)
        let defeated = crate::econ::victory::defeated(w, crate::world::HouseId(i as u8));
        if i == house || allied || defeated || a <= 0 {
            continue;
        }
        if best.is_none_or(|(_, b)| a > b) {
            best = Some((i, a));
        }
    }
    let enemy = best.map(|(i, _)| crate::world::HouseId(i as u8));
    if let Some(h) = w.houses.get_mut(house) {
        h.combat.enemy = enemy;
    }
}

/// The type's cost of S-0314 claim 5: the cost field without house or
/// country factors (the production specs' building adjustments are not
/// modelled yet).
pub fn type_cost(w: &World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id) else { return 0 };
    let r = w.rules();
    match o.kind {
        Kind::Building => r
            .buildings
            .get(o.type_index as usize)
            .map_or(0, |b| b.raw.cost.unwrap_or(0)),
        Kind::Terrain => 0,
        // spec: S-0009 Import mapping (Cost absent: 0)
        _ => r.unit(o.kind, o.type_index).map_or(0, |u| u.cost),
    }
}

/// What a hit's retaliation came to (S-0208 Terms *may retaliate*; S-0215
/// claims 18 and 19).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Retaliation {
    /// The actor may not retaliate against the attacker: a condition of
    /// S-0215 claim 18 other than its distance rule, or one of the
    /// target-acquisition specs' (S-0309 claims 18 and 25 part h, S-0304
    /// claim 3), does not hold.
    MayNot,
    /// It may retaliate but does not retaliate against this hit: F7's
    /// distance rule holds it back, or the override to *attack* is refused
    /// and keeps its slots.
    Held,
    /// It retaliates: the override installed the attacker as its target.
    Done,
}

/// Retaliation of `actor` against the attacker of a hit it survived
/// (S-0309 claim 18, S-0304 claim 3, S-0215 claim 18). Returns whether it
/// may retaliate and whether it does, for the scatter from damage (S-0215
/// claim 19, S-0208 claim 15).
// spec: S-0215 F7 ((S + 0.5) × 256 = 256 S + 128 exactly)
pub fn retaliate(w: &mut World, actor: ObjectId, attacker: ObjectId) -> Retaliation {
    use Retaliation::{Done, Held, MayNot};
    let Some(o) = w.obj(actor) else {
        return MayNot;
    };
    // The actor and the attacker as the hit left them.
    let (o_kind, o_type_index, o_owner, o_target, cur) =
        (o.kind, o.type_index, o.owner, o.target, o.beh.seen());
    let Some(a) = w.obj(attacker) else {
        return MayNot;
    };
    let (a_kind, a_owner) = (a.kind, a.owner);
    if o_kind == Kind::Terrain {
        return MayNot;
    }
    // spec: S-0216 claim 8 (Removal before the groups: no object targets an idle test house's objects, a retaliation included; design)
    if w.of_idle_house(attacker) {
        return MayNot;
    }
    let rules = w.rules_arc();
    let Some(tc) = w
        .obj(actor)
        .and_then(|x| crate::combat::veterancy::type_combat_in(&rules, x.kind, x.type_index))
    else {
        return MayNot;
    };
    // spec: S-0309 claim 18 (the type's retaliate flag)
    if !tc.retaliate {
        return MayNot;
    }
    // spec: S-0309 claim 18 (neither a harvesting slave nor a slave master, S-0215 Terms) and Terms (not of a spawner type, whether or not any of its spawned aircraft is launching or alive)
    if crate::seams::is_harvesting_slave(w, actor)
        || crate::seams::is_slave_master(w, actor)
        || crate::spawn::spawn_rules(w, actor).is_some()
    {
        return MayNot;
    }
    // spec: S-0309 claim 18 and S-0601 claim 2 (not a mind-controlling object at capacity)
    if crate::mind::at_capacity(w, actor) {
        return MayNot;
    }
    // spec: S-0309 claim 18 (the attacker is not allied with it); S-0516 claim 36 (and does not appear disguised to its owner)
    if w.allied(o_owner, a_owner) || crate::cloak::appears_disguised(w, attacker, o_owner) {
        return MayNot;
    }
    // spec: S-0309 claim 18 and S-0614 claim 17 (X is not the parasite attached to it)
    if crate::parasite::parasite_of(w, actor) == Some(attacker) {
        return MayNot;
    }
    // spec: S-0309 claim 18 (mean weapon damage above 0, and armed)
    if mean_damage(w, actor) <= 0 || !armed(w, actor) {
        return MayNot;
    }
    let t = Target::Object(attacker);
    let (slot, weapon) = would_use(w, actor, Some(t));
    let Some(weapon) = weapon else { return MayNot };
    // spec: S-0304 claim 3 (retaliation needs a versus value above τ_r)
    if !versus(w, weapon, t).is_some_and(|v| v.gt(tau_r())) {
        return MayNot;
    }
    // spec: S-0309 claim 18 (the firing check without range answers neither illegal nor impossible)
    if matches!(
        check(w, actor, Some(t), slot, false),
        Answer::Illegal | Answer::Impossible
    ) {
        return MayNot;
    }
    let player = player_owned(w, actor);
    if player {
        // spec: S-0309 claim 18 (human-controlled owner, building attacker: not an explosive-charges infantry unit or holder of the ability)
        let charges = rules
            .unit(o_kind, o_type_index)
            .is_some_and(|u| u.raw.c4.unwrap_or(false))
            || has_ability(w, actor, Ability::C4);
        if a_kind == Kind::Building && charges {
            return MayNot;
        }
        // spec: S-0309 claim 18 (human-controlled owner: a vehicle deploying into an artillery building never retaliates)
        if o_kind == Kind::Vehicle
            && rules
                .unit(o_kind, o_type_index)
                .and_then(|u| u.raw.deploys_into.as_ref())
                .and_then(|n| rules.index_of(Kind::Building, n))
                .is_some_and(|b| rules.buildings[b as usize].building.artillary)
        {
            return MayNot;
        }
    }
    // spec: S-0215 claim 18 (only while the current behaviour has the retaliate property)
    if !cur.is_some_and(|b| rules.behaviour(b).retaliate) {
        return MayNot;
    }
    // spec: S-0208 Terms (may retaliate: every condition but claim 18's distance rule; does retaliate: claim 18 lets it override)
    let mut held = false;
    if player {
        // spec: S-0215 claim 18 (a mobile actor of a player house: guard, area guard or patrol only, unless player return fire)
        let listed = matches!(
            cur,
            Some(Behaviour::Guard | Behaviour::AreaGuard | Behaviour::Patrol)
        );
        if o_kind.is_mobile() && !listed && !rules.general.player_return_fire {
            return MayNot;
        }
        // spec: S-0215 claim 18 (an actor of a player house that already has a target never retaliates)
        if o_target.is_some() {
            return MayNot;
        }
        // spec: S-0215 claim 18 and F7 (an actor that cannot reach the attacker overrides only within (S + 0.5) × 256)
        let reaches = crate::combat::range::in_range(w, actor, Some(weapon), Some(t), None);
        // spec: S-0215 F7 and Import mapping (Sight: the actor's type's sight in cells, a building type's too; absent 0)
        let sight = match o_kind {
            Kind::Building => rules
                .buildings
                .get(o_type_index as usize)
                .map_or(0, |b| b.raw.econ.sight.unwrap_or(0)),
            _ => rules.unit(o_kind, o_type_index).map_or(0, |u| u.sight),
        };
        // spec: S-0208 Terms (the distance rule is not part of *may retaliate*: an actor it holds back may retaliate but does not)
        held = !reaches && rho3(w, actor, attacker) > i64::from(256 * sight + 128);
    }
    if !player
        && let Some(Target::Object(y)) = o_target
        && w.obj(y).is_some_and(|x| x.kind != Kind::Terrain)
    {
        // spec: S-0309 claim 25 part h (the neutral and special houses: only when the attacker's threat sum, d in cells as in a range search, is at least the present target's, compared exactly)
        let sum = |x| threat_sum(w, actor, x, false);
        if let (Some(sx), Some(sy)) = (sum(attacker), sum(y))
            && !sx.ge(sy)
        {
            return MayNot;
        }
    }
    // spec: S-0309 claim 25 part h (an object of the neutral or special house does not retaliate while it is draining an object with a draining weapon, S-0608)
    if !player && crate::drain::draining(w, actor) {
        return MayNot;
    }
    // spec: S-0215 claim 18 (an actor of a player house that cannot reach the attacker beyond F7's distance does not override)
    if held {
        return Held;
    }
    // spec: S-0215 claims 10 and 18 (an aircraft overrides to attack as any other actor; S-0501 T1 and T2 row 1 own its attack run and the restore at the target's loss) and S-0501 claim 52 (r154, Q-2792: on its pad too; standing on a pad, a contact, its height and its ammunition add no condition and remove none)
    if override_attack(w, actor, attacker) {
        Done
    } else {
        Held
    }
}

/// An override to *attack* with `target` as target and no destination
/// (S-0215 claims 10 and 18). Returns whether the target was installed.
pub fn override_attack(w: &mut World, actor: ObjectId, target: ObjectId) -> bool {
    override_attack_on(w, actor, Target::Object(target))
}

/// [`override_attack`] with any target, an object or a cell (S-0215 claim
/// 10; S-0209 claim 9 takes a wall's cell so). Returns whether the target
/// was installed: not when the override was refused and kept its slots.
pub fn override_attack_on(w: &mut World, actor: ObjectId, target: Target) -> bool {
    let q = w.rules().quirks;
    let Some(o) = w.obj_mut(actor) else {
        return false;
    };
    let slots_changed = o.beh.override_with(
        Behaviour::Attack,
        q.refused_override_retargets,
        q.override_suspends_queued,
    );
    let refused_kept = !slots_changed && !q.refused_override_retargets;
    if refused_kept {
        return false;
    }
    // spec: S-0215 claim 10 (the target and destination are put aside and replaced)
    o.beh.aside = Some((o.target, o.dest));
    o.target = Some(target);
    let mobile = o.kind.is_mobile();
    // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
    crate::emitters::retarget_let_go(w, actor);
    // spec: S-0215 claim 13 and S-0502 claim 28 (a destination clear requested by behaviour processing, here after the override installed its target: a balloon-hover vehicle with a nonempty destination can reject it, keeping destination, point, commitment and reservation)
    let retained = mobile && crate::jump::clear_rejected(w, actor);
    let Some(o) = w.obj_mut(actor) else {
        return true;
    };
    if mobile && !retained {
        o.dest = None;
    }
    o.combat.opportunity = false;
    // spec: S-0107 r46 claim 11, Changes of target (a retaliation at a hit, in the turn of the projectile or actor that delivers it, S-0215 claim 18 and S-0309 claim 18, and any other override's new target: the change ends an infantry unit's attack state and makes its request at once)
    crate::infantry::watch_test(w, actor);
    true
}

/// The expected remaining health's per-turn change (S-0309 claim 15, F7).
pub fn expected_health_turn(w: &mut World, id: ObjectId) {
    let t = w.tick;
    let Some(o) = w.obj_mut(id) else { return };
    let x = o.combat.expected_health;
    let h = o.health;
    // spec: S-0309 F7 (x′ = h if x ≥ h; max(x, −30) + 1 on ticks with ⌊t / 4⌋ odd; else x)
    o.combat.expected_health = if x >= h {
        h
    } else if (t.div_euclid(4)) % 2 == 1 {
        x.max(-30) + 1
    } else {
        x
    };
}

/// The kind of an automatic search (S-0309 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchKind {
    /// Centred on the searcher, radius of claim 8.
    Range,
    /// Centred on the guard spot.
    Area,
    /// No radius.
    MapWide,
}

/// Which search ran it, for the opportunity mark (S-0309 claim 24).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchContext {
    /// The per-tick search of claim 1's first row.
    PerTick,
    /// An attack-move search (claim 23), run as for an object with no target.
    AttackMoveFresh,
    /// An attack-move search that keeps a usable target.
    AttackMoveKeep,
    /// The area search of area guard.
    Area,
    /// The map-wide search of hunt.
    Hunt,
    /// The search at the end of a teleporting move (claim 1's last row).
    Teleport,
    /// The break-off search of a charging unit (S-0518 claim 15): no targeting-timer gate or reset draw, and the held target is neither tested nor protected.
    BreakOff,
}

/// The search classes (S-0309 claim 5): infantry, vehicles, aircraft,
/// buildings, and the friendly modifier.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Classes {
    /// Infantry.
    pub infantry: bool,
    /// Vehicles (with aircraft and objects that count as vehicles).
    pub vehicles: bool,
    /// Aircraft.
    pub aircraft: bool,
    /// Buildings.
    pub buildings: bool,
    /// Friendly modifier.
    pub friendly: bool,
}

/// The search classes of a searcher (S-0309 claims 5 and 6).
pub fn classes(w: &World, a: ObjectId) -> Classes {
    let Some(o) = w.obj(a) else {
        return Classes::default();
    };
    let r = w.rules();
    let player = player_owned(w, a);
    let mut c = Classes::default();
    // spec: S-0309 claim 6 (a healer for acquisition: infantry or vehicles, friendly)
    if crate::combat::weapon::healer_for_acquisition(w, a) && o.kind != Kind::Building {
        if o.kind == Kind::Infantry {
            c.infantry = true;
        } else if o.kind == Kind::Vehicle {
            c.vehicles = true;
        }
        c.friendly = true;
        return c;
    }
    match o.kind {
        // spec: S-0309 claim 5 (an aircraft takes no classes from its weapons)
        Kind::Aircraft | Kind::Terrain => return c,
        Kind::Vehicle => {
            let u = r.unit(o.kind, o.type_index);
            // spec: S-0309 claim 5 (a player's deploy-to-fire vehicle never searches; no-auto-fire likewise)
            if player && u.is_some_and(|u| u.vehicle.deploy_to_fire) {
                return c;
            }
        }
        _ => {}
    }
    let tc = type_combat(w, a);
    if player && tc.is_some_and(|t| t.no_auto_fire) {
        return c;
    }
    // spec: S-0309 claim 5 (classes from the weapons in slots 0 and 1, or the weapon number's for a switching type)
    let weapons: Vec<crate::combat::data::WeaponId> = if crate::combat::weapon::switching(w, a) {
        crate::combat::weapon::slot_weapon(w, a, crate::combat::weapon::weapon_number(w, a))
            .into_iter()
            .collect()
    } else {
        [0, 1]
            .iter()
            .filter_map(|&i| crate::combat::weapon::slot_weapon(w, a, i))
            .collect()
    };
    let armed = !weapons.is_empty();
    for wp in &weapons {
        if let Some(p) = r.combat.projectile_of(*wp) {
            if p.aa {
                c.aircraft = true;
            }
            if p.ag {
                c.infantry = true;
                c.vehicles = true;
                c.buildings = true;
            }
        }
    }
    if o.kind == Kind::Infantry {
        let u = r.unit(o.kind, o.type_index);
        let thief = u.is_some_and(|u| u.vehicle_thief);
        // spec: S-0309 Import mapping (the money-thief flag: `Thief`)
        let money = u.is_some_and(|u| u.raw.thief.unwrap_or(false));
        let infiltrate = u.is_some_and(|u| u.raw.infiltrate.unwrap_or(false));
        let engineer = u.is_some_and(|u| u.engineer);
        if !armed {
            // spec: S-0309 claim 5 (an unarmed vehicle thief: vehicles only; an unarmed money thief with infiltrate: no class; others none)
            c = Classes::default();
            if thief {
                c.vehicles = true;
            } else if !(money && infiltrate) {
                return c;
            }
        }
        // spec: S-0309 claim 5 (an armed infantry unit with an infantry-only primary warhead searches for infantry only)
        if armed
            && crate::combat::weapon::slot_weapon(w, a, 0)
                .and_then(|p| r.combat.warhead_of(p))
                .is_some_and(|h| h.infantry_only())
        {
            c = Classes {
                infantry: c.infantry,
                ..Classes::default()
            };
        }
        if money {
            c.buildings = true; // spec: S-0309 claim 5 (the money-thief flag also searches for buildings)
        }
        let charges = u.is_some_and(|u| u.raw.c4.unwrap_or(false))
            || crate::combat::veterancy::has_ability(w, a, crate::combat::data::Ability::C4);
        if !player && charges {
            // spec: S-0309 claim 25 part b (an infantry unit of the neutral or special house with explosive charges also searches for buildings)
            c.buildings = true;
        }
        if engineer {
            // spec: S-0309 claim 5 (an engineer never searches for infantry or vehicles)
            c.infantry = false;
            c.vehicles = false;
        }
    }
    c
}

/// The radius R and ring count n of a range or area search (S-0309 F1).
fn radius(w: &World, a: ObjectId, kind: SearchKind) -> (i32, i32) {
    let Some(o) = w.obj(a) else { return (0, 0) };
    let r = w.rules();
    let u = r.unit(o.kind, o.type_index);
    let g = u.map_or(0, |u| u.guard_range);
    let engineer = u.is_some_and(|u| u.engineer);
    let w0 = crate::combat::range::firer_weapon_range(w, a, 0);
    let w1 = crate::combat::range::firer_weapon_range(w, a, 1);
    let mut big_r = match kind {
        // spec: S-0309 F1 (range: g when g ≠ 0 and not an engineer, else 0)
        SearchKind::Range => {
            if g != 0 && !engineer {
                g
            } else {
                0
            }
        }
        // spec: S-0309 F1 (area: R = min(max(2G, 0), 4096), G = g or max(w₀, w₁))
        SearchKind::Area => {
            let gg = if g != 0 { g } else { w0.max(w1) };
            (2 * gg).clamp(0, 4096)
        }
        SearchKind::MapWide => 0,
    };
    // spec: S-0309 F1 (a healer for acquisition in guard: R = 512)
    if crate::combat::weapon::healer_for_acquisition(w, a) && o.beh.seen() == Some(Behaviour::Guard)
    {
        big_r = 512;
    }
    let n = if big_r != 0 {
        big_r / 256 // spec: S-0309 F1 (n = trunc(R / 256))
    } else {
        let tc = type_combat(w, a);
        let ww = if tc.is_some_and(|t| t.underwater && t.organic && t.self_healing) {
            // spec: S-0309 F1 (a type that is underwater, organic and self-healing uses g for w)
            g
        } else if crate::combat::weapon::switching(w, a) {
            crate::combat::range::firer_weapon_range(
                w,
                a,
                crate::combat::weapon::weapon_number(w, a),
            )
        } else {
            w0.max(w1)
        };
        let b = tc.map_or(0, |t| t.air_range_bonus);
        // spec: S-0309 F1 (n = trunc(w / 256) + 1 + trunc(b / 256) when R = 0)
        ww / 256 + 1 + b / 256
    };
    (big_r, n)
}

/// The cells of square ring r around c in the order of S-0309 F2.
fn search_ring(c: glasnost_map::CellXY, r: i32) -> Vec<glasnost_map::CellXY> {
    use glasnost_map::CellXY;
    if r == 0 {
        return vec![c];
    }
    let mut out = Vec::new();
    // spec: S-0309 F2 (rows y − r and y + r in pairs by column, then columns x − r and x + r in pairs by row)
    for x in c.x - r..=c.x + r {
        out.push(CellXY::new(x, c.y - r));
        out.push(CellXY::new(x, c.y + r));
    }
    for y in c.y - r + 1..=c.y + r - 1 {
        out.push(CellXY::new(c.x - r, y));
        out.push(CellXY::new(c.x + r, y));
    }
    out
}

/// The friendly-row exceptions of a searcher (S-0309 Terms): attacks
/// friendlies, berserk, a passenger of a mind-controlled open-topped
/// transport (the permanent mark is not modelled).
fn friendly_exception(w: &World, a: ObjectId) -> bool {
    let Some(o) = w.obj(a) else { return false };
    let attacks = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.raw.attack_friendlies.unwrap_or(false));
    let berserk = o.combat.berserk.is_some();
    // spec: S-0309 Terms (a passenger of a mind-controlled open-topped transport, S-0601 Terms)
    attacks || berserk || controlled_carrier_house(w, a).is_some()
}

/// The house that controls the open-topped transport searcher `a` rides
/// in, when that transport is mind-controlled (S-0309 Terms, claim 10,
/// table C row "controller's allies").
pub(crate) fn controlled_carrier_house(w: &World, a: ObjectId) -> Option<Option<HouseId>> {
    let t = crate::transport::open_topped_carrier(w, a)?;
    crate::mind::mind_controlled(w, t).then(|| w.obj(t).and_then(|o| o.owner))
}

/// A searcher whose type attacks friendlies or that is berserk (S-0309
/// claim 10, the table's second row).
fn attacks_or_berserk(w: &World, a: ObjectId) -> bool {
    let Some(o) = w.obj(a) else { return false };
    o.combat.berserk.is_some()
        || w.rules()
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.raw.attack_friendlies.unwrap_or(false))
}

/// Whether T's kind is among the classes (table C row "class").
fn class_ok(w: &World, c: Classes, t: ObjectId) -> bool {
    let Some(o) = w.obj(t) else { return false };
    match o.kind {
        Kind::Infantry => c.infantry,
        Kind::Vehicle => c.vehicles,
        // spec: S-0309 table C (a search for vehicles also accepts aircraft and objects that count as vehicles)
        Kind::Aircraft => c.aircraft || c.vehicles,
        Kind::Building => c.buildings || (c.vehicles && crate::targeting::counts_as_vehicle(w, t)),
        Kind::Terrain => false,
    }
}

/// The answer of table C for one candidate (S-0309 claim 12), before the
/// draw of claim 25 (g).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    /// A row rejects T, with no draw.
    Reject,
    /// Every row lets T through.
    Accept,
    /// Claim 25 (g): T's disguise blinks for a computer-owner searcher, and
    /// every row outside the six of part g lets T through, so the draw is
    /// taken. `rest`: the rows of the six that [`candidate`] tests (capture,
    /// engineer, storage, bridge layers, healer; score is the caller's) let
    /// T through too.
    Draw { rest: bool },
}

/// Table C for one candidate, with claim 25 (g)'s draw taken when it is due
/// (S-0309 claim 12; the score row follows in the caller).
fn admit(
    w: &mut World,
    a: ObjectId,
    t: ObjectId,
    c: Classes,
    big_r: i32,
    kind: SearchKind,
    zone: bool,
) -> bool {
    match candidate(w, a, t, c, big_r, kind, zone) {
        Verdict::Reject => false,
        Verdict::Accept => true,
        Verdict::Draw { rest } => {
            let half = w.rules().quirks.ranged_draw_half_span;
            // spec: S-0309 claim 25 (g) (one ranged draw R(0, 99) from the scenario random stream, taken for this test)
            let d = w.rng.ranged(0, 99, half);
            // spec: S-0309 claim 25 (g) (the house's entry of the disguise-detection list, selected by its difficulty index; S-0309 Terms, Test-only exception: a marked house's oracle index)
            let entry = w
                .obj(a)
                .and_then(|o| o.owner)
                .and_then(|h| w.houses.get(usize::from(h.0)))
                .and_then(|h| usize::try_from(h.econ.setup.difficulty).ok())
                .and_then(|i| w.rules().combat.general.disguise_detection.get(i).copied());
            // spec: S-0309 claim 25 (g) (let through when the draw is at most the entry) and Import mapping (no such entry: no disguised target is let through by the draw, a departure)
            rest && entry.is_some_and(|e| d <= e)
        }
    }
}

/// The candidate tests of table C (S-0309 claim 12). `big_r` is the
/// search radius R (0: the weapon's reach), `zone` whether the zone row
/// applies (claims 7 and 11). The rows of claim 25 (g) that reject only
/// after its draw set `six` instead of returning at once.
// spec: S-0309 Constants (lowest searchable height −20 leptons)
fn candidate(
    w: &World,
    a: ObjectId,
    t: ObjectId,
    c: Classes,
    big_r: i32,
    kind: SearchKind,
    zone: bool,
) -> Verdict {
    // spec: S-0309 claim 25 (g) (an object that the capture, engineer, storage, bridge-layers, healer or score row rejects has still taken the draw)
    let mut six = false;
    let r = w.rules();
    let Some(ao) = w.obj(a) else {
        return Verdict::Reject;
    };
    let Some(to) = w.obj(t) else {
        return Verdict::Reject;
    };
    // spec: S-0309 table C (exists, self)
    if t == a || to.health <= 0 || !to.on_map || crate::combat::damage::dead(to) {
        return Verdict::Reject;
    }
    // spec: S-0216 claim 8 (Removal before the groups: no object targets an idle test house's objects; design)
    if w.of_idle_house(t) {
        return Verdict::Reject;
    }
    let ttc = type_combat(w, t);
    // spec: S-0309 table C (legal target)
    let legal = match to.kind {
        Kind::Building => r.buildings[to.type_index as usize]
            .raw
            .legal_target
            .unwrap_or(true),
        Kind::Terrain => false,
        _ => r
            .unit(to.kind, to.type_index)
            .is_none_or(|u| u.raw.legal_target.unwrap_or(true)),
    };
    if !legal || !class_ok(w, c, t) {
        return Verdict::Reject;
    }
    let tt = Target::Object(t);
    let (slot, weapon) = would_use(w, a, Some(tt));
    // spec: S-0309 table C (the firing check without range answers illegal)
    if check(w, a, Some(tt), slot, false) == Answer::Illegal {
        return Verdict::Reject;
    }
    // spec: S-0309 table C and S-0304 claim 4 (the versus value at most τ_a)
    if let Some(wp) = weapon
        && r.combat.warhead_of(wp).is_some()
        && !versus(w, wp, tt).is_some_and(|v| v.gt(tau_a()))
    {
        return Verdict::Reject;
    }
    let atc = type_combat(w, a);
    // spec: S-0309 table C (land targeting: mode 2 and a floor-bound target on a cell that is not water)
    if atc.is_some_and(|x| x.land_targeting == 2)
        && !crate::combat::weapon::airborne_by_height(w, t)
        && w.cells
            .cell(to.pos.cell())
            .is_some_and(|cl| cl.land != glasnost_map::terrain::land::WATER)
    {
        return Verdict::Reject;
    }
    // spec: S-0309 table C (playfield; no-threat; underground below −20)
    if !w.cells.usable(to.pos.cell()) {
        return Verdict::Reject;
    }
    if to.beh.seen().is_some_and(|b| r.behaviour(b).no_threat) {
        return Verdict::Reject;
    }
    if w.height_above_surface(t) < -20 {
        return Verdict::Reject;
    }
    // spec: S-0309 table C (zone, in the searches claims 7 and 11 restrict)
    if zone && ao.kind.is_mobile() && ao.kind != Kind::Aircraft && !same_zone(w, a, t) {
        return Verdict::Reject;
    }
    let allied = w.allied(ao.owner, to.owner);
    let healer = crate::combat::weapon::healer_for_acquisition(w, a);
    let engineer = r.unit(ao.kind, ao.type_index).is_some_and(|u| u.engineer);
    let full = to.health >= crate::combat::damage::strength(w, t);
    // spec: S-0309 table C (friendly)
    if !friendly_exception(w, a) && allied && (!(healer || engineer) || full) {
        return Verdict::Reject;
    }
    // spec: S-0309 table C (healer: an infantry or vehicle healer rejects an object not allied)
    if healer && matches!(ao.kind, Kind::Infantry | Kind::Vehicle) && !allied {
        six = true;
    }
    // spec: S-0309 table C (healer aircraft)
    if healer
        && ao.kind == Kind::Vehicle
        && allied
        && to.kind == Kind::Aircraft
        && !friendly_exception(w, a)
        && w.height_above_surface(t) > 0
    {
        six = true;
    }
    // spec: S-0309 table C (distance: F3)
    if kind != SearchKind::MapWide {
        let delta = if crate::combat::weapon::airborne_by_height(w, t) {
            rho2(w, a, t)
        } else {
            rho3(w, a, t)
        };
        let pass = if big_r > 0 {
            delta <= i64::from(big_r)
        } else if weapon.is_some() {
            crate::combat::range::in_range(w, a, weapon, Some(tt), None)
        } else {
            delta <= i64::from(r.unit(ao.kind, ao.type_index).map_or(0, |u| u.guard_range))
        };
        if !pass {
            return Verdict::Reject;
        }
    }
    // spec: S-0309 table C (harvester immunity: the map sets it and T's type is a game-wide harvester type)
    if crate::combat::area::harvester_immune(w, to.kind, to.type_index) {
        return Verdict::Reject;
    }
    // spec: S-0309 table C (invisible building)
    if to.kind == Kind::Building && r.buildings[to.type_index as usize].invisible {
        return Verdict::Reject;
    }
    // spec: S-0309 table C (cloak: fully cloaked, not on a sensed cell of A's owner, not owned by A's owner); S-0516 claim 21
    if crate::cloak::fully_cloaked(w, t)
        && !crate::cloak::senses(w, ao.owner, t)
        && to.owner != ao.owner
    {
        return Verdict::Reject;
    }
    // spec: S-0309 table C (disguise: appears disguised to A's owner and A's type cannot detect disguises)
    let mut draw = false;
    if crate::cloak::appears_disguised(w, t, ao.owner) && !atc.is_some_and(|x| x.detect_disguise) {
        // spec: S-0309 claim 25 (g) (a searcher of the computer-owner houses: rejected unless T's disguise is blinking and the draw lets it through; S-0516 claim 39 for the blink)
        if player_owned(w, a) || !crate::cloak::disguise::blinking(w, t) {
            return Verdict::Reject;
        }
        draw = true;
    }
    let player = player_owned(w, a);
    // spec: S-0309 table C (passive house: T's owner multiplayer-passive and A's owner a player house)
    let passive = to
        .owner
        .and_then(|h| w.houses.get(usize::from(h.0)))
        .and_then(|h| r.countries.get(h.country as usize))
        .is_some_and(|c| c.passive);
    let a_unit = r.unit(ao.kind, ao.type_index);
    let a_engineer = ao.kind == Kind::Infantry && a_unit.is_some_and(|u| u.engineer);
    let t_building = (to.kind == Kind::Building).then(|| &r.buildings[to.type_index as usize]);
    // spec: S-0309 claim 25 part e (for a searcher of the neutral or special house, or of a marked test house, the two rows let T through only when T is a building, A an infantry unit, and an engineer of another owner may capture it or A may occupy it)
    let capture = a_engineer
        && t_building.is_some_and(|b| b.raw.needs_engineer.unwrap_or(false))
        && to.owner != ao.owner;
    // spec: S-0309 claim 25 part e (occupation: A's type can occupy buildings and T can be occupied by A, the structures specs' test: S-0508 claim 1); S-0508 claim 1 (r19, Q-2926 and Q-2927: asked only of a searcher of an occupier type, so an assaulter type that is not an occupier type never passes; no behaviour or search condition; the count compared is the occupants inside, no place held for units on their way)
    let occupy = || t_building.is_some() && crate::structures::garrison::may_enter(w, a, t);
    let house_exception = ao.kind == Kind::Infantry && (capture || occupy());
    if passive && (player || !house_exception) {
        return Verdict::Reject;
    }
    // spec: S-0309 table C (insignificant: T not mind-controlled, not a building of a non-passive owner, A's owner a player house; claim 25 part e for the neutral and special houses)
    if ttc.is_some_and(|x| x.insignificant)
        && !crate::mind::mind_controlled(w, to.id)
        && !(to.kind == Kind::Building && !passive)
        && (player || !house_exception)
    {
        return Verdict::Reject;
    }
    // spec: S-0309 table C (controller's allies: A a passenger of a mind-controlled open-topped transport and T allied with the house that controls the transport)
    if let Some(h) = controlled_carrier_house(w, a)
        && w.allied(h, to.owner)
    {
        return Verdict::Reject;
    }
    // spec: S-0309 table C (train: T a train and A a vehicle thief)
    if r.unit(ao.kind, ao.type_index)
        .is_some_and(|u| u.vehicle_thief)
        && r.unit(to.kind, to.type_index).is_some_and(|u| u.train)
    {
        return Verdict::Reject;
    }
    let money_thief =
        ao.kind == Kind::Infantry && a_unit.is_some_and(|u| u.raw.thief.unwrap_or(false));
    // spec: S-0309 table C (capture: A a money thief, or an engineer of the neutral or special house, and T not a building whose type is capturable)
    if (money_thief || (!player && a_engineer))
        && !t_building.is_some_and(|b| b.raw.capturable.unwrap_or(false))
    {
        six = true;
    }
    // spec: S-0309 table C (storage: A a money thief and T's type without storage capacity; Storage absent: 0)
    if money_thief && t_building.is_none_or(|b| b.raw.storage.unwrap_or(0) <= 0) {
        six = true;
    }
    // spec: S-0309 table C (harmless building: a player searcher, a building with no current weapon or a threat value of 0; not for an engineer)
    if player
        && !engineer
        && to.kind == Kind::Building
        && !crate::targeting::counts_as_vehicle(w, t)
    {
        let weaponless = crate::combat::weapon::current_weapon(w, t).is_none();
        // spec: S-0309 Terms (threat value: of a building that holds occupants, its occupant count times the game-wide threat per occupant; otherwise the threat-posed field of the unit in its tank bunker, if one is there, otherwise its own type's; #171)
        let occupants = crate::structures::garrison::count(w, t);
        let threat = if occupants > 0 {
            i64::from(occupants) * i64::from(w.rules().combat.general.threat_per_occupant)
        } else {
            i64::from(
                crate::structures::bunker::occupant(w, t)
                    .and_then(|v| type_combat(w, v))
                    .map_or_else(|| ttc.map_or(0, |x| x.threat_posed), |v| v.threat_posed),
            )
        };
        if weaponless || threat == 0 {
            return Verdict::Reject;
        }
    }
    // spec: S-0309 table C (engineer: T not a building, or allied and above the red ratio)
    if engineer && to.kind != Kind::Building {
        six = true;
    }
    // spec: S-0309 table C (bridge layers)
    let deck_cell = |id: ObjectId| w.obj(id).is_some_and(|o| w.high_bridge(o.pos.cell()));
    if deck_cell(a)
        && deck_cell(t)
        && (ao.layer == crate::world::Layer::Deck) != (to.layer == crate::world::Layer::Deck)
    {
        six = true;
    }
    // spec: S-0309 table C (strong scan: expected remaining health 0 or less)
    if atc.is_some_and(|x| x.scan_mode == crate::combat::data::ScanMode::Strong)
        && to.combat.expected_health <= 0
    {
        return Verdict::Reject;
    }
    match (draw, six) {
        (true, _) => Verdict::Draw { rest: !six },
        (false, true) => Verdict::Reject,
        (false, false) => Verdict::Accept,
    }
}

/// ρ₂ between reference points.
fn rho2(w: &World, a: ObjectId, b: ObjectId) -> i64 {
    let (p, q) = (w.reference_point(a), w.reference_point(b));
    let (dx, dy) = (i64::from(p.x - q.x), i64::from(p.y - q.y));
    let r = w.rules();
    root_floor(
        (dx * dx + dy * dy) as u64,
        r.quirks.table_root_distance,
        r.mode,
    ) as i64
}

/// Whether T's cell lies in the connected region of A's movement zone that
/// holds A's cell (S-0309 claims 7 and 11; S-0212 claim 4). A's own cell
/// is read for its deck whenever it is a bridge cell, whatever layer A is
/// in; T's cell is read in the layer T occupies, always for A's movement
/// zone. Two cells of no reach region count as the same region. When A's
/// deck read finds no bridge link, the restriction is omitted.
pub fn same_zone(w: &World, a: ObjectId, t: ObjectId) -> bool {
    use crate::regions::{Reach, reach_read};
    let (Some(ao), Some(to)) = (w.obj(a), w.obj(t)) else {
        return false;
    };
    let Some(u) = w.rules().unit(ao.kind, ao.type_index) else {
        return true;
    };
    // spec: S-0309 claim 7 (revision 21, Q-1502 answered: the searcher's own cell is read for its deck whenever it is a bridge cell, whatever layer the searcher is in)
    let own = reach_read(w, ao.pos.cell(), u.zone, true);
    if own == Reach::LoneDeck {
        // spec: S-0309 claims 7 and 11 (no-link deck exception: the searcher's own deck read finds no bridge link, so the connected-region restriction is omitted for the whole search)
        return true;
    }
    // spec: S-0309 claim 7 (the object's cell is read in the layer the object occupies, for the searcher's movement zone)
    let other = reach_read(
        w,
        to.pos.cell(),
        u.zone,
        to.layer == crate::world::Layer::Deck,
    );
    // spec: S-0309 claims 7 and 11 and S-0212 claim 4 (regions compared as S-0212 claim 4 does: two cells of no reach region count as the same region, and so do two off-map cells; a no-link deck read differs from every other region)
    match (own, other) {
        (Reach::Region(x), Reach::Region(y)) => x == y,
        (Reach::None, Reach::None) | (Reach::OffMap, Reach::OffMap) => true,
        _ => false,
    }
}

/// v(X → Y): the versus value of the weapon X would use against Y, 0 when
/// it has none (S-0309 F4).
fn v_of(w: &World, x: ObjectId, y: ObjectId) -> F64 {
    let (_, wp) = would_use(w, x, Some(Target::Object(y)));
    wp.and_then(|wp| versus(w, wp, Target::Object(y)))
        .unwrap_or(F64::ZERO)
}

/// The threat sum of a candidate (S-0309 Terms, F4): the argument of conv
/// in F4, before conversion and before the modifiers of claim 14.
/// `ground_pass`: the candidate is considered in the ground-layer pass of
/// the map-wide search, whose distance is in leptons.
// spec: S-0309 F4 (cells of 256 leptons)
fn threat_sum(w: &World, a: ObjectId, t: ObjectId, ground_pass: bool) -> Option<F64> {
    let r = w.rules();
    let mode = r.mode;
    let ao = w.obj(a)?;
    let to = w.obj(t)?;
    let tc = type_combat(w, a)?;
    let ttc = type_combat(w, t);
    let [cme, cte, cst, csg, cd] = tc.coefficients;
    let vat = v_of(w, a, t);
    let vta = v_of(w, t, a);
    // spec: S-0309 F4 (t = −(c_te × v(T → A)) when T's target is A, else c_te × v(T → A))
    let tt = if to.target == Some(Target::Object(a)) {
        cte.mul(vta, mode).negated()
    } else {
        cte.mul(vta, mode)
    };
    let st = ttc.map_or(F64::ZERO, |x| x.special_threat);
    let mut p = tt.add(cst.mul(st, mode), mode);
    // spec: S-0309 F4 (B when A's owner has an enemy house and that house owns T)
    let enemy = ao
        .owner
        .and_then(|h| w.houses.get(usize::from(h.0)))
        .and_then(|h| h.combat.enemy);
    if enemy.is_some() && enemy == to.owner {
        p = p.add(r.combat.general.enemy_house_bonus, mode);
    }
    let s = crate::combat::damage::strength(w, t);
    let ht = F64::from_i64(i64::from(to.health), mode).div(F64::from_i64(i64::from(s), mode), mode);
    let (_, wp) = would_use(w, a, Some(Target::Object(t)));
    let g = r.unit(ao.kind, ao.type_index).map_or(0, |u| u.guard_range);
    // spec: S-0309 F4 (k = trunc(range / 256), or trunc(g / 256) without a weapon)
    let k = wp.map_or(g / 256, |x| r.combat.weapon(x).range / 256);
    let rh = rho3(w, a, t);
    // spec: S-0309 F4 and claim 7 (d in cells; in leptons in the ground-layer pass of the map-wide search, a landed aircraft considered in that pass included, quirks.hunt_distance_in_leptons)
    let dd = if ground_pass && r.quirks.hunt_distance_in_leptons {
        rh
    } else {
        rh / 256
    };
    let e = (dd - i64::from(k)).max(0);
    // spec: S-0309 F4 (score = conv((c_d × e + ((P + c_me × v(A → T)) + c_sg × h_T)) + 100000))
    let sum = cd
        .mul(F64::from_i64(e, mode), mode)
        .add(
            p.add(cme.mul(vat, mode), mode).add(csg.mul(ht, mode), mode),
            mode,
        )
        .add(F64::from_i64(100000, mode), mode);
    Some(sum)
}

/// The threat score of a candidate (S-0309 claims 13 and 14, F4 to F6,
/// F9); `None` rejects it (a score of exactly 0).
// spec: S-0309 F5 and F6 (N = trunc(L / 256))
fn score(w: &World, a: ObjectId, t: ObjectId, ground_pass: bool) -> Option<i32> {
    let r = w.rules();
    let mode = r.mode;
    let ao = w.obj(a)?;
    let to = w.obj(t)?;
    let tc = type_combat(w, a)?;
    let s = crate::combat::damage::strength(w, t);
    let mut sc = threat_sum(w, a, t, ground_pass)?.to_i32_common();
    // spec: S-0309 F5 (normal mode: halved at expected health ≤ 0, doubled at ≤ trunc(S / 2))
    if tc.scan_mode == crate::combat::data::ScanMode::Normal {
        let x = to.combat.expected_health;
        if x <= 0 {
            sc /= 2;
        } else if x <= s / 2 {
            sc = sc.wrapping_mul(2);
        }
    }
    // spec: S-0309 F6 (a suppressing current weapon: × 0.5 per allied building cell in rings 1 … N − 1)
    let supress =
        crate::combat::weapon::current_weapon(w, a).is_some_and(|x| r.combat.weapon(x).supress);
    if supress {
        let n = r.combat.general.fire_supress / 256;
        let mut m = 0;
        let c = to.pos.cell();
        for ring in 1..n {
            for cell in search_ring(c, ring) {
                if w.cells.cell(cell).is_none() {
                    continue;
                }
                let has = w
                    .cells
                    .list(cell, crate::world::Layer::Ground)
                    .iter()
                    .any(|&b| {
                        w.obj(b).is_some_and(|ob| {
                            ob.kind == Kind::Building && w.allied(ao.owner, ob.owner)
                        })
                    });
                if has {
                    m += 1;
                }
            }
        }
        if m > 0 {
            // spec: S-0309 F6 (score′ = conv(score × 0.5^m), equal to trunc(score / 2^m))
            let p = F64::from_parts(false, 1, -m, Rounding::TowardZero);
            sc = F64::from_i64(i64::from(sc), mode)
                .mul(p, mode)
                .to_i32_common();
        }
    }
    // spec: S-0309 F9 (exactly 0 rejects, quirks.zero_score_rejected; below 1 counts as 1)
    if sc == 0 && r.quirks.zero_score_rejected {
        return None;
    }
    Some(sc.max(1))
}

/// The object a visited cell offers (S-0309 claim 10).
fn offered(w: &World, a: ObjectId, c: glasnost_map::CellXY) -> Option<ObjectId> {
    let ao = w.obj(a)?;
    let deck = w.cells.list(c, crate::world::Layer::Deck);
    let list: Vec<ObjectId> = if !deck.is_empty() {
        deck.to_vec()
    } else {
        w.cells.list(c, crate::world::Layer::Ground).to_vec()
    };
    let units: Vec<ObjectId> = list
        .into_iter()
        .filter(|&x| x != a && w.obj(x).is_some_and(|o| o.kind != Kind::Terrain))
        .collect();
    let healer = crate::combat::weapon::healer_for_acquisition(w, a);
    let any = friendly_exception(w, a);
    let carrier = controlled_carrier_house(w, a);
    // spec: S-0309 claim 10 (the first object the searcher selects)
    let sel = units.iter().copied().find(|&x| {
        let Some(o) = w.obj(x) else { return false };
        let allied = w.allied(ao.owner, o.owner);
        if healer {
            allied && o.health < crate::combat::damage::strength(w, x)
        } else if let Some(h) = carrier
            && !attacks_or_berserk(w, a)
        {
            // spec: S-0309 claim 10 (a passenger of a mind-controlled open-topped transport selects an object allied with the house that controls the transport)
            w.allied(h, o.owner)
        } else if any {
            true
        } else {
            !allied
        }
    });
    // spec: S-0309 claim 10 (none selected: the last of them, low confidence)
    sel.or_else(|| units.last().copied())
}

/// Runs an automatic search (S-0309 claims 4 to 17, 20). Returns the pick.
// spec: S-0309 claims 4, 7 and 9 (timer N + U(0, 2); the two passes of the map-wide search; early stops trunc(n / 4), trunc(n / 2))
pub fn search(
    w: &mut World,
    a: ObjectId,
    kind: SearchKind,
    ctx: SearchContext,
) -> Option<ObjectId> {
    let rules = w.rules_arc();
    let half = rules.quirks.ranged_draw_half_span;
    crate::regions::ensure(w);
    let ao = w.obj(a).cloned()?;
    // spec: S-0215 claim 33 and S-0309 claim 4 (every search restarts the targeting timer: N + U(0, 2), its first draw)
    let n = if ao.beh.seen() == Some(Behaviour::AreaGuard) {
        rules.general.guard_area_targeting_delay
    } else {
        rules.general.normal_targeting_delay
    };
    // spec: S-0518 claim 15 (the break-off search has no targeting-timer reset draw and leaves the timer alone)
    if ctx != SearchContext::BreakOff {
        let restart = n.wrapping_add(w.rng.ranged(0, 2, half));
        let tick = w.tick;
        if let Some(ob) = w.obj_mut(a) {
            ob.targeting_timer = tick.wrapping_add(restart);
        }
    }
    // Claim 4: a kept target (an order's, or a usable opportunity target).
    // spec: S-0518 claim 15 (the held building target of the break-off search is not tested for usability or protected by S-0309 claim 4's retention rule, and is not cleared to make the search possible)
    let fresh = matches!(
        ctx,
        SearchContext::AttackMoveFresh | SearchContext::BreakOff
    );
    if let Some(t) = ao.target
        && !fresh
    {
        if !ao.combat.opportunity {
            return None;
        }
        let (slot, _) = would_use(w, a, Some(t));
        let ans = check(w, a, Some(t), slot, true);
        if ans == Answer::Impossible && ao.spawn.is_some() {
            // spec: S-0309 claim 4 (a spawner that gets impossible keeps the target and resets its spawn controller, S-0506 claim 11)
            crate::spawn::impossible(w, a);
            return None;
        }
        if !matches!(
            ans,
            Answer::Illegal | Answer::OutOfRange | Answer::Impossible
        ) {
            return None;
        }
        // spec: S-0309 claim 4 (an unusable opportunity target is dropped; the same search may pick anew); for a spawner the drop requests the conditional pending-target clear of S-0506 claim 25, even if the search then picks a new target
        if let Some(ob) = w.obj_mut(a) {
            crate::spawn::clear_own_target(ob);
            ob.combat.opportunity = false;
            ob.combat.burst = 0;
        }
    }
    // spec: S-0309 claim 5 (a building always searches with the range kind); claim 22 (the limited-scan mark turns area and map-wide searches into range searches)
    let kind = if ao.kind == Kind::Building || ao.combat.limited_scan {
        SearchKind::Range
    } else {
        kind
    };
    let c = classes(w, a);
    let mut cands: Vec<(ObjectId, i32)> = Vec::new();
    if kind == SearchKind::MapWide {
        let ids: Vec<ObjectId> = w.objects.keys().copied().collect();
        // spec: S-0309 claim 7 (aircraft first when looking for aircraft, then ground-layer objects; allied ones only for a healer, every one with a friendly-row exception)
        let healer = crate::combat::weapon::healer_for_acquisition(w, a);
        for pass in 0..2 {
            for &x in &ids {
                let Some(o) = w.obj(x) else { continue };
                // spec: S-0309 claim 7 (the second pass considers every infantry unit, vehicle, aircraft and building of the ground render layer, so landed aircraft are among them, considered again after the first pass; an object neither an aircraft nor in the ground layer, such as a hovering jumpjet, is never found)
                if pass == 0 && (o.kind != Kind::Aircraft || !c.aircraft) {
                    continue;
                }
                // spec: S-0501 claim 48 (a) and its Changes for S-0309 claim 7 (the ground-layer pass takes the objects filed in the ground render layer's list, whatever their height: an aircraft that a factory release left above its pad, not re-filed since, is among them)
                // spec: S-0306 r72 claim 17 (An aircraft that cannot land, in flight: it stays in the ground list, S-0102 claim 4, and S-0309 claim 7's ground-layer pass reads the list, so it considers the aircraft in flight as a landed aircraft there, in both passes of a search that looks for aircraft and in the second pass alone otherwise; the connected-region test and table C apply as to any candidate)
                // spec: S-0309 r114 claim 7 (an aircraft whose type cannot land and whose movement is neither rocket nor drop-pod movement stays in the ground list in flight, so the ground-layer pass considers it in flight, as it considers a landed one)
                // spec: S-0306 r75 claim 17, S-0501 r111 claim 48 (a) and S-0102 r43 (Q-2467: quirks.layer_change_on_events_only is a drawing setting with no consequence in the simulation; the ground-layer pass reads the list as the switch's on position keeps it, in both positions, so this pass reads the simulation's own marks and never the client's switch)
                if pass == 1
                    && !crate::temporal::ground_layer(w, x)
                    && !crate::air::ground_filed(w, x)
                    && !crate::air::kept_filed(w, x)
                {
                    continue;
                }
                let allied = w.allied(ao.owner, o.owner);
                let consider = if healer {
                    allied
                } else {
                    friendly_exception(w, a) || !allied
                };
                if !consider {
                    continue;
                }
                // spec: S-0309 claim 7 (the zone test applies to the ground-layer objects of the second pass)
                if admit(w, a, x, c, 0, kind, pass == 1)
                    && let Some(s) = score(w, a, x, pass == 1)
                {
                    cands.push((x, s));
                }
            }
        }
    } else {
        let (big_r, nrings) = radius(w, a, kind);
        let centre = match kind {
            SearchKind::Area => ao
                .guard_spot
                .map(|g| crate::combat::range::target_point(w, g).cell())
                .unwrap_or(ao.pos.cell()),
            _ => ao.pos.cell(),
        };
        // spec: S-0309 claim 9 (a range or area search that looks for aircraft first scans the tracked flying objects that S-0303's flying-object search finds around the centre cell, radius max(n, 1) cells; they precede cell candidates on equal scores, and count in K for the early stops)
        if c.aircraft && matches!(kind, SearchKind::Range | SearchKind::Area) {
            // spec: S-0309 table C row "class" (the scan of airborne objects also accepts infantry and vehicles)
            let ca = Classes {
                infantry: true,
                vehicles: true,
                ..c
            };
            for x in crate::combat::flying::candidates(w, centre, nrings) {
                let Some(o) = w.obj(x) else { continue };
                // spec: S-0309 claim 9 (only objects on the map and not in the ground render layer, each once; allied ones only with the friendly-row exceptions)
                // spec: S-0309 r114 claim 9 (the layer read from state and height: an aircraft whose type cannot land and whose movement is neither rocket nor drop-pod movement is not left out as a ground-layer object once it is tracked and above height 0, although it flies in the ground list, S-0306 claim 17)
                if x == a || !o.on_map || crate::temporal::ground_layer(w, x) {
                    continue;
                }
                if w.allied(ao.owner, o.owner) && !friendly_exception(w, a) {
                    continue;
                }
                if admit(w, a, x, ca, big_r, kind, false)
                    && let Some(s) = score(w, a, x, false)
                {
                    cands.push((x, s));
                }
            }
        }
        let only_aircraft =
            kind == SearchKind::Range && c.aircraft && !c.infantry && !c.vehicles && !c.buildings;
        if !only_aircraft && nrings >= 1 {
            let r1 = nrings / 4; // spec: S-0309 claim 9 (early stops at trunc(n / 4) and trunc(n / 2))
            let r2 = nrings / 2;
            for ring in 0..nrings {
                for cell in search_ring(centre, ring) {
                    if w.cells.cell(cell).is_none() {
                        continue;
                    }
                    // spec: S-0309 claim 11 (an area search skips cells outside the searcher's zone region)
                    let cands_here: Vec<ObjectId> = if rules.quirks.search_one_object_per_cell {
                        offered(w, a, cell).into_iter().collect()
                    } else {
                        // spec: S-0309 Edge cases (search_one_object_per_cell off: every unit and building of the cell)
                        [crate::world::Layer::Ground, crate::world::Layer::Deck]
                            .iter()
                            .flat_map(|&l| w.cells.list(cell, l).to_vec())
                            .filter(|&x| {
                                x != a && w.obj(x).is_some_and(|o| o.kind != Kind::Terrain)
                            })
                            .collect()
                    };
                    for x in cands_here {
                        let allied = w.allied(ao.owner, w.obj(x).and_then(|o| o.owner));
                        let tested = !allied
                            || crate::combat::weapon::healer_for_acquisition(w, a)
                            || friendly_exception(w, a);
                        if tested
                            && admit(w, a, x, c, big_r, kind, kind == SearchKind::Area)
                            && let Some(s) = score(w, a, x, false)
                        {
                            cands.push((x, s));
                        }
                    }
                }
                // spec: S-0309 claim 9 (the last ring looked at: r₁ or r₂ when K is not empty, else n − 1)
                if (ring == r1 || ring == r2) && !cands.is_empty() {
                    break;
                }
            }
        }
    }
    let distributed = rules
        .unit(ao.kind, ao.type_index)
        .is_some_and(|u| u.raw.distributed_fire.unwrap_or(false));
    let pick = if distributed {
        if kind == SearchKind::MapWide {
            if let Some(ob) = w.obj_mut(a) {
                ob.combat.fired_at.clear();
            }
            if rules.quirks.hunt_ignores_distributed_fire {
                // spec: S-0309 claim 20 (a map-wide search picks nothing and empties the list, quirks.hunt_ignores_distributed_fire)
                None
            } else {
                // spec: S-0309 Edge cases (hunt_ignores_distributed_fire off: claim 17 among the map-wide candidates, the list empty afterwards)
                best_of(cands.iter().copied())
            }
        } else {
            let list = ao.combat.fired_at.clone();
            let not_on: Vec<&(ObjectId, i32)> =
                cands.iter().filter(|(x, _)| !list.contains(x)).collect();
            // spec: S-0309 claim 20 (the best not on the list, keeping those of L in C; or the best of C and an empty list)
            let (pick, keep) = if cands.is_empty() {
                (None, Vec::new())
            } else if !not_on.is_empty() {
                let best = best_of(not_on.into_iter().copied());
                let keep: Vec<ObjectId> = list
                    .iter()
                    .copied()
                    .filter(|x| cands.iter().any(|(c, _)| c == x))
                    .collect();
                (best, keep)
            } else {
                (best_of(cands.iter().copied()), Vec::new())
            };
            if let Some(ob) = w.obj_mut(a) {
                ob.combat.fired_at = keep;
            }
            pick
        }
    } else {
        // spec: S-0309 claim 17 (the highest score; ties: the first considered)
        best_of(cands.iter().copied())
    };
    if pick.is_none()
        && ao.combat.limited_scan
        && let Some(ob) = w.obj_mut(a)
    {
        // spec: S-0309 claim 22 (a search that finds nothing clears the limited-scan mark)
        ob.combat.limited_scan = false;
    }
    if let Some(x) = pick {
        set_target_from_search(w, a, x, ctx, distributed, ao.target);
    }
    pick
}

fn best_of(it: impl Iterator<Item = (ObjectId, i32)>) -> Option<ObjectId> {
    let mut best: Option<(ObjectId, i32)> = None;
    for (x, s) in it {
        if best.is_none_or(|(_, b)| s > b) {
            best = Some((x, s));
        }
    }
    best.map(|(x, _)| x)
}

/// Sets a searched target (S-0309 claims 4 and 24).
fn set_target_from_search(
    w: &mut World,
    a: ObjectId,
    x: ObjectId,
    ctx: SearchContext,
    distributed: bool,
    before: Option<Target>,
) {
    // `before` is the target held before the search, even when the search dropped it (claim 24, table Search outcome row 1).
    let t = redirect(w, a, Target::Object(x));
    let rules = w.rules_arc();
    let expected = match t {
        Some(Target::Object(y)) if !distributed => {
            let (_, wp) = would_use(w, a, Some(Target::Object(y)));
            wp.filter(|wp| {
                !rules
                    .combat
                    .projectile_of(*wp)
                    .is_some_and(|p| p.inaccurate)
            })
            .map(|wp| (y, crate::combat::damage::expected_damage(w, a, wp, y)))
        }
        _ => None,
    };
    if let Some(ob) = w.obj_mut(a) {
        match t {
            Some(_) => ob.target = t,
            // spec: S-0309 claim 24 (a nonempty target redirected to none leaves the object with no target and requests a spawner's conditional pending-target clear)
            None => crate::spawn::redirected_to_none(ob),
        }
        // spec: S-0309 claim 24 (a different target set by the per-tick search is an opportunity target); claim 1 (so is one set by the teleport search)
        ob.combat.opportunity = matches!(ctx, SearchContext::PerTick | SearchContext::Teleport)
            && t.is_some()
            && t != before;
    }
    // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
    crate::emitters::retarget_let_go(w, a);
    // spec: S-0309 claim 4 (a pick lowers the target's expected remaining health by the expected damage, unless the projectile is inaccurate)
    if let Some((y, e)) = expected
        && let Some(tob) = w.obj_mut(y)
    {
        tob.combat.expected_health = tob.combat.expected_health.wrapping_sub(e);
    }
}

/// The redirects of S-0309 claim 24: a bunkered vehicle gives its bunker
/// to an object with a temporal weapon, a locomotor-warhead primary or an
/// airstrike team; the object itself gives its own cell; a dead or sinking
/// object, or an infantry unit in a death sequence, gives no target.
pub fn redirect(w: &World, a: ObjectId, t: Target) -> Option<Target> {
    let Target::Object(x) = t else { return Some(t) };
    // spec: S-0309 claim 24 row 1 (a unit sitting in a tank bunker, when the object has a temporal weapon, its primary weapon's warhead is a locomotor warhead, or its type has an airstrike team: the bunker building; the later rows judge the target as this row leaves it)
    let (t, x) = match crate::structures::bunker::bunkered_in(w, x) {
        Some(k) if bunker_redirecting(w, a) => (Target::Object(k), k),
        _ => (t, x),
    };
    // spec: S-0309 claim 24 (the object itself: its own cell)
    if x == a {
        return w.obj(a).map(|o| Target::Cell(o.pos.cell()));
    }
    let o = w.obj(x)?;
    let dying = o.kind == Kind::Infantry
        && o.infantry.as_ref().and_then(|s| s.seq).is_some_and(|q| {
            matches!(
                q,
                crate::infantry::Seq::Die1
                    | crate::infantry::Seq::Die2
                    | crate::infantry::Seq::Die3
                    | crate::infantry::Seq::Die4
                    | crate::infantry::Seq::Die5
                    | crate::infantry::Seq::WetDie1
                    | crate::infantry::Seq::WetDie2
                    | crate::infantry::Seq::AirDeathStart
                    | crate::infantry::Seq::AirDeathFall
                    | crate::infantry::Seq::AirDeathEnd
            )
        });
    // spec: S-0309 claim 24 (a dead or sinking object, or an infantry unit whose current sequence is a death kind of S-0107: die 1 to 5, wet die 1 and 2, the three air-death kinds: no target)
    if crate::combat::damage::dead(o) || o.health <= 0 || dying || crate::naval::sinking(w, x) {
        return None;
    }
    Some(t)
}

/// Whether object `a` is one that S-0309 claim 24's first row redirects:
/// a temporal weapon, a locomotor warhead on its primary weapon, or an
/// airstrike team.
fn bunker_redirecting(w: &World, a: ObjectId) -> bool {
    let r = w.rules();
    let tc = type_combat(w, a);
    // spec: S-0309 claim 24 row 1 (the object has a temporal weapon, S-0514 Terms)
    let temporal = crate::temporal::has_beam(w, a);
    // spec: S-0309 claim 24 row 1 (its primary weapon's warhead is a locomotor warhead)
    let locomotor = tc
        .and_then(|tc| tc.slots[0])
        .and_then(|p| r.combat.weapon(p).warhead)
        .is_some_and(|h| r.combat.warhead(h).is_locomotor);
    // spec: S-0309 claim 24 row 1 and Import mapping (its type can call airstrikes: an airstrike team above 0, whether or not an airstrike is in progress)
    let airstrike = tc.is_some_and(|tc| tc.airstrike_team > 0);
    temporal || locomotor || airstrike
}

/// Whether an object may search passively (S-0309 claim 2).
pub fn may_search_passively(w: &World, a: ObjectId) -> bool {
    let Some(o) = w.obj(a) else { return false };
    let r = w.rules();
    let Some(tc) = type_combat(w, a) else {
        return false;
    };
    // spec: S-0309 claim 2 (the passive-acquire flag; armed; not an empty occupiable building; not a player's engineer)
    if !tc.passive_acquire || !armed(w, a) {
        return false;
    }
    // spec: S-0309 claim 2 (no temporal weapon of its own is erasing a target) and S-0301 r73 claim 9 (a building that drops the target its beam holds keeps the beam attached, so while that target is being erased it takes none by a search; a departed warp member is no longer being erased, S-0513 claim 39a)
    if w.temporal
        .beams
        .get(&a)
        .is_some_and(|b| crate::temporal::erased(w, b.target))
    {
        return false;
    }
    if o.kind == Kind::Building {
        let b = &r.buildings[o.type_index as usize].building;
        // spec: S-0309 claim 2 (only an occupiable building that holds no occupants is kept from searching: a garrisoned one searches)
        if b.can_be_occupied && crate::structures::garrison::count(w, a) == 0 {
            return false;
        }
    }
    let engineer = r.unit(o.kind, o.type_index).is_some_and(|u| u.engineer);
    // spec: S-0309 claim 2 and S-0601 claim 2 (not a mind-controlling object that already controls as many objects as it can)
    !(engineer && player_owned(w, a)) && !crate::mind::at_capacity(w, a)
}

/// The opportunity gate (S-0309 claim 3).
fn opportunity_gate(w: &World, a: ObjectId) -> bool {
    let Some(o) = w.obj(a) else { return false };
    let r = w.rules();
    let u = r.unit(o.kind, o.type_index);
    if u.is_some_and(|u| u.opportunity_fire) {
        return true;
    }
    match o.beh.seen() {
        Some(Behaviour::Move) => {
            // spec: S-0309 claim 3 (move, arrived, a balloon-hover type or a vehicle that deploys in place)
            let arrived = matches!(o.dest, Some(Target::Cell(c)) if c == o.pos.cell());
            let balloon = u.is_some_and(|u| u.raw.balloon_hover.unwrap_or(false));
            let simple = o.kind == Kind::Vehicle && u.is_some_and(|u| u.simple_deployer);
            arrived && (balloon || simple)
        }
        Some(Behaviour::Guard) => {
            // spec: S-0309 claim 3 (guard, unless the slot-1 weapon (slot 0 for spray attack) is area-fire and the one it would use)
            let spray = u.is_some_and(|u| u.raw.spray_attack.unwrap_or(false));
            let s = if spray { 0 } else { 1 };
            let area = crate::combat::weapon::slot_weapon(w, a, s)
                .is_some_and(|x| r.combat.weapon(x).area_fire);
            !(area && crate::combat::weapon::choose_slot(w, a, o.target) == s)
        }
        _ => o.kind == Kind::Building,
    }
}

/// The per-tick automatic search under behaviours (S-0215 claim 33; S-0309
/// claims 1 to 3 and 23), after the actor's processing of the tick.
pub fn per_tick(w: &mut World, a: ObjectId) {
    let tick = w.tick;
    let Some(o) = w.obj(a) else { return };
    if crate::combat::damage::dead(o)
        || o.health <= 0
        || matches!(o.kind, Kind::Aircraft | Kind::Terrain)
    {
        return;
    }
    // spec: S-0215 claim 33 (an actor whose targeting timer has run out)
    if tick < o.targeting_timer {
        return;
    }
    if o.attack_move.is_some() {
        attack_move_search(w, a);
        return;
    }
    // spec: S-0215 claim 33 (move, harvest or guard; buildings search in guard)
    let listed = matches!(
        o.beh.seen(),
        Some(Behaviour::Move | Behaviour::Harvest | Behaviour::Guard)
    );
    if !listed || !may_search_passively(w, a) || !opportunity_gate(w, a) {
        return;
    }
    search(w, a, SearchKind::Range, SearchContext::PerTick);
}

/// The search at the end of a teleporting move (S-0309 claim 1, last row;
/// S-0207 claim 20 f): a range search when the object has no target and
/// claim 2 and the opportunity gate of claim 3 allow it.
pub fn teleport_search(w: &mut World, a: ObjectId) {
    let Some(o) = w.obj(a) else { return };
    // spec: S-0309 claim 1 (a teleporting move has just ended, no target, claims 2 and 3 allow it)
    if o.target.is_some() || !may_search_passively(w, a) || !opportunity_gate(w, a) {
        return;
    }
    search(w, a, SearchKind::Range, SearchContext::Teleport);
}

/// Whether `t` is within reach of `a` (S-0309 claim 23): the range test of
/// S-0307 answers yes for it with the weapon `a` would use against it
/// (S-0308). No target is not within reach.
pub(crate) fn within_reach(w: &World, a: ObjectId, t: Option<Target>) -> bool {
    t.is_some_and(|t| {
        let (_, wp) = would_use(w, a, Some(t));
        crate::combat::range::in_range(w, a, wp, Some(t), None)
    })
}

/// The attack-move searches of S-0309 claim 23 (a).
fn attack_move_search(w: &mut World, a: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let Some(o) = w.obj(a).cloned() else { return };
    let tick = w.tick;
    let Some(mem) = o.attack_move else { return };
    let within = |w: &World, t: Option<Target>| -> bool { within_reach(w, a, t) };
    match mem {
        AttackMoveMemory::Dest(_) => {
            if o.combat.engaged && within(w, o.target) {
                return;
            }
            let ctx = if o.combat.engaged {
                SearchContext::AttackMoveFresh
            } else {
                SearchContext::AttackMoveKeep
            };
            if ctx == SearchContext::AttackMoveFresh
                && let Some(ob) = w.obj_mut(a)
            {
                // spec: S-0309 claim 23 (running as for an object with no target does not itself request a spawner's pending-target clear)
                ob.target = None;
                crate::emitters::cleared(ob);
                ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
            }
            search(w, a, SearchKind::Range, ctx);
            if w.obj(a).is_some_and(|x| x.target.is_some())
                && let Some(ob) = w.obj_mut(a)
            {
                // spec: S-0309 claim 23 (a target: queues attack with an immediate start and counts as engaged)
                ob.beh
                    .queue_and_start_at_once(Behaviour::Attack, tick, stale);
                ob.combat.engaged = true;
            } else {
                // spec: S-0215 r118 claim 11 (the attack-move rule discards a target without the change-of-target ending: where its search finds no other, the attack state lasts to the no-target ending in the firing turn)
                crate::infantry::watch_carry(w, a, o.target);
            }
        }
        AttackMoveMemory::Target(mem) => {
            if within(w, Some(mem)) || (o.combat.engaged && within(w, o.target)) {
                return;
            }
            if let Some(ob) = w.obj_mut(a) {
                // spec: S-0309 claim 23 (running as for an object with no target does not itself request a spawner's pending-target clear)
                ob.target = None;
                crate::emitters::cleared(ob);
                ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
            }
            search(w, a, SearchKind::Range, SearchContext::AttackMoveFresh);
            let found = w.obj(a).is_some_and(|x| x.target.is_some());
            if let Some(ob) = w.obj_mut(a) {
                if found {
                    ob.beh
                        .queue_and_start_at_once(Behaviour::Attack, tick, stale);
                    ob.combat.engaged = true;
                } else {
                    // spec: S-0309 claim 23 (nothing found: the memory's target again)
                    ob.target = Some(mem);
                }
            }
            if !found {
                // spec: S-0215 r118 claim 11 (the attack-move rule puts a remembered target back without the change-of-target ending: the state stays as it was)
                crate::infantry::watch_carry(w, a, o.target);
            }
            // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
            crate::emitters::retarget_let_go(w, a);
        }
    }
}
