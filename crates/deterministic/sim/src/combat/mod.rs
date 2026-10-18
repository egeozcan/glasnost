//! Combat (milestone M4): the firing check and cycle (S-0301), damage
//! resolution (S-0302), area damage (S-0303), versus bands (S-0304), armour
//! classes (S-0305), projectiles (S-0306), the range test (S-0307), weapon
//! choice (S-0308), target acquisition (S-0309), aiming (S-0310),
//! veterancy (S-0311), death and destruction (S-0312), firing positions
//! (S-0313), healing (S-0314) and fear (S-0315).
//!
//! Features whose own topics are not written yet (garrisons, bunkers,
//! transports, cloaking, berserk and other special warheads, spawned
//! aircraft, aircraft flight, crates, bridges, smudges, particles and the
//! playback of animations) are seams: the state that would switch their
//! rows on is never set, and each rule that reads such a state keeps its
//! place so that the rows land where the specs put them once the topic is
//! written.

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
pub mod lof;
pub mod position;
pub mod projectile;
pub mod railgun;
pub mod range;
pub mod state;
pub mod veterancy;
pub mod weapon;
