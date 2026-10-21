//! Offline tests of the mixer on synthetic entries and samples (S-0701
//! claims 5 to 15, Formulas F2 to F4).

use super::*;
use crate::catalog::{EntryId, FULL, Priority, SoundEntry};
use glasnost_sim::rng::Rng;
use glasnost_sim::world::Pos;
use std::sync::Arc;

const RATE: u32 = 8000;

fn clip(frames: usize, v: i16) -> Arc<Clip> {
    Arc::new(Clip {
        sample_rate: RATE,
        channels: 1,
        samples: vec![v; frames],
    })
}

fn entry() -> SoundEntry {
    SoundEntry::new("TEST", &["a"])
}

fn prep_with(id: u32, e: SoundEntry, clips: Vec<Arc<Clip>>) -> Prepared {
    Prepared {
        id: EntryId(id),
        entry: Arc::new(e),
        clips,
    }
}

fn prep(id: u32, e: SoundEntry, n: usize) -> Prepared {
    prep_with(id, e, (0..n).map(|_| clip(RATE as usize, 1000)).collect())
}

fn mixer() -> Mixer {
    Mixer::new(RATE, 7, MixQuirks::default())
}

fn view(origin: (i32, i32)) -> View {
    View {
        origin,
        size: (800, 600),
        project: flat,
    }
}

fn at(x: i32, y: i32) -> Pos {
    Pos::new(x, y, 0)
}

/// Renders `frames` stereo frames.
fn render(m: &mut Mixer, frames: usize) -> Vec<i16> {
    let mut b = vec![0i16; frames * 2];
    m.render(&mut b);
    b
}

fn nonzero_left(b: &[i16]) -> usize {
    b.chunks(2).filter(|f| f[0] != 0).count()
}

// spec: S-0701 claim 5 (a screen sound: the entry's level, centre pan, wherever the view is)
#[test]
fn screen_sound_full_level_centre_pan() {
    let mut m = mixer();
    let id = m.request(prep(0, entry(), 1), Where::Screen, FULL).unwrap();
    assert_eq!(m.placement(id), Some((FULL, CENTRE)));
    m.set_view(view((100_000, 100_000)));
    assert!(m.service(0));
    assert!(m.is_playing(id));
}

// spec: S-0701 claim 6 and F2 (edge distance, vertical counted double, linear fade over 60 px per cell)
#[test]
fn positional_level_and_pan() {
    let v = view((0, 0));
    let e = entry(); // range 10: R = 600 px
    // Inside the view: full level; the right edge pans to 12288.
    assert_eq!(place(&e, &v, at(400, 300)), Some((FULL, CENTRE)));
    assert_eq!(place(&e, &v, at(800, 300)), Some((FULL, 12288)));
    // 300 px right of the view: half level; δ = 700 px, pan 700 × 8192 / 800 + 8192.
    assert_eq!(place(&e, &v, at(1100, 300)), Some((8192, 15360)));
    // 150 px below the view counts as 300: half level.
    assert_eq!(place(&e, &v, at(400, 750)).map(|p| p.0), Some(8192));
    // At the range: silent; 1 px inside it: L below 0.05, silent.
    assert_eq!(place(&e, &v, at(1400, 300)), None);
    assert_eq!(place(&e, &v, at(1399, 300)), None);
    // Just above 0.05 of the range (31 px inside it) is heard; exactly
    // 0.05 is stored toward zero in single precision and is not.
    assert!(place(&e, &v, at(1369, 300)).is_some());
    assert_eq!(place(&e, &v, at(1370, 300)), None);
}

// spec: S-0701 claim 6 and F2 (local: distance from the view centre; global: at least the minimum level)
#[test]
fn local_and_global_placement() {
    let v = view((0, 0));
    let mut local = entry();
    local.placement.local = true;
    assert_eq!(place(&local, &v, at(400, 300)).map(|p| p.0), Some(FULL));
    assert_eq!(place(&local, &v, at(700, 300)).map(|p| p.0), Some(8192));
    // Vertical doubled from the centre: 150 px counts as 300.
    assert_eq!(place(&local, &v, at(400, 450)).map(|p| p.0), Some(8192));
    let mut global = entry();
    global.placement.global = true;
    let floor = (f64::from(global.min_level) * f64::from(FULL)) as i32;
    assert_eq!(
        place(&global, &v, at(100_000, 300)).map(|p| p.0),
        Some(floor)
    );
}

// spec: S-0701 claim 6 (below 0.05 a sound is not started and takes none of claim 13's draws)
#[test]
fn out_of_range_takes_no_draws() {
    let mut m = mixer();
    m.set_view(view((0, 0)));
    let mut e = entry();
    e.fshift = (-10, 10);
    e.vshift = 20;
    let before = m.rng().clone();
    assert_eq!(
        m.request(prep(0, e, 1), Where::Fixed(at(5000, 300)), FULL),
        Err(Refused::OutOfRange)
    );
    m.service(0);
    assert_eq!(m.rng(), &before);
}

// spec: S-0701 claim 7 (direct requests keep their placement; follow-placed ones are placed again at passes)
#[test]
fn fixed_keeps_placement_follow_is_replaced() {
    let mut m = mixer();
    m.set_view(view((0, 0)));
    let fixed = m
        .request(prep(0, entry(), 1), Where::Fixed(at(1100, 300)), FULL)
        .unwrap();
    let follow = m
        .request(prep(1, entry(), 1), Where::Follow(at(1100, 300)), FULL)
        .unwrap();
    m.service(0);
    m.set_view(view((300, 0)));
    m.service(100);
    assert_eq!(m.placement(fixed), Some((8192, 15360)));
    assert_eq!(m.placement(follow).map(|p| p.0), Some(FULL));
    // Scrolled away: the follow-placed one-shot stops and is not restarted.
    m.set_view(view((100_000, 0)));
    m.service(200);
    assert!(!m.is_alive(follow));
    m.set_view(view((0, 0)));
    m.service(300);
    assert!(!m.is_alive(follow));
    assert!(m.is_playing(fixed));
}

// spec: S-0701 claim 7 (attached sounds are placed again when the anchor moves; an indefinite loop restarts when audible again)
#[test]
fn attached_loop_dormant_and_restart() {
    let mut m = mixer();
    m.set_view(view((0, 0)));
    let mut e = entry();
    e.control.looped = true;
    let id = m
        .request(
            prep(0, e, 1),
            Where::Attached {
                anchor: 5,
                pos: at(400, 300),
            },
            FULL,
        )
        .unwrap();
    m.service(0);
    assert!(m.is_playing(id));
    m.move_anchor(5, at(1100, 300));
    assert_eq!(m.placement(id).map(|p| p.0), Some(8192));
    m.move_anchor(5, at(100_000, 300));
    assert!(m.is_alive(id));
    assert!(!m.is_playing(id));
    assert_eq!(m.busy_channels(), 0);
    m.move_anchor(5, at(400, 300));
    m.service(100);
    assert!(m.is_playing(id));
}

// spec: S-0701 claim 8 (without interrupt a new request at about the same loudness is dropped)
#[test]
fn limit_drops_new_without_interrupt() {
    let mut m = mixer();
    let mut e = entry();
    e.limit = 1;
    let a = m
        .request(prep(0, e.clone(), 1), Where::Screen, FULL)
        .unwrap();
    m.service(0);
    // Within ⌊e / 10⌋ of each other: equally loud.
    let b = m.request(prep(0, e, 1), Where::Screen, FULL - 100).unwrap();
    m.service(100);
    assert!(m.is_playing(a));
    assert!(!m.is_alive(b));
}

// spec: S-0701 claim 8 (with interrupt the oldest is cut)
#[test]
fn limit_cuts_oldest_with_interrupt() {
    let mut m = mixer();
    let mut e = entry();
    e.limit = 1;
    e.control.interrupt = true;
    let a = m
        .request(prep(0, e.clone(), 1), Where::Screen, FULL)
        .unwrap();
    m.service(0);
    let b = m.request(prep(0, e, 1), Where::Screen, FULL).unwrap();
    m.service(100);
    assert!(!m.is_alive(a));
    assert!(m.is_playing(b));
}

// spec: S-0701 claim 8 (ranked loudest first: the quietest goes, new or not)
#[test]
fn limit_cuts_quietest() {
    let mut m = mixer();
    let mut e = entry();
    e.limit = 2;
    e.control.interrupt = true;
    let a = m
        .request(prep(0, e.clone(), 1), Where::Screen, FULL / 2)
        .unwrap();
    let b = m
        .request(prep(0, e.clone(), 1), Where::Screen, FULL)
        .unwrap();
    m.service(0);
    let c = m.request(prep(0, e, 1), Where::Screen, FULL).unwrap();
    m.service(100);
    assert!(!m.is_alive(a));
    assert!(m.is_playing(b));
    assert!(m.is_playing(c));
}

fn with_priority(p: Priority) -> SoundEntry {
    let mut e = entry();
    e.priority = p;
    e
}

/// Fills the 16 channels with normal-priority sounds of entries 0 … 15.
fn fill(m: &mut Mixer, quiet: Option<usize>, waiting: Option<usize>, low: Option<usize>) {
    for c in 0..CHANNELS {
        let mut e = with_priority(if low == Some(c) {
            Priority::Low
        } else {
            Priority::Normal
        });
        if waiting == Some(c) {
            e.control.predelay = true;
            e.delay = (5000, 5000);
        }
        let scale = if quiet == Some(c) { FULL / 2 } else { FULL };
        m.request(prep(c as u32, e, 1), Where::Screen, scale)
            .unwrap();
    }
    m.service(0);
    assert_eq!(m.busy_channels(), CHANNELS);
}

// spec: S-0701 claim 9 (a candidate of higher priority than the new instance: the new one is discarded, not retried)
#[test]
fn channels_lower_priority_discarded() {
    let mut m = mixer();
    fill(&mut m, None, None, None);
    let id = m
        .request(
            prep(99, with_priority(Priority::Low), 1),
            Where::Screen,
            FULL,
        )
        .unwrap();
    m.service(100);
    assert!(!m.is_alive(id));
    m.service(200);
    assert!(!m.channel_entries().contains(&Some(EntryId(99))));
}

// spec: S-0701 claim 9 (equal priority playing channels: the quietest is replaced; equal levels keep the first)
#[test]
fn channels_quietest_playing_replaced() {
    let mut m = mixer();
    fill(&mut m, Some(5), None, None);
    m.request(
        prep(99, with_priority(Priority::Normal), 1),
        Where::Screen,
        FULL,
    )
    .unwrap();
    m.service(100);
    assert_eq!(m.channel_entries()[5], Some(EntryId(99)));
    let mut m = mixer();
    fill(&mut m, None, None, None);
    m.request(
        prep(99, with_priority(Priority::High), 1),
        Where::Screen,
        FULL,
    )
    .unwrap();
    m.service(100);
    assert_eq!(m.channel_entries()[0], Some(EntryId(99)));
}

// spec: S-0701 claim 9 (the playing candidate only if its priority is strictly lower than the non-playing one's)
#[test]
fn channels_waiting_before_playing() {
    let mut m = mixer();
    fill(&mut m, Some(5), Some(3), None);
    m.request(
        prep(99, with_priority(Priority::Normal), 1),
        Where::Screen,
        FULL,
    )
    .unwrap();
    m.service(100);
    assert_eq!(m.channel_entries()[3], Some(EntryId(99)));
    let mut m = mixer();
    fill(&mut m, None, Some(3), Some(5));
    m.request(
        prep(99, with_priority(Priority::Normal), 1),
        Where::Screen,
        FULL,
    )
    .unwrap();
    m.service(100);
    assert_eq!(m.channel_entries()[5], Some(EntryId(99)));
}

// spec: S-0701 claim 10 (at most 300 instances; 200 follow-placed ones)
#[test]
fn instance_pool_limits() {
    let mut m = mixer();
    for k in 0..POOL {
        m.request(prep(k as u32, entry(), 1), Where::Screen, FULL)
            .unwrap();
    }
    assert_eq!(
        m.request(prep(999, entry(), 1), Where::Screen, FULL),
        Err(Refused::PoolFull)
    );
    let mut m = mixer();
    m.set_view(view((0, 0)));
    for k in 0..FOLLOW_POOL {
        m.request(
            prep(k as u32, entry(), 1),
            Where::Follow(at(400, 300)),
            FULL,
        )
        .unwrap();
    }
    assert_eq!(
        m.request(prep(999, entry(), 1), Where::Follow(at(400, 300)), FULL),
        Err(Refused::PoolFull)
    );
    assert!(
        m.request(prep(999, entry(), 1), Where::Fixed(at(400, 300)), FULL)
            .is_ok()
    );
}

// spec: S-0701 claim 12 (a pass only when more than 33 ms have elapsed; channel allocation waits for a pass)
#[test]
fn passes_on_the_audio_clock() {
    let mut m = mixer();
    assert!(m.service(1000));
    assert!(!m.service(1033));
    let id = m.request(prep(0, entry(), 1), Where::Screen, FULL).unwrap();
    assert!(!m.service(1033));
    assert!(!m.is_playing(id));
    assert!(m.service(1034));
    assert!(m.is_playing(id));
}

// spec: S-0701 claim 13 (pitch, volume shift, then pre-delay, in this order; a pre-delay of 33 ms or more is waited)
#[test]
fn instance_draws_in_order() {
    let mut m = mixer();
    let mut e = entry();
    e.fshift = (-10, 10);
    e.vshift = 20;
    e.control.predelay = true;
    e.delay = (40, 80);
    let mut r = m.rng().clone();
    let id = m.request(prep(0, e, 1), Where::Screen, FULL).unwrap();
    m.service(1000);
    let pitch = 100 + r.ranged(-10, 10, true);
    let shift = r.ranged(0, 20, true);
    let pre = r.ranged(40, 80, true);
    // Delayed mode, one sample: no further draws.
    assert_eq!(m.rng(), &r);
    assert_eq!(m.draws(id), Some((pitch, shift)));
    assert!(!m.is_playing(id));
    m.service(1000 + i64::from(pre) - 1);
    assert!(!m.is_playing(id));
    m.service(1000 + i64::from(pre));
    assert!(m.is_playing(id));
}

// spec: S-0701 claim 13 (no pitch shift and no volume shift: no raw value; ambient: a = 33; a short pre-delay is no wait)
#[test]
fn instance_draws_ambient_and_none() {
    let mut m = mixer();
    let before = m.rng().taken();
    let plain = m.request(prep(0, entry(), 1), Where::Screen, FULL).unwrap();
    m.service(0);
    assert_eq!(m.rng().taken(), before);
    assert!(m.is_playing(plain));
    let mut e = entry();
    e.control.ambient = true;
    e.delay = (0, 50);
    let mut r = m.rng().clone();
    let id = m.request(prep(1, e, 1), Where::Screen, FULL).unwrap();
    m.service(100);
    let pre = r.ranged(33, 50, true);
    assert!(pre >= 33);
    assert_eq!(m.rng(), &r);
    assert!(!m.is_playing(id));
    let mut e = entry();
    e.control.predelay = true;
    e.delay = (10, 20);
    let short = m.request(prep(2, e, 1), Where::Screen, FULL).unwrap();
    m.service(200);
    assert!(m.is_playing(short));
}

// spec: S-0701 claim 14 (immediate mode: attack R(0, A − 1), body R(A, n − D − 1) with random, decay R(n − D, n − 1))
#[test]
fn sample_order_attack_body_decay() {
    let mut m = mixer();
    let mut e = entry();
    e.attack = 2;
    e.decay = 2;
    e.control.attack = true;
    e.control.decay = true;
    e.control.random = true;
    let mut r = m.rng().clone();
    let id = m.request(prep(0, e, 6), Where::Screen, FULL).unwrap();
    m.service(0);
    let a = r.ranged(0, 1, true) as usize;
    let b = r.ranged(2, 3, true) as usize;
    let d = r.ranged(4, 5, true) as usize;
    assert_eq!(m.rng(), &r);
    // The attack sample is playing; the body and decay follow.
    assert_eq!(m.sample_order(id), Some(vec![b, d]));
    assert!(a <= 1);
}

fn shuffled(r: &mut Rng, mut left: Vec<usize>) -> Vec<usize> {
    let mut out = Vec::new();
    while !left.is_empty() {
        let k = r.ranged(0, left.len() as i32 - 1, true) as usize;
        out.push(left.remove(k));
    }
    out
}

// spec: S-0701 claim 14 and Edge cases (all and random: without repetition; quirks.sound_order_startup_draw: one discarded R(0, m − 1) first)
#[test]
fn sample_order_all_random_with_startup_draw() {
    for quirk in [true, false] {
        let mut m = Mixer::new(
            RATE,
            7,
            MixQuirks {
                sound_order_startup_draw: quirk,
                ranged_draw_half_span: true,
            },
        );
        let mut e = entry();
        e.control.all = true;
        e.control.random = true;
        let mut r = m.rng().clone();
        let id = m.request(prep(0, e, 4), Where::Screen, FULL).unwrap();
        m.service(0);
        if quirk {
            let _ = r.ranged(0, 3, true);
        }
        let order = shuffled(&mut r, vec![0, 1, 2, 3]);
        assert_eq!(m.rng(), &r, "quirk {quirk}");
        // The first is playing.
        assert_eq!(m.sample_order(id), Some(order[1..].to_vec()));
    }
}

// spec: S-0701 claim 14 (all without random: every body sample in list order; otherwise the first body sample)
#[test]
fn sample_order_all_in_list_order() {
    let mut m = mixer();
    let mut e = entry();
    e.control.all = true;
    let id = m.request(prep(0, e, 3), Where::Screen, FULL).unwrap();
    m.service(0);
    assert_eq!(m.sample_order(id), Some(vec![1, 2]));
    let one = m.request(prep(1, entry(), 3), Where::Screen, FULL).unwrap();
    m.service(100);
    assert_eq!(m.sample_order(one), Some(vec![]));
}

// spec: S-0701 claim 14 (with loop: that many passes in total, or forever at loop count 0); claim 23 (finish looping)
#[test]
fn loops_counted_and_forever() {
    let mut m = mixer();
    let mut e = entry();
    e.control.all = true;
    e.control.looped = true;
    e.loop_count = 2;
    let clips = vec![clip(100, 1000), clip(100, 1000)];
    let id = m
        .request(prep_with(0, e.clone(), clips.clone()), Where::Screen, FULL)
        .unwrap();
    m.service(0);
    let b = render(&mut m, 1000);
    assert_eq!(nonzero_left(&b), 400);
    assert!(!m.is_alive(id));
    e.loop_count = 0;
    m.set_view(view((0, 0)));
    let id = m
        .request(
            prep_with(1, e, clips),
            Where::Attached {
                anchor: 1,
                pos: at(0, 0),
            },
            FULL,
        )
        .unwrap();
    m.service(100);
    assert_eq!(nonzero_left(&render(&mut m, 1000)), 1000);
    assert!(m.is_alive(id));
    // Finishing ends the current body pass, not the current sample.
    m.finish_anchor(1);
    let b = render(&mut m, 1000);
    assert!(nonzero_left(&b) > 0 && nonzero_left(&b) <= 200);
    assert!(!m.is_alive(id));
}

// spec: S-0701 F3 (σ = 16384 − ⌊s × 16384 / 100⌋; the truncation chain)
#[test]
fn heard_level_chain() {
    assert_eq!(heard(FULL, FULL, 0, FULL, FULL), FULL);
    assert_eq!(heard(FULL, FULL / 2, 0, FULL, FULL), FULL / 2);
    // s = 50: σ = 8192.
    assert_eq!(heard(FULL, FULL, 50, FULL, FULL), 8192);
    // ⌊⌊13107 × 8192 / 16384⌋ × 16383 / 16384⌋ = ⌊6553 × 16383 / 16384⌋ = 6552.
    assert_eq!(heard(13107, 8192, 0, 16383, FULL), 6552);
}

// spec: S-0701 F4 (T(k) = round(1000 × log₂(k / 100)), T(0) = −10000; halving costs 10 dB)
#[test]
fn attenuation_law() {
    assert_eq!(attenuation(100), 0);
    assert_eq!(attenuation(50), -1000);
    assert_eq!(attenuation(25), -2000);
    assert_eq!(attenuation(0), -10000);
    assert_eq!(amplitude(0), 65536);
    assert_eq!(amplitude(-10000), 0);
    assert_eq!(amplitude(-1000), 20724);
}

// spec: S-0701 claim 15 and F4 (the far side attenuated on the same law: pan 12288 is 10 dB quieter on the left)
#[test]
fn balance_law() {
    assert_eq!(gains(FULL, CENTRE), (65536, 65536));
    assert_eq!(gains(FULL, 12288), (20724, 65536));
    assert_eq!(gains(FULL, 4096), (65536, 20724));
    assert_eq!(gains(8192, CENTRE), (20724, 20724));
    assert_eq!(gains(0, CENTRE), (0, 0));
}

// spec: S-0701 claim 15 (rendered: level, pan and pitch of what is heard)
#[test]
fn rendered_level_pan_and_pitch() {
    let mut m = mixer();
    m.set_view(view((0, 0)));
    let mut e = entry();
    e.level = FULL;
    m.request(
        prep_with(0, e.clone(), vec![clip(100, 10000)]),
        Where::Screen,
        FULL,
    )
    .unwrap();
    m.service(0);
    let b = render(&mut m, 10);
    assert_eq!((b[0], b[1]), (10000, 10000));
    let mut m = mixer();
    m.set_view(view((0, 0)));
    m.request(
        prep_with(0, e.clone(), vec![clip(100, 10000)]),
        Where::Fixed(at(800, 300)),
        FULL,
    )
    .unwrap();
    m.service(0);
    let b = render(&mut m, 10);
    assert_eq!((b[0], b[1]), (3162, 10000));
    // Pitch 200: the sample plays twice as fast.
    let mut m = mixer();
    e.fshift = (100, 100);
    m.request(prep_with(0, e, vec![clip(100, 10000)]), Where::Screen, FULL)
        .unwrap();
    m.service(0);
    assert_eq!(nonzero_left(&render(&mut m, 200)), 50);
}

// spec: S-0701 F3 (C = Σ ⌊20 × e / 16384⌋; master 16384 when C ≤ 100, else ⌊1638400 / C⌋)
#[test]
fn master_level() {
    let mut m = mixer();
    let mut e = entry();
    e.level = FULL;
    for k in 0..5 {
        m.request(prep(k, e.clone(), 1), Where::Screen, FULL)
            .unwrap();
    }
    m.service(0);
    assert_eq!(m.master(), FULL);
    m.request(prep(5, e, 1), Where::Screen, FULL).unwrap();
    m.service(100);
    assert_eq!(m.master(), 1_638_400 / 120);
}

// spec: S-0701 claim 27 (announcements play separate from the 16 channels)
#[test]
fn announcement_voice_is_separate() {
    let mut m = mixer();
    fill(&mut m, None, None, None);
    m.speak(clip(100, 1000));
    assert!(m.speaking());
    render(&mut m, 200);
    assert!(!m.speaking());
    assert_eq!(m.busy_channels(), CHANNELS);
}

// spec: S-0701 claim 35 (the voice option is at full by default); the person's voice volume setting (M8 C2) scales it and zero silences it
#[test]
fn voice_volume_option_scales_announcements() {
    let level = |option: Option<i32>| {
        let mut m = mixer();
        if let Some(o) = option {
            m.set_voice_volume(o);
        }
        m.speak(clip(100, 1000));
        render(&mut m, 4)[0]
    };
    let full = level(None);
    assert_eq!(full, level(Some(FULL)));
    assert_eq!(level(Some(FULL / 2)), full / 2);
    assert_eq!(level(Some(0)), 0);
}
