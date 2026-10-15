//! Seam: paradrop delivery after the launch (S-0522 claims 6 to 15). The
//! launch side, which sends the planes and loads their paratroopers
//! (S-0522 claims 1 to 3 and 16), is `superweapons::planes`.

use crate::world::{ObjectId, World};

/// The paradrop plane's processing of *paradrop approach* and *paradrop
/// overfly* (S-0522 claims 6 to 11): approach, passes, drop attempts and
/// the paratroopers' fall. Not built: returns `None`, so the plane keeps
/// the no-rule delay and holds its paratroopers. It never leaves, because
/// S-0501 claim 10 keeps a plane in either paradrop behaviour on the map.
pub fn drop_passengers(w: &mut World, plane: ObjectId) -> Option<i32> {
    let _ = plane;
    w.note("paradrop delivery (S-0522 claims 6 to 15) is not built: the planes hold their paratroopers");
    None
}
