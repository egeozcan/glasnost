//! Objects a map places (S-0009 claims 27 and 34 to 43), and the integer
//! rules of their starting state that need no rules data: starting health
//! (claim 37), starting facing (claim 39), upgrade fields carried over
//! (claim 40) and follower-car links (claim 41).
//!
//! The importer splits each entry as claim 34 says and reads it by the field
//! layout of claim 35. Every entry whose owner and type fields are present
//! is kept in entry order, short entries included (claim 36), because which
//! entries survive the owner and type match, and so which draws they take,
//! is decided where the houses and types are known.

use serde::{Deserialize, Serialize};

use crate::geometry::CellXY;

/// The fields every mobile or building entry shares.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Placed {
    /// The entry's key in its section (kept for object info and warnings;
    /// it has no effect, claim 34).
    pub key: Vec<u8>,
    /// Owner house name, matched case-sensitively by the caller (claim 27).
    pub owner: Vec<u8>,
    /// Type name, compared without regard to letter case (claim 36).
    pub type_name: Vec<u8>,
    /// Health value (claim 37: 256 is full strength); 0 for a short entry.
    pub health: i32,
    /// Cell after the coordinate wrap (claim 38; Formulas, object cell). For
    /// a building, the foundation's top-left cell.
    pub cell: CellXY,
    /// Facing value (claim 39): field 6 for vehicles, aircraft and
    /// buildings, field 8 for infantry (0 when absent).
    pub facing: i32,
    /// The entry's fields after the split of claim 34, for object info.
    pub raw: Vec<Vec<u8>>,
}

/// A vehicle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    /// Shared fields.
    pub placed: Placed,
}

/// An aircraft.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aircraft {
    /// Shared fields.
    pub placed: Placed,
}

/// An infantry unit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Infantry {
    /// Shared fields.
    pub placed: Placed,
    /// Standing position number, field 6 (claim 35; S-0208 claim 5).
    pub sub_cell: i32,
}

/// A building.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Structure {
    /// Shared fields; the cell is the foundation's top-left cell (claim 38).
    pub placed: Placed,
}

/// A terrain object (tree and similar), claim 42.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainObject {
    /// Cell.
    pub cell: CellXY,
    /// Type name, compared without regard to letter case.
    pub type_name: Vec<u8>,
}

/// A smudge entry that places something (claim 42: piece number absent or
/// 0, both cell fields present).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Smudge {
    /// Type name, compared without regard to letter case.
    pub type_name: Vec<u8>,
    /// The origin cell; the smudge covers its type's width × height cells
    /// from it.
    pub cell: CellXY,
}

/// The optional fields of a vehicle, aircraft, infantry or building entry
/// (claim 35), each `None` when the entry does not give it, and whether the
/// entry lacks a required field (claim 36).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryFields {
    /// The entry lacks a required field (claim 36): the original does not
    /// survive it; our engine drops it (a departure) after the draws the
    /// claim gives.
    pub short: bool,
    /// Behaviour name (vehicles, aircraft, infantry; claim 41).
    pub behaviour: Option<Vec<u8>>,
    /// Tag name (claim 41).
    pub tag: Option<Vec<u8>>,
    /// Veterancy value (claim 41; S-0311 claim 11).
    pub veterancy: Option<i32>,
    /// Group number (vehicles, aircraft, infantry; claim 41).
    pub group: Option<i32>,
    /// Bridge flag (vehicles, infantry; claim 38).
    pub bridge: Option<bool>,
    /// Follow index (vehicles; claim 41).
    pub follow: Option<i32>,
    /// Recruit-no flag (claim 41).
    pub recruit_no: Option<bool>,
    /// Recruit-yes flag (claim 41).
    pub recruit_yes: Option<bool>,
    /// Computer-sell flag (buildings; claim 40).
    pub ai_sell: Option<bool>,
    /// Computer-rebuild flag (buildings; claim 40).
    pub ai_rebuild: Option<bool>,
    /// Switched-on flag (buildings; claim 40, default set).
    pub switched_on: Option<bool>,
    /// Upgrade count (buildings; claim 40, default 0).
    pub upgrade_count: Option<i32>,
    /// Searchlight value (buildings; claim 40, default 1).
    pub searchlight: Option<i32>,
    /// The three upgrade name fields as given (buildings; claim 40).
    pub upgrades: [Option<Vec<u8>>; 3], // spec: S-0009 Constants (upgrade name fields: 3)
    /// Computer-repair flag (buildings; claim 40).
    pub ai_repair: Option<bool>,
    /// Nominal flag (buildings; claim 40).
    pub nominal: Option<bool>,
}

impl EntryFields {
    /// The switched-on flag with its default (claim 40: set).
    pub fn switched_on(&self) -> bool {
        // spec: S-0009 Import mapping ([Structures] switched-on flag absent: set)
        self.switched_on.unwrap_or(true)
    }
}

/// Every object a map places, in entry order per list.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Objects {
    /// `[Units]`.
    pub units: Vec<Unit>,
    /// `[Infantry]`.
    pub infantry: Vec<Infantry>,
    /// `[Aircraft]`.
    pub aircraft: Vec<Aircraft>,
    /// `[Structures]`.
    pub structures: Vec<Structure>,
    /// `[Terrain]`.
    pub terrain: Vec<TerrainObject>,
    /// `[Smudge]`.
    pub smudges: Vec<Smudge>,
    /// The optional fields of `units`, by index; an index without an entry
    /// here has every optional field absent.
    #[serde(default)]
    pub unit_fields: Vec<EntryFields>,
    /// The optional fields of `infantry`, by index.
    #[serde(default)]
    pub infantry_fields: Vec<EntryFields>,
    /// The optional fields of `aircraft`, by index.
    #[serde(default)]
    pub aircraft_fields: Vec<EntryFields>,
    /// The optional fields of `structures`, by index.
    #[serde(default)]
    pub structure_fields: Vec<EntryFields>,
}

/// The four object lists that hold owned objects (claim 27).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ListKind {
    /// `[Units]`.
    Vehicles,
    /// `[Aircraft]`.
    Aircraft,
    /// `[Infantry]`.
    Infantry,
    /// `[Structures]`.
    Buildings,
}

impl ListKind {
    /// The kinds in placement order (claim 43: vehicles, aircraft,
    /// infantry, buildings; terrain objects before and smudges after them).
    // spec: S-0009 claim 43 (placement order)
    pub const ORDER: [ListKind; 4] = [
        ListKind::Vehicles,
        ListKind::Aircraft,
        ListKind::Infantry,
        ListKind::Buildings,
    ];
}

/// Every optional field absent (S-0009 Import mapping: the absent defaults apply).
// nocite: an empty record of our own, no behaviour
static NO_FIELDS: EntryFields = EntryFields {
    short: false,
    behaviour: None,
    tag: None,
    veterancy: None,
    group: None,
    bridge: None,
    follow: None,
    recruit_no: None,
    recruit_yes: None,
    ai_sell: None,
    ai_rebuild: None,
    switched_on: None,
    upgrade_count: None,
    searchlight: None,
    upgrades: [None, None, None],
    ai_repair: None,
    nominal: None,
};

impl Objects {
    /// Total number of objects.
    pub fn len(&self) -> usize {
        self.units.len()
            + self.infantry.len()
            + self.aircraft.len()
            + self.structures.len()
            + self.terrain.len()
            + self.smudges.len()
    }

    /// Whether the map places no object.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The entries of one owned-object list, in entry order.
    pub fn list(&self, kind: ListKind) -> Vec<&Placed> {
        match kind {
            ListKind::Vehicles => self.units.iter().map(|u| &u.placed).collect(),
            ListKind::Aircraft => self.aircraft.iter().map(|u| &u.placed).collect(),
            ListKind::Infantry => self.infantry.iter().map(|u| &u.placed).collect(),
            ListKind::Buildings => self.structures.iter().map(|u| &u.placed).collect(),
        }
    }

    /// The optional fields of entry `i` of a list (every field absent when
    /// none were recorded).
    pub fn fields(&self, kind: ListKind, i: usize) -> &EntryFields {
        let v = match kind {
            ListKind::Vehicles => &self.unit_fields,
            ListKind::Aircraft => &self.aircraft_fields,
            ListKind::Infantry => &self.infantry_fields,
            ListKind::Buildings => &self.structure_fields,
        };
        v.get(i).unwrap_or(&NO_FIELDS)
    }
}

/// Facing units per unit of the facing value (claim 39).
pub const FACING_VALUE_SCALE: i32 = 256; // spec: S-0009 Constants (facing value scale)

/// The starting body facing of a pre-placed object with facing value `v`:
/// 256 × (v mod 256) facing units, mod the non-negative remainder (claim 39,
/// Formulas "Facing of a pre-placed object").
pub fn facing_from_value(v: i32) -> u16 {
    // spec: S-0009 claim 39 (256 × (v mod 256), non-negative remainder)
    (v.rem_euclid(FACING_VALUE_SCALE) * FACING_VALUE_SCALE) as u16
}

/// The health value of full strength (claim 37).
pub const FULL_HEALTH_VALUE: i32 = 256; // spec: S-0009 Constants (health value of full strength)

/// The near-full margin in hit points (claim 37).
pub const NEAR_FULL_MARGIN: i32 = 3; // spec: S-0009 Constants (near-full margin)

/// Significant binary digits of the product in the original's number
/// domain (S-0009 Formulas "Object health": RZ₅₃).
const PRODUCT_DIGITS: u32 = 53; // spec: S-0009 Formulas (object health: RZ₅₃, S-0205 claim 5)

/// The kinds whose low end of starting health differs (claim 37).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthKind {
    /// Infantry: a result below 1 starts with 1.
    Infantry,
    /// Vehicles and aircraft: 0 starts with 1; below 0 is unspecified.
    VehicleOrAircraft,
    /// Buildings: the value is first limited to 256; 0 or below is
    /// unspecified.
    Building,
}

/// Rounds an exact product to 53 significant binary digits, toward zero with
/// `chop` (the round-toward-zero mode, S-0205 claim 5), otherwise to the
/// nearest with ties to even.
fn round_product(p: i64, chop: bool) -> i64 {
    let a = p.unsigned_abs();
    let bits = u64::BITS - a.leading_zeros();
    if bits <= PRODUCT_DIGITS {
        return p;
    }
    let shift = bits - PRODUCT_DIGITS;
    let mut m = a >> shift;
    if !chop {
        let rem = a & ((1u64 << shift) - 1);
        let half = 1u64 << (shift - 1);
        if rem > half || (rem == half && m & 1 == 1) {
            m += 1;
        }
    }
    let r = (m << shift) as i64;
    if p < 0 { -r } else { r }
}

/// The starting health of a pre-placed object of strength `strength` with
/// health value `value` (claim 37, Formulas "Object health"). Returns the
/// health and whether the original's result is unspecified (our engine then
/// gives 1 and the importer warns, a departure). `near_full_rounds_up` is
/// `quirks.near_full_health_rounds_up`; `chop` is S-0205's
/// `quirks.chop_rounding`.
pub fn starting_health(
    kind: HealthKind,
    value: i32,
    strength: i32,
    near_full_rounds_up: bool,
    chop: bool,
) -> (i32, bool) {
    // spec: S-0009 claim 37 (a building's value above 256 is first replaced by 256)
    let h = if kind == HealthKind::Building {
        value.min(FULL_HEALTH_VALUE)
    } else {
        value
    };
    // spec: S-0009 Formulas (h₀ = w₃₂(trunc(RZ₅₃(H × S) / 256)))
    let product = round_product(i64::from(h) * i64::from(strength), chop);
    let h0 = (product / i64::from(FULL_HEALTH_VALUE)) as i32;
    let h = if near_full_rounds_up {
        // spec: S-0009 claim 37 (raised to S whenever above S − 3; quirks.near_full_health_rounds_up)
        if h0 > strength.wrapping_sub(NEAR_FULL_MARGIN) {
            strength
        } else {
            h0
        }
    } else {
        // spec: S-0009 Formulas (without the switch, h = min(h₀, S))
        h0.min(strength)
    };
    match kind {
        // spec: S-0009 claim 37 (infantry below 1 starts with 1)
        HealthKind::Infantry => (h.max(1), false),
        // spec: S-0009 claim 37 (vehicle or aircraft: 0 gives 1; negative unspecified, our engine 1)
        HealthKind::VehicleOrAircraft => match h {
            0 => (1, false),
            h if h < 0 => (1, true),
            h => (h, false),
        },
        // spec: S-0009 claim 37 (building: the result as it is; 0 or negative unspecified, our engine 1)
        HealthKind::Building => {
            if h <= 0 {
                (1, true)
            } else {
                (h, false)
            }
        }
    }
}

/// Upgrade name fields per building entry (claim 40).
pub const UPGRADE_FIELDS: usize = 3; // spec: S-0009 Constants (upgrade name fields)

/// The upgrade names each building entry stands for after the carry-over
/// of claim 40. `entries` gives, in entry order, whether the entry matched
/// an owner and a building type (an entry dropped for its owner or type
/// gives no field) and its three upgrade name fields as given.
/// `carry_over` is `quirks.upgrade_fields_carry_over`. Entries that did not
/// match get no names.
pub fn resolve_upgrades(
    entries: &[(bool, [Option<Vec<u8>>; UPGRADE_FIELDS])],
    carry_over: bool,
) -> Vec<[Option<Vec<u8>>; UPGRADE_FIELDS]> {
    let mut held: [Option<Vec<u8>>; UPGRADE_FIELDS] = [None, None, None];
    let mut out = Vec::with_capacity(entries.len());
    for (matched, fields) in entries {
        if !matched {
            // spec: S-0009 claim 40 (an entry dropped for its owner or type gives no field)
            out.push([None, None, None]);
            continue;
        }
        let mut names: [Option<Vec<u8>>; UPGRADE_FIELDS] = [None, None, None];
        for (k, f) in fields.iter().enumerate() {
            match f {
                Some(n) => {
                    held[k] = Some(n.clone());
                    names[k] = Some(n.clone());
                }
                // spec: S-0009 claim 40 (an absent field keeps the most recent earlier entry's; quirks.upgrade_fields_carry_over)
                None if carry_over => names[k] = held[k].clone(),
                None => {}
            }
        }
        out.push(names);
    }
    out
}

/// One vehicle entry for the follower-car links of claim 41.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FollowEntry {
    /// The follow-index field, when the entry gives one.
    pub value: Option<i32>,
    /// Whether the vehicle was placed on the map.
    pub placed: bool,
}

/// The follower-car links of claim 41.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FollowLinks {
    /// For each placed vehicle, numbered from 0 in entry order: the number
    /// of its leader among the placed vehicles.
    pub leaders: Vec<Option<usize>>,
    /// Placed vehicles numbered at or beyond the number of follow values
    /// (unspecified in the original; no leader, the importer warns).
    pub unspecified: Vec<usize>,
    /// Some entries give the field and others do not (warned).
    pub mixed: bool,
    /// Placed vehicles that are their own leader (warned).
    pub own_leader: Vec<usize>,
    /// Placed vehicles that lie on a loop of links (warned).
    pub in_loop: Vec<usize>,
}

/// Works out which placed vehicle follows which (claim 41). `entries` are
/// the vehicle entries that matched an owner and a vehicle type, in entry
/// order. `by_place` is `quirks.follow_index_by_place`.
pub fn follow_links(entries: &[FollowEntry], by_place: bool) -> FollowLinks {
    let values: Vec<i32> = entries.iter().filter_map(|e| e.value).collect();
    let placed: Vec<&FollowEntry> = entries.iter().filter(|e| e.placed).collect();
    let v = placed.len();
    let mut out = FollowLinks {
        mixed: !values.is_empty() && values.len() != entries.len(),
        ..Default::default()
    };
    for (i, e) in placed.iter().enumerate() {
        let n = if by_place {
            // spec: S-0009 claim 41 (the placed vehicle numbered i takes the follow value numbered i)
            match values.get(i) {
                Some(&n) => Some(n),
                None => {
                    // spec: S-0009 claim 41 (beyond the follow values: unspecified; our engine no leader)
                    out.unspecified.push(i);
                    None
                }
            }
        } else {
            // spec: S-0009 Edge cases (quirks.follow_index_by_place off: each vehicle takes its own entry's value)
            e.value
        };
        // spec: S-0009 claim 41 (0 ≤ n < V names the leader; any other value gives no leader)
        let leader = n.and_then(|n| usize::try_from(n).ok()).filter(|&n| n < v);
        if leader == Some(i) {
            out.own_leader.push(i);
        }
        out.leaders.push(leader);
    }
    // A loop: following leaders from a vehicle comes back to it (length ≥ 2).
    for start in 0..v {
        let mut at = out.leaders[start];
        let mut steps = 0;
        while let Some(n) = at {
            if n == start {
                if steps > 0 {
                    out.in_loop.push(start);
                }
                break;
            }
            steps += 1;
            if steps > v {
                break;
            }
            at = out.leaders[n];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0009 claim 39 (facing 256 × (v mod 256))
    #[test]
    fn claim_39_facing() {
        assert_eq!(facing_from_value(0), 0);
        assert_eq!(facing_from_value(64), 16384);
        assert_eq!(facing_from_value(255), 65280);
        assert_eq!(facing_from_value(256), 0);
        assert_eq!(facing_from_value(-1), 65280);
        assert_eq!(facing_from_value(300), 44 * 256);
    }

    /// spec: S-0009 claim 37 (scale, truncation, near-full rule, low ends, wrap)
    #[test]
    fn claim_37_health() {
        let v = HealthKind::VehicleOrAircraft;
        // Formulas examples.
        assert_eq!(starting_health(v, 128, 300, true, true), (150, false));
        assert_eq!(starting_health(v, 255, 300, true, true), (300, false));
        assert_eq!(starting_health(v, 254, 100, true, true), (100, false));
        assert_eq!(
            starting_health(v, 1099511756, 1000, true, true),
            (500, false)
        );
        assert_eq!(
            starting_health(HealthKind::Building, 1099511756, 1000, true, true),
            (1000, false)
        );
        // Without the switch: never more than S, no raising.
        assert_eq!(starting_health(v, 254, 100, false, true), (99, false));
        assert_eq!(starting_health(v, 400, 100, false, true), (100, false));
        // Low ends.
        assert_eq!(
            starting_health(HealthKind::Infantry, 0, 100, true, true),
            (1, false)
        );
        assert_eq!(
            starting_health(HealthKind::Infantry, -50, 100, true, true),
            (1, false)
        );
        assert_eq!(starting_health(v, 1, 100, true, true), (1, false));
        assert_eq!(starting_health(v, -10, 100, true, true), (1, true));
        assert_eq!(
            starting_health(HealthKind::Building, 0, 100, true, true),
            (1, true)
        );
        assert_eq!(
            starting_health(HealthKind::Building, 1, 100, true, true),
            (1, true)
        );
        assert_eq!(
            starting_health(HealthKind::Building, 3, 100, true, true),
            (1, false)
        );
    }

    /// spec: S-0009 Formulas (object health: RZ₅₃ changes the product only from 2⁵³ on)
    #[test]
    fn object_health_rounding() {
        let p = (1i64 << 60) + 1;
        assert_eq!(round_product(p, true), 1i64 << 60);
        assert_eq!(round_product(-p, true), -(1i64 << 60));
        let q = (1i64 << 54) + 3;
        assert_eq!(round_product(q, false), (1i64 << 54) + 4);
        assert_eq!(round_product(1 << 40, true), 1 << 40);
    }

    /// spec: S-0009 claim 40 (upgrade name fields carry over; quirks.upgrade_fields_carry_over)
    #[test]
    fn claim_40_upgrade_carry_over() {
        let n = |s: &str| Some(s.as_bytes().to_vec());
        let entries = vec![
            (true, [n("A"), None, None]),
            (false, [n("X"), n("Y"), None]),
            (true, [None, n("B"), None]),
        ];
        let r = resolve_upgrades(&entries, true);
        assert_eq!(r[0], [n("A"), None, None]);
        assert_eq!(r[1], [None, None, None]);
        assert_eq!(r[2], [n("A"), n("B"), None]);
        let r = resolve_upgrades(&entries, false);
        assert_eq!(r[2], [None, n("B"), None]);
    }

    /// spec: S-0009 claim 41 (follow values move forward onto earlier vehicles; quirks.follow_index_by_place)
    #[test]
    fn claim_41_follow_links() {
        let e = |value, placed| FollowEntry { value, placed };
        // The claim's example: no field, n = 2, n = −1.
        let entries = [e(None, true), e(Some(2), true), e(Some(-1), true)];
        let l = follow_links(&entries, true);
        assert_eq!(l.leaders, vec![Some(2), None, None]);
        assert_eq!(l.unspecified, vec![2]);
        assert!(l.mixed);
        // Switch off: each vehicle takes its own entry's value.
        let l = follow_links(&entries, false);
        assert_eq!(l.leaders, vec![None, Some(2), None]);
        // An unplaced vehicle's value still counts; n names placed vehicles.
        let entries = [e(Some(1), false), e(Some(0), true), e(Some(0), true)];
        let l = follow_links(&entries, true);
        assert_eq!(l.leaders, vec![Some(1), Some(0)]);
        assert_eq!(l.in_loop, vec![0, 1]);
        // Own leader.
        let l = follow_links(&[e(Some(0), true)], true);
        assert_eq!(l.own_leader, vec![0]);
    }
}
