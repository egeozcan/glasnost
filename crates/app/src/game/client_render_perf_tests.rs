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

/// The stand-in ring markers' inks (our own, `session.rs` up to render
/// batch R19, which drew the rings' art in their place).
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

/// spec: S-0601 claim 18 (the ring is the rules' ring animation type, played by the live view):
/// the game's drawing has no stand-in marker for a ring any more, for a controlled or a
/// permanently controlled object, and drawing changes no game state. (The fixture's import has no
/// art for the invented ring types, so nothing is drawn for them.)
#[test]
fn claim_18_no_stand_in_marker_is_drawn_for_a_ring() {
    use glasnost_sim::save::state_hash;
    let mut c = client((800, 600), 100);
    start_by_keyboard_with(&mut c, ring_rules());
    let mcv = own(&c, Kind::Vehicle)[0];
    let s = session_mut(&mut c);
    let me = s.play.me();
    let at = s.play.world().obj(mcv).unwrap().pos;
    glasnost_sim::econ::shroud::area_reveal(&mut s.play.game.world, me, at, 8);
    let base = at.cell();
    let w = &mut s.play.game.world;
    let units: Vec<ObjectId> = (1..=2)
        .map(|dx| {
            let cell = CellXY::new(base.x + dx, base.y + 1);
            testkit::spawn(w, Kind::Vehicle, "MINR", me.0, cell, 0)
        })
        .collect();
    w.mind.victims.insert(
        units[0],
        Victim {
            controller: mcv,
            original_owner: None,
            line_until: 0,
        },
    );
    w.mind.permanent.insert(units[1]);
    assert!(glasnost_sim::mind::ring(w, units[0]).is_some());
    assert!(glasnost_sim::mind::ring(w, units[1]).is_some());
    let before = state_hash(w);
    let d = s.draw(DrawList::new());
    assert_eq!(ring_inks(&d), Vec::<[u8; 4]>::new(), "no marker");
    assert_eq!(state_hash(&session(&c).play.game.world), before);
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

/// design: render (#118) (the kept light alpha moved with a scrolled view,
/// only the uncovered strips made again): a draw that keeps its light alpha
/// shows what a new draw shows after scrolls in both axes and both
/// directions, a jump past the whole view, and a scroll in the same frame as
/// the shroud opening inside the part of the view both frames show, or
/// closing everywhere.
/// spec: S-0110 claims 5 to 7 and 10 (the light-alpha values of the view after its covers)
#[test]
fn render_perf_118_b_scrolled_light_alpha_follows_the_shroud_and_the_view() {
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
    let me = s.play.me();
    let at = s.play.world().obj(mcv).unwrap().pos;
    // An explored patch whose edges cross the view, so that every scroll
    // moves light-alpha changes.
    glasnost_sim::econ::shroud::area_reveal(&mut s.play.game.world, me, at, 3);
    draw_lit_ground(s);
    let mut kept = crate::game::GpuDraw::new(&device, OFFSCREEN_FORMAT);
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
    // A scroll draws the scene again, without the item: it is put back.
    let scroll = |c: &mut Client, d: (i32, i32)| -> bool {
        let before = session(c).view().origin;
        session_mut(c).scroll(d);
        let _ = c.frame();
        draw_lit_ground(session_mut(c));
        session(c).view().origin != before
    };
    let mut last = check(&mut c, "the first frame");
    let mut moved = 0;
    for d in [
        (30, 0),
        (-47, 0),
        (0, 15),
        (0, -31),
        (61, -17),
        (-13, 29),
        (7, 7),
        (-120, -90),
    ] {
        moved += usize::from(scroll(&mut c, d));
        last = check(&mut c, &format!("a scroll by {d:?}"));
    }
    assert!(moved >= 6, "only {moved} scrolls moved the view");
    // A scroll and the shroud opening inside both frames' part of the view.
    let o = session(&c).view().origin;
    assert!(scroll(&mut c, (23, 11)), "the view scrolls");
    let s = session_mut(&mut c);
    let open = glasnost_sim::world::Pos {
        x: at.x + 5 * 256,
        y: at.y - 2 * 256,
        z: at.z,
    };
    glasnost_sim::econ::shroud::area_reveal(&mut s.play.game.world, me, open, 3);
    let opened = check(&mut c, "a scroll as the shroud opens");
    assert!(opened != last, "the opened shroud shows");
    assert_ne!(session(&c).view().origin, o);
    // A jump from one corner of the map to the other, past the whole view:
    // no part is shown by both frames.
    scroll(&mut c, (-5000, -5000));
    let _ = check(&mut c, "a corner");
    let o = session(&c).view().origin;
    scroll(&mut c, (5000, 5000));
    let jumped = session(&c).view().origin;
    let _ = check(&mut c, "a jump");
    let (w, h) = session(&c).view().size;
    assert!(
        (jumped.0 - o.0).unsigned_abs() >= w || (jumped.1 - o.1).unsigned_abs() >= h,
        "the jump from {o:?} to {jumped:?} leaves part of the view shown"
    );
    // Back, as the shroud closes everywhere.
    scroll(&mut c, (-5000, -5000));
    let s = session_mut(&mut c);
    glasnost_sim::econ::shroud::crate_darkness(&mut s.play.game.world, me);
    let closed = check(&mut c, "a scroll as the shroud closes");
    assert!(closed != opened, "the closed shroud shows");
}

/// design: render (#138) (the frames of ticks no frame sees are put off to
/// the next frame or pick; the battlefield a frame keeps holds only while
/// the scene's stamp stands, #112): after ticks whose frames were put off,
/// the next frame shows the redrawn scene, through a draw that keeps its
/// battlefield as through a new draw; a pick draws the owed frame first.
#[test]
fn render_perf_138_a_frame_after_put_off_ticks_shows_them() {
    use glasnost_render::offscreen::OFFSCREEN_FORMAT;
    let Ok(g) = glasnost_render::gpu::shared_device() else {
        eprintln!("skipping: no GPU adapter");
        return;
    };
    let (device, queue) = (g.device, g.queue);
    let size = (800, 600);
    let mut c = client(size, 100);
    start_by_keyboard_with(&mut c, fixture::rules());
    // A drawing no redraw makes: the redraw after a tick takes it away.
    draw_lit_ground(session_mut(&mut c));
    let mut kept = crate::game::GpuDraw::new(&device, OFFSCREEN_FORMAT);
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
    let first = field(&gpu_frame(&device, &queue, &mut kept, &mut c));
    let s = session_mut(&mut c);
    s.put_off_frames = true;
    for _ in 0..3 {
        s.step();
    }
    s.put_off_frames = false;
    let owed = s.frame_owed();
    let stamp = s.scene_stamp();
    let k = field(&gpu_frame(&device, &queue, &mut kept, &mut c));
    if owed {
        assert_ne!(
            session(&c).scene_stamp(),
            stamp,
            "the owed frame moves the stamp on"
        );
    }
    assert!(!session(&c).frame_owed(), "the frame drew the owed frame");
    let mut fresh = crate::game::GpuDraw::new(&device, OFFSCREEN_FORMAT);
    let f = field(&gpu_frame(&device, &queue, &mut fresh, &mut c));
    let differs = k.chunks(4).zip(f.chunks(4)).position(|(a, b)| a != b);
    assert!(differs.is_none(), "pixel {differs:?} differs");
    assert!(k != first, "the ticks' redraw shows");
    // A pick after ticks whose frames were put off reads the redrawn scene.
    let s = session_mut(&mut c);
    s.put_off_frames = true;
    s.step();
    s.put_off_frames = false;
    let p = glasnost_ui::geom::Point::new(m.x + m.w / 2, m.y + m.h / 2);
    let _ = s.under(p);
    assert!(!s.frame_owed(), "the pick drew the owed frame");
}

/// design: render (#149) (the session's live drawing keeps no copy of the
/// start world: the game's world is lent to it while it draws, and in
/// between it holds an empty world in its place): after the start and after
/// ticks the live drawing's own world holds no object, no cell and no
/// shroud; while it draws it sees the game's world, which comes back whole.
#[test]
fn render_perf_149_the_live_drawing_keeps_no_start_world() {
    use glasnost_sim::save::state_hash;
    let mut c = client((800, 600), 100);
    start_by_keyboard_with(&mut c, fixture::rules());
    let held = |c: &Client| {
        let live = session(c).loaded.live.as_ref().expect("a live drawing");
        let w = &live.world;
        (
            w.objects.len(),
            w.cells.grid.len(),
            w.econ.shroud.houses.len(),
        )
    };
    assert_eq!(held(&c), (0, 0, 0), "no copy of the start world is kept");
    for _ in 0..4 {
        session_mut(&mut c).step();
        let _ = c.frame();
    }
    assert_eq!(held(&c), (0, 0, 0), "nor after ticks and frames");
    let s = session_mut(&mut c);
    let game = state_hash(&s.play.game.world);
    let objects = s.play.game.world.objects.len();
    let seen = s
        .with_live(|live, _, _| (live.world.objects.len(), state_hash(&live.world)))
        .expect("a live drawing");
    assert!(objects > 0, "the game has objects");
    assert_eq!(
        seen,
        (objects, game.clone()),
        "the drawing sees the game's world"
    );
    assert_eq!(
        state_hash(&session(&c).play.game.world),
        game,
        "the game's world comes back"
    );
    assert_eq!(held(&c), (0, 0, 0), "and the drawing keeps none");
}
