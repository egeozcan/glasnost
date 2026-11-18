//! The trace under the pointer (#442): what the drawing shows at a world
//! pixel, as text for the log and a local file. Our own diagnostics: no
//! spec applies, nothing here reaches the simulation, and the text names
//! types by their rules names only in the local file and the log.

use glasnost_map::CellXY;
use glasnost_render::scene::{Item, Scene};
use glasnost_sim::world::ObjectId;

use super::{Live, LiveAnim, Player, SimKind, type_name};

/// How far, in cells either way, an animation or a building counts as
/// near the pointer's cell.
// design: #442 (our own diagnostics radius)
pub const NEAR: i32 = 2;

/// Where the pointer is, for [`Live::trace_report`].
#[derive(Clone, Copy, Debug)]
pub struct TraceAt {
    /// The pointer's screen pixel.
    pub screen: (i32, i32),
    /// The world pixel under it.
    pub world: (i32, i32),
    /// The view's top world row (for a tiled animation's copies).
    pub top: i32,
    /// The cell under it.
    pub cell: CellXY,
}

/// Whether `c` is within [`NEAR`] cells of `p` either way.
fn near(c: CellXY, p: CellXY) -> bool {
    (c.x - p.x).abs().max((c.y - p.y).abs()) <= NEAR
}

/// A playback's state and loop settings (S-0109's fields, as the drawing
/// holds them).
fn play_text(p: &Player) -> String {
    let q = &p.play;
    let next = q.next.as_deref().map_or_else(
        || "none".to_owned(),
        |n| String::from_utf8_lossy(n).into_owned(),
    );
    format!(
        "frame {} stage {}; start {} end {} loop start {} loop end {} loop count {}; \
         rate (stage delay) {} random rate {:?}; reverse {} ping-pong {}; random loop delay {:?}; \
         next {next}; passes left {:?}; ended {}",
        p.frame(),
        p.stage(),
        q.start,
        q.end,
        q.loop_start,
        q.loop_end,
        q.loop_count,
        q.delay,
        q.random_delay,
        q.reverse,
        q.ping_pong,
        q.random_loop_delay,
        p.remaining_passes(),
        p.ended(),
    )
}

impl Live {
    /// An object, for the trace: id, kind, type name, owner, cell and
    /// position.
    fn object_text(&self, id: ObjectId) -> String {
        match self.world.obj(id) {
            Some(o) => format!(
                "object #{} {:?} {} owner {:?} cell ({}, {}) pos ({}, {}, {}) on map {}",
                o.id.0,
                o.kind,
                type_name(&self.rules, o.kind, o.type_index),
                o.owner.map(|h| h.0),
                o.pos.cell().x,
                o.pos.cell().y,
                o.pos.x,
                o.pos.y,
                o.pos.z,
                o.on_map
            ),
            None => format!("object #{} (no longer in the game)", id.0),
        }
    }

    /// The playing animation that drew `it` in the last redraw, as text, if
    /// one did.
    pub(super) fn anim_item_text(&self, it: &Item) -> Option<String> {
        let names: Vec<String> = self
            .anim_keys
            .iter()
            .filter(|(image, x, y, _)| (*image, *x, *y) == (it.image, it.x, it.y))
            .filter_map(|(_, _, _, serial)| self.anims.iter().find(|a| a.serial == *serial))
            .map(|a| format!("animation #{} {}", a.serial, a.player.name_text()))
            .collect();
        (!names.is_empty()).then(|| names.join(", "))
    }

    /// What a playing animation is attached to.
    fn anim_owner(&self, a: &LiveAnim) -> String {
        if let Some(o) = a.owner {
            format!("attached to {} (a parachute)", self.object_text(o))
        } else if let Some((o, _)) = a.shooter {
            format!("attached to its firer {}", self.object_text(o))
        } else if let Some(d) = a.drainer {
            format!("attached to its drainer {}", self.object_text(d))
        } else {
            "none".to_owned()
        }
    }

    /// What made a playing animation, as far as the drawing knows.
    fn anim_cause(&self, a: &LiveAnim) -> String {
        let mut parts: Vec<String> = Vec::new();
        if !a.cause.is_empty() {
            parts.push(a.cause.to_owned());
        }
        if let Some(id) = a.sim {
            parts.push(match self.world.anims.anims.get(&id) {
                Some(s) => format!(
                    "shows the simulation's animation #{} (role {:?}, warning marker {:?}, made through a loop {})",
                    id.0, s.role, s.warning, s.looped
                ),
                None => format!(
                    "showed the simulation's animation #{} (no longer in play)",
                    id.0
                ),
            });
        }
        if let Some(id) = a.walker {
            parts.push(format!("shows the burning walker #{}", id.0));
        }
        if let Some(id) = a.maker {
            parts.push(format!("shows the infantry-making animation #{}", id.0));
        }
        if let Some(id) = a.dominating {
            parts.push(format!("shows the domination animation #{}", id.0));
        }
        if self.source_marker == Some(a.serial) {
            parts.push("the viewer's teleport source marker".to_owned());
        }
        if parts.is_empty() {
            "not known to the drawing".to_owned()
        } else {
            parts.join("; ")
        }
    }

    /// The buildings on the map with a foundation cell near `cell`.
    fn buildings_near(&self, cell: CellXY) -> Vec<ObjectId> {
        self.world
            .objects
            .values()
            .filter(|o| o.kind == SimKind::Building && o.on_map)
            .filter(|o| {
                glasnost_sim::econ::placement::foundation_cells(
                    &self.world,
                    o.type_index,
                    o.pos.cell(),
                )
                .into_iter()
                .any(|c| near(c, cell))
            })
            .map(|o| o.id)
            .collect()
    }

    /// The trace under the pointer (#442): the tick and the pointer's
    /// place; every drawn item covering the pointer's world pixel, top
    /// first, naming the animation of an animation item; the drawing's and
    /// the simulation's animations at or near the pointer's cell (within
    /// [`NEAR`] cells) with their owners, places, frames, stages, loop
    /// settings, creation ticks and makers; the slot animations of the
    /// buildings near it; and the objects in the pointer's cell. Our own
    /// diagnostics: nothing reads it, and it changes nothing.
    pub fn trace_report(
        &self,
        scene: &Scene,
        statics: &[super::super::build::ObjectInfo],
        at: TraceAt,
    ) -> Vec<String> {
        let mut out = Vec::new();
        out.push(format!(
            "trace under the pointer at tick {}: screen pixel ({}, {}), world pixel ({}, {}), cell ({}, {})",
            self.world.tick,
            at.screen.0,
            at.screen.1,
            at.world.0,
            at.world.1,
            at.cell.x,
            at.cell.y
        ));
        let mut items = self.trace_point(scene, statics, at.world.0, at.world.1, at.top);
        items.reverse();
        out.push(format!(
            "drawn items at the pointer, top first: {}",
            items.len()
        ));
        out.extend(items.into_iter().map(|l| format!("  {l}")));
        // The playing animations drawn at the pointer, or placed near it.
        let drawn_here: Vec<u32> = scene
            .items_at(at.world.0, at.world.1, at.top)
            .into_iter()
            .flat_map(|i| {
                let it = &scene.items[i];
                self.anim_keys
                    .iter()
                    .filter(move |(image, x, y, _)| (*image, *x, *y) == (it.image, it.x, it.y))
                    .map(|k| k.3)
            })
            .collect();
        let anims: Vec<&LiveAnim> = self
            .anims
            .iter()
            .filter(|a| drawn_here.contains(&a.serial) || near(a.pos.cell(), at.cell))
            .collect();
        out.push(format!(
            "the drawing's animations at or near the pointer (within {NEAR} cells): {}",
            anims.len()
        ));
        for a in anims {
            let cell = a.pos.cell();
            out.push(format!(
                "  animation #{} {}: owner {}; pos ({}, {}, {}) cell ({}, {}); layer {:?}; {}; first update tick {}, created in tick {}; made by {}; {}",
                a.serial,
                a.player.name_text(),
                self.anim_owner(a),
                a.pos.x,
                a.pos.y,
                a.pos.z,
                cell.x,
                cell.y,
                a.layer,
                play_text(&a.player),
                a.first_update,
                a.created,
                self.anim_cause(a),
                if drawn_here.contains(&a.serial) {
                    "drawn at the pointer"
                } else if a.entry.as_ref().is_some_and(|e| e.sprite) {
                    "has art"
                } else {
                    "has no art (not drawn)"
                },
            ));
        }
        let shown: Vec<ObjectId> = self.anims.iter().filter_map(|a| a.sim).collect();
        let sims: Vec<_> = self
            .world
            .anims
            .anims
            .iter()
            .filter(|(_, s)| {
                let pos = s.pos.or_else(|| {
                    let (o, d) = s.owner?;
                    let o = self.world.obj(o)?;
                    Some(glasnost_sim::world::Pos::new(
                        o.pos.x + d.x,
                        o.pos.y + d.y,
                        o.pos.z + d.z,
                    ))
                });
                pos.is_some_and(|p| near(p.cell(), at.cell))
            })
            .collect();
        out.push(format!(
            "the simulation's animations at or near the pointer: {}",
            sims.len()
        ));
        for (id, s) in sims {
            out.push(format!(
                "  simulation animation #{} {}: owner {}; pos {:?}; role {:?}; warning marker {:?}; made through a loop {}; {}; {} (the simulation keeps no creation tick)",
                id.0,
                s.player.name_text(),
                s.owner
                    .map_or_else(|| "none".to_owned(), |(o, _)| self.object_text(o)),
                s.pos.map(|p| (p.x, p.y, p.z)),
                s.role,
                s.warning,
                s.looped,
                play_text(&s.player),
                if shown.contains(id) {
                    "shown by the drawing"
                } else {
                    "not shown by the drawing"
                },
            ));
        }
        let buildings = self.buildings_near(at.cell);
        let mut slots = Vec::new();
        for &b in &buildings {
            for (slot, r) in self.slot_anims.get(&b).into_iter().flat_map(|s| &s.running) {
                slots.push(format!(
                    "  building {} slot {slot:?}: {}; {}; created in tick {}; frozen {}; has art {}",
                    self.object_text(b),
                    String::from_utf8_lossy(&r.name),
                    play_text(&r.anim.player),
                    r.anim.created,
                    r.frozen,
                    r.anim.entry.sprite,
                ));
            }
            for d in self
                .world
                .depot_anims
                .anims
                .values()
                .filter(|d| d.depot == b)
            {
                slots.push(format!(
                    "  building {} slot {:?} (the simulation's): {}; {} (the simulation keeps no creation tick)",
                    self.object_text(b),
                    d.slot,
                    d.player.name_text(),
                    play_text(&d.player),
                ));
            }
        }
        out.push(format!(
            "slot animations of the buildings at or near the pointer: {}",
            slots.len()
        ));
        out.extend(slots);
        let objects: Vec<ObjectId> = self
            .world
            .objects
            .values()
            .filter(|o| {
                o.pos.cell() == at.cell
                    || (o.kind == SimKind::Building
                        && o.on_map
                        && glasnost_sim::econ::placement::foundation_cells(
                            &self.world,
                            o.type_index,
                            o.pos.cell(),
                        )
                        .contains(&at.cell))
            })
            .map(|o| o.id)
            .collect();
        out.push(format!("objects in the pointer's cell: {}", objects.len()));
        out.extend(
            objects
                .into_iter()
                .map(|id| format!("  {}", self.object_text(id))),
        );
        out
    }
}
