//! Combat (milestone M4): the firing check and cycle (S-0301), damage
//! resolution (S-0302), area damage (S-0303), versus bands (S-0304), armour
//! classes (S-0305), projectiles (S-0306), the range test (S-0307), weapon
//! choice (S-0308), target acquisition (S-0309), aiming (S-0310),
//! veterancy (S-0311), death and destruction (S-0312), firing positions
//! (S-0313), healing (S-0314) and fear (S-0315).
//!
//! The rules here call the modules of the features they meet (garrisons,
//! bunkers, transports, cloaking, psychic and other special warheads,
//! spawned aircraft, aircraft flight, crates, bridges and particles) and
//! read their state. Where a row of a combat claim is still missing, the
//! comment at its site says that the row is not implemented, and names the
//! state it would read; it is a seam only where that state is not modelled
//! at all (rubble and smudges, for example).

pub mod acquire;
pub mod area;
pub mod ballistics;
pub mod behave;
pub mod check;
pub mod damage;
pub mod data;
pub mod death;
pub mod effects;
pub mod fear;
pub mod fire;
pub mod flying;
pub mod heal;
pub mod held;
pub mod lof;
pub mod position;
pub mod projectile;
pub mod railgun;
pub mod range;
pub mod shotanim;
pub mod shrapnel;
pub mod state;
pub mod survivors;
pub mod veterancy;
pub mod weapon;
