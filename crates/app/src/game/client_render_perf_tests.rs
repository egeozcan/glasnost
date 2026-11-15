//! Guards of the render perf batch RP3's changes to the game's drawing,
//! through the windowless client on synthetic data (invented rules and
//! map, `glasnost_ai::fixture`). The specs cited are the ones the drawing
//! implements; the rest is our own design, named so where it is tested.

use std::sync::Arc;

use super::access_tests::{client, own, session, session_mut, start_by_keyboard_with};
use super::*;
use glasnost_ai::fixture;
use glasnost_map::CellXY;
use glasnost_sim::mind::Victim;
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::ObjectId;
use glasnost_ui::draw::Prim;

/// The fixture's rules with invented control and permanent control ring
/// animation types, so that `glasnost_sim::mind::ring` gives rings.
fn ring_rules() -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    r.sim.general.controlled_animation_type = Some("TESTRING".into());
    r.sim.general.perma_controlled_animation_type = Some("TESTPERMA".into());
    Arc::new(SimRules::new(&r, "synthetic-skirmish"))
}

/// The stand-in ring markers' inks (our own, `session.rs`).
const RING: [u8; 4] = [200, 80, 230, 255];
const PERMA: [u8; 4] = [255, 120, 255, 255];

/// The inks of the ring-marker lines in `d`, in drawing order.
fn ring_inks(d: &DrawList) -> Vec<[u8; 4]> {
    let mut out = Vec::new();
    for p in &d.prims {
        match p {
            Prim::Line { rgba, .. } if *rgba == RING || *rgba == PERMA => out.push(*rgba),
            Prim::Layer { list, .. } => out.extend(ring_inks(list)),
            _ => {}
        }
    }
    out
}

/// spec: S-0601 claim 18 (the control ring on a controlled object, the
/// permanent ring on a permanently controlled one) and design: render
/// (#122) (the rings are found from the victims and the permanently
/// controlled objects, not from every object): each ring marker in object
/// order, the permanent one where an object is both, none for an object
/// off the map or a record whose object is gone.
#[test]
fn render_perf_122_the_rings_in_object_order() {
    let mut c = client((800, 600), 100);
    start_by_keyboard_with(&mut c, ring_rules());
    let mcv = own(&c, Kind::Vehicle)[0];
    let s = session_mut(&mut c);
    let me = s.play.me();
    let at = s.play.world().obj(mcv).unwrap().pos;
    glasnost_sim::econ::shroud::area_reveal(&mut s.play.game.world, me, at, 8);
    let base = at.cell();
    let w = &mut s.play.game.world;
    let units: Vec<ObjectId> = (1..=4)
        .map(|dx| {
            let cell = CellXY::new(base.x + dx, base.y + 1);
            testkit::spawn(w, Kind::Vehicle, "MINR", me.0, cell, 0)
        })
        .collect();
    let [a, b, both, off] = units[..] else {
        panic!("four units");
    };
    assert!(a < b && b < both, "spawned in id order");
    let victim = Victim {
        controller: mcv,
        original_owner: None,
        line_until: 0,
    };
    for id in [both, a, off, ObjectId(999_999)] {
        w.mind.victims.insert(id, victim);
    }
    w.mind.permanent.insert(both);
    w.mind.permanent.insert(b);
    w.obj_mut(off).unwrap().on_map = false;
    let d = s.draw(DrawList::new());
    let mut want = vec![RING; 8];
    want.extend([PERMA; 16]);
    assert_eq!(ring_inks(&d), want);
}

/// An invented shroud sprite: piece 15 (full cover) dark, every other
/// piece clear, over the whole canvas.
fn shroud_sprite() -> crate::mapview::cover::CoverSprites {
    use glasnost_fmt::image::IndexedImage;
    use glasnost_render::shroud::{
        CANVAS_HEIGHT, CANVAS_WIDTH, CoverFrame, CoverSprite, FULL_COVER, PIECES,
    };
    let (w, h) = (CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32);
    let frames = (0..PIECES)
        .map(|i| {
            let v = if i == usize::from(FULL_COVER) { 0 } else { 254 };
            Some(CoverFrame {
                image: IndexedImage::new(w, h, vec![v; (w * h) as usize]).unwrap(),
                x: 0,
                y: 0,
            })
        })
        .collect();
    crate::mapview::cover::CoverSprites {
        shroud: Some(CoverSprite { frames }),
        fog: None,
    }
}

/// A frame of `c` drawn through `draw` offscreen, RGBA.
fn gpu_frame(
    device: &glasnost_render::wgpu::Device,
    queue: &glasnost_render::wgpu::Queue,
    draw: &mut crate::game::GpuDraw,
    c: &mut Client,
) -> Vec<u8> {
    let (w, h) = (c.size.0 as u32, c.size.1 as u32);
    // The batch's palettes and atlas go to every draw's device.
    c.painter.dirty = true;
    let mut encoder = device.create_command_encoder(&Default::default());
    draw.prepare(device, queue, c, (w, h), &mut encoder);
    queue.submit([encoder.finish()]);
    glasnost_render::offscreen::render_rgba(device, queue, w, h, |pass| draw.draw(pass))
}

/// A lit item (S-0110 claims 5 and 6: its row follows the light alpha)
/// over twice the view around it, of three rows of plain colours, so that
/// every light-alpha change shows on the battlefield.
fn draw_lit_ground(s: &mut Session) {
    use glasnost_render::compose::{DepthTest, Paint, Profile, Shade};
    use glasnost_render::scene::{Group, Item, ItemDepth};
    let v = s.view();
    let (w, h) = (v.size.0 * 2, v.size.1 * 2);
    let image = s.scene_mut().add_image(
        glasnost_fmt::image::IndexedImage::new(w, h, vec![1; (w * h) as usize]).unwrap(),
    );
    let base = s.scene_mut().add_row([0x001F; 256]);
    assert_eq!(s.scene_mut().add_row([0x07E0; 256]), base + 1);
    assert_eq!(s.scene_mut().add_row([0xF800; 256]), base + 2);
    s.scene_mut().items.push(Item {
        image,
        x: v.origin.0 - v.size.0 as i32 / 2,
        y: v.origin.1 - v.size.1 as i32 / 2,
        test: DepthTest::Ignore,
        depth: ItemDepth::Profile {
            profile: Profile::Flat,
            bias: 0,
            companion: None,
        },
        paint: Paint::Lit(Shade::new(base, 3, 1000)),
        group: Group::Vehicles,
        provisional: false,
        object: None,
        stack: None,
    });
}

/// design: render (#119) (the light alpha kept between frames): a draw
/// that keeps its light alpha from frame to frame shows what a new draw
/// shows, after the shroud closes and opens again, after the view
/// scrolls, and on a frame where nothing changed.
#[test]
fn render_perf_119_a_kept_light_alpha_follows_the_shroud_and_the_view() {
    use glasnost_render::offscreen::OFFSCREEN_FORMAT;
    let Ok(g) = glasnost_render::gpu::shared_device() else {
        eprintln!("skipping: no GPU adapter");
        return;
    };
    let (device, queue) = (g.device, g.queue);
    let size = (800, 600);
    let mut c = client(size, 100);
    start_by_keyboard_with(&mut c, fixture::rules());
    let mcv = own(&c, Kind::Vehicle)[0];
    let s = session_mut(&mut c);
    s.loaded.covers = shroud_sprite();
    draw_lit_ground(s);
    let mut kept = crate::game::GpuDraw::new(&device, OFFSCREEN_FORMAT);
    // The battlefield's pixels only: the client hands its changed images
    // (the radar's) to one draw alone.
    let m = session(&c).map_rect();
    let field = move |rgba: &[u8]| -> Vec<u8> {
        let w = size.0;
        (m.y..m.y + m.h)
            .flat_map(|y| {
                let at = ((y * w + m.x) * 4) as usize;
                rgba[at..at + (m.w * 4) as usize].to_vec()
            })
            .collect()
    };
    let mut check = |c: &mut Client, what: &str| {
        let k = field(&gpu_frame(&device, &queue, &mut kept, c));
        let mut fresh = crate::game::GpuDraw::new(&device, OFFSCREEN_FORMAT);
        let f = field(&gpu_frame(&device, &queue, &mut fresh, c));
        let first = k.chunks(4).zip(f.chunks(4)).position(|(a, b)| a != b);
        assert!(first.is_none(), "{what}: pixel {first:?} differs");
        k
    };
    // Nothing is explored before the first tick: the view is all dark.
    let before = check(&mut c, "the first frame");
    let s = session_mut(&mut c);
    let me = s.play.me();
    let at = s.play.world().obj(mcv).unwrap().pos;
    glasnost_sim::econ::shroud::area_reveal(&mut s.play.game.world, me, at, 4);
    let opened = check(&mut c, "the shroud opened");
    assert!(before != opened, "the opened shroud shows");
    assert!(opened == check(&mut c, "nothing changed"));
    // A scroll draws the scene again, without the item.
    session_mut(&mut c).scroll((90, 45));
    let _ = c.frame();
    draw_lit_ground(session_mut(&mut c));
    let scrolled = check(&mut c, "the view scrolled");
    let s = session_mut(&mut c);
    glasnost_sim::econ::shroud::crate_darkness(&mut s.play.game.world, me);
    let closed = check(&mut c, "the shroud closed");
    assert!(closed != scrolled, "the closed shroud shows");
}
