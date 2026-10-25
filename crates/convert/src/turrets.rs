//! The turret table of a multi-turret vehicle type and the fixed-name
//! charge-turret rule (S-0105 Terms, claim 43 and Import mapping, *Turret
//! table pairs*).
//!
//! A turret table has 18 entries, one per weapon number, each a turret
//! index or *none*. Only the vehicle type with the fixed type name reads
//! its pairs of keys (`quirks.turret_table_fixed_type_only`); every other
//! type keeps an all-*none* table. In every rules layer that has the
//! type's section, each of the 17 pairs gives a turret index (its stem's
//! default when the layer leaves the index key out) and a weapon number
//! (−1 when left out). A pair with a weapon number 0 … 17 sets that entry;
//! a later pair, or a later layer, setting the same entry wins, and the
//! table is never cleared. A pair with no weapon number (absent or −1)
//! leaves the table alone; for the fixed type name it decides the
//! charge-turret flag instead (`quirks.turret_pair_no_weapon_sets_charge_flag`).

use glasnost_ini::value;

use crate::layers::{KeyState, TypeRec, What};

/// Entries of a turret table: one per weapon number 0 … 17.
// spec: S-0105 Constants (turret table size: 18 entries, one per weapon number 0 … 17)
pub const TABLE_LEN: usize = 18;

/// The fixed vehicle type name whose section fills the turret table,
/// compared ignoring letter case.
// spec: S-0105 Import mapping (the pairs are read only when the type name is FV, letters in either case)
pub const FIXED_TYPE: &[u8] = b"FV";

/// The 17 stems of the pairs in Import mapping order, each with the turret
/// index a layer that leaves the index key out gives that pair.
// spec: S-0105 Import mapping (stems in this order; absent index: 1 for Repair, 2 for MachineGun, 3 for Flak, 0 for every other stem)
pub const STEMS: [(&str, i32); 17] = [
    ("Normal", 0),
    ("Repair", 1),
    ("MachineGun", 2),
    ("Flak", 3),
    ("Pistol", 0),
    ("Sniper", 0),
    ("Shock", 0),
    ("Explode", 0),
    ("BrainBlast", 0),
    ("RadCannon", 0),
    ("Chrono", 0),
    ("TerroristExplode", 0),
    ("Cow", 0),
    ("Initiate", 0),
    ("Virus", 0),
    ("YuriPrime", 0),
    ("Guardian", 0),
];

/// A pair's weapon number when the layer leaves the key out.
// spec: S-0105 Import mapping (weapon number absent: −1, read again in every layer)
const NO_WEAPON: i32 = -1;

/// The turret index that means *none*.
// spec: S-0105 Import mapping (a turret index of −1 means none)
const NONE_INDEX: i32 = -1;

/// The 34 keys of the pairs: each stem's index key, then its weapon key.
pub const PAIR_KEYS: [&str; 34] = [
    "NormalTurretIndex",
    "NormalTurretWeapon",
    "RepairTurretIndex",
    "RepairTurretWeapon",
    "MachineGunTurretIndex",
    "MachineGunTurretWeapon",
    "FlakTurretIndex",
    "FlakTurretWeapon",
    "PistolTurretIndex",
    "PistolTurretWeapon",
    "SniperTurretIndex",
    "SniperTurretWeapon",
    "ShockTurretIndex",
    "ShockTurretWeapon",
    "ExplodeTurretIndex",
    "ExplodeTurretWeapon",
    "BrainBlastTurretIndex",
    "BrainBlastTurretWeapon",
    "RadCannonTurretIndex",
    "RadCannonTurretWeapon",
    "ChronoTurretIndex",
    "ChronoTurretWeapon",
    "TerroristExplodeTurretIndex",
    "TerroristExplodeTurretWeapon",
    "CowTurretIndex",
    "CowTurretWeapon",
    "InitiateTurretIndex",
    "InitiateTurretWeapon",
    "VirusTurretIndex",
    "VirusTurretWeapon",
    "YuriPrimeTurretIndex",
    "YuriPrimeTurretWeapon",
    "GuardianTurretIndex",
    "GuardianTurretWeapon",
];

/// A turret table: entry *w* is the turret index for weapon number *w*,
/// `None` for *none*. An index may lie outside the type's turret count; it
/// is kept and draws nothing (S-0105 claim 12).
pub type TurretTable = [Option<i32>; TABLE_LEN];

/// The all-*none* table every type has unless its pairs are read.
// spec: S-0105 Terms (every entry is none unless the import fills it)
pub const EMPTY: TurretTable = [None; TABLE_LEN];

/// Whether `name` is the fixed type name (S-0105 Import mapping).
pub fn is_fixed_type(name: &[u8]) -> bool {
    // spec: S-0105 Import mapping (FV, letters in either case)
    value::type_name_eq(name, FIXED_TYPE)
}

/// One pair as one rules layer gives it: the values in force for that
/// layer and where the keys stand (layer, line), `None` for a key the
/// layer leaves out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pair {
    /// Position of the stem in [`STEMS`].
    pub stem: usize,
    /// The turret index: the layer's value, else the stem's default.
    pub index: i32,
    /// The weapon number: the layer's value, else −1.
    pub weapon: i32,
    /// Where the layer gives the index key.
    pub index_at: Option<(usize, u32)>,
    /// Where the layer gives the weapon key.
    pub weapon_at: Option<(usize, u32)>,
}

impl Pair {
    /// A pair with no weapon number (absent or −1).
    pub fn no_weapon(&self) -> bool {
        // spec: S-0105 Import mapping (a weapon number of −1, or an absent weapon-number key)
        self.weapon == NO_WEAPON
    }

    /// Whether the pair sets a table entry: a weapon number 0 … 17.
    pub fn sets_entry(&self) -> bool {
        (0..TABLE_LEN as i32).contains(&self.weapon)
    }
}

/// The integer `layer` gives a key, with its line: the last value of that
/// layer that reads as an integer. A value that does not read, or a "not
/// given" value, keeps what the layer started from, here the pair's
/// default (S-0003 claims 3 and 8, S-0006 claim 3; the per-layer default of
/// S-0105 Import mapping).
fn in_layer(state: Option<&KeyState>, layer: usize) -> Option<(i32, (usize, u32))> {
    let mut got = None;
    for e in state.into_iter().flat_map(|s| &s.history) {
        if e.layer != layer {
            continue;
        }
        if let What::Set(v) = &e.what
            && let Some(n) = value::integer(v)
        {
            got = Some((n, (e.layer, e.line)));
        }
    }
    got
}

/// The 17 pairs as `layer` gives them, in Import mapping order.
pub fn pairs_in(t: &TypeRec, layer: usize) -> [Pair; 17] {
    std::array::from_fn(|stem| {
        let (name, default) = STEMS[stem];
        let index = in_layer(t.keys.get(format!("{name}TurretIndex").as_bytes()), layer);
        let weapon = in_layer(t.keys.get(format!("{name}TurretWeapon").as_bytes()), layer);
        Pair {
            stem,
            // spec: S-0105 Import mapping (in every layer that has the section, an absent index key uses its stem's default index, never an earlier layer's)
            index: index.map_or(default, |(n, _)| n),
            // spec: S-0105 Import mapping (weapon number absent: −1)
            weapon: weapon.map_or(NO_WEAPON, |(n, _)| n),
            index_at: index.map(|(_, at)| at),
            weapon_at: weapon.map(|(_, at)| at),
        }
    })
}

/// The layers that have the type's section, in order (S-0006 claim 7).
fn layers_of(t: &TypeRec) -> impl Iterator<Item = usize> + '_ {
    t.reads.iter().copied()
}

/// The turret table the type's own pairs give (S-0105 Import mapping,
/// *Turret table pairs*), whatever its name. Which types read it is
/// `quirks.turret_table_fixed_type_only` ([`table_for`]).
pub fn read_table(t: &TypeRec) -> TurretTable {
    let mut table = EMPTY;
    // spec: S-0105 Import mapping (the table is never cleared: an entry set in one layer that has the section stays unless a later such layer sets it)
    for layer in layers_of(t) {
        // spec: S-0105 Import mapping (pairs apply in the order listed, so a later pair naming the same weapon number wins)
        for p in pairs_in(t, layer) {
            if p.sets_entry() {
                // spec: S-0105 Import mapping (each pair sets the entry for its weapon number to its turret index; −1 means none; other indices are kept)
                table[p.weapon as usize] = (p.index != NONE_INDEX).then_some(p.index);
            }
            // spec: S-0105 Import mapping (a pair with no weapon number leaves the table unchanged; another weapon number outside 0 … 17 is ignored, design)
        }
    }
    table
}

/// The table vehicle type `t` has: its own pairs' table when it is the
/// fixed type name or `fixed_type_only` (the switch) is off, else all
/// *none*.
pub fn table_for(t: &TypeRec, fixed_type_only: bool) -> TurretTable {
    if fixed_type_only && !is_fixed_type(&t.name) {
        // spec: S-0105 claim 43 and Edge cases (quirks.turret_table_fixed_type_only: every other type keeps an all-none table)
        return EMPTY;
    }
    // spec: S-0105 Edge cases (quirks.turret_table_fixed_type_only off: every multi-turret type reads the same pairs from its own section, by the same rules)
    read_table(t)
}

/// The charge-turret flag of the fixed type name with
/// `quirks.turret_pair_no_weapon_sets_charge_flag` on: in each layer that
/// has the section, the layer's own charge-turret key applies as usual, and
/// then the last pair with no weapon number, if any, overrides it: on
/// exactly when its turret index is neither 0 nor a multiple of 256.
/// `None` when no layer gives the flag either way (the key's default).
pub fn charge_flag(t: &TypeRec) -> Option<bool> {
    let state = t.keys.get(b"IsChargeTurret".as_slice());
    let mut held = None;
    for layer in layers_of(t) {
        // The layer's own key, read as S-0006 claim 3 and S-0003 claim 2 say.
        for e in state.into_iter().flat_map(|s| &s.history) {
            if e.layer != layer {
                continue;
            }
            match &e.what {
                What::Set(v) => {
                    if let Some(b) = value::boolean(v) {
                        held = Some(b);
                    }
                }
                What::Reset => held = None,
                _ => {}
            }
        }
        // spec: S-0105 Edge cases (quirks.turret_pair_no_weapon_sets_charge_flag: such a pair overrides that layer's own charge-turret key; with several, the last one in Import mapping order decides)
        if let Some(p) = pairs_in(t, layer).iter().rev().find(|p| p.no_weapon()) {
            // spec: S-0105 Edge cases (on exactly when the turret index is not 0 and not a multiple of 256)
            held = Some(p.index.rem_euclid(256) != 0);
        }
    }
    held
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_keys_follow_the_stems() {
        for (i, (stem, _)) in STEMS.iter().enumerate() {
            assert_eq!(PAIR_KEYS[2 * i], format!("{stem}TurretIndex"));
            assert_eq!(PAIR_KEYS[2 * i + 1], format!("{stem}TurretWeapon"));
        }
    }
}
