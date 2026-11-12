//! The performance benchmark (M8 step C4): a late game of our own AI on
//! an install map, watched from fixed views at a fixed size, with every
//! frame's cost written out.
//!
//! The bench starts a skirmish of AI houses only on the install's
//! `map`-th offered map, runs `late` ticks straight on the game (no
//! drawing), brings the drawing up to it, and then times frames: for each
//! view in turn, `warmup` frames that are recorded but left out of the
//! numbers, then `frames` timed ones. The views are picked from the game
//! as each starts: **base** (the watched house's densest cluster of
//! buildings), **battle** (the mobile unit with the most units of other
//! player houses in view, and its own), **open** (the view with the fewest
//! objects on a grid of candidate centres) and **pan** (a sweep across the
//! map's width through its middle, there and back).
//!
//! Ticks follow the game's own pacer against a virtual clock of a steady
//! 60 frames a second (`tick=paced`, the default), so the same frames tick
//! on every run and machine; `tick=every` runs a tick every frame. The
//! frame's drawing depends only on that virtual clock, so two builds draw
//! the same pixels for the same frame: the offscreen host writes one
//! picture per view to compare.
//!
//! Each frame row holds the host-clock times of its parts (the client's
//! update with the tick and its parts, the draw list, the scene uploads,
//! the light alpha, the map's draw recording, the batch, the surface's
//! acquire, the encoding, the submit and present) and the renderer's counts
//! (instances, draw calls, passes, backdrop copies, uploads and bytes), and
//! the GPU time when the device has timestamp queries. The summary gives
//! mean, median, p95, p99, worst and the frames over 16.7 ms per view; on
//! the web, whose totals are whole display refresh intervals, the frames
//! that missed a refresh instead (#91).
//!
//! Hosts: [`run_offscreen`] (native, no window), the game window with
//! `--bench` (native) or `?bench=` (web). Our own tooling: nothing here is
//! game behaviour, so no spec applies; the numbers are ours.

use std::fmt::Write as _;

use glasnost_audio::Audio;
use glasnost_audio::output::{DEFAULT_RATE, NullOutput};
use glasnost_audio::speech::SpeechCatalog;
use glasnost_render::projection::project_lifted;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseRole, World};
use glasnost_ui::setup::{MapEntry, Setup, SlotKind};
use glasnost_ui::shell::Screen;

use super::client::{Client, Job};
use super::session::Session;
use super::window::{DrawLaps, DrawStats};

/// The frame time a 60 Hz display allows, milliseconds.
// nocite: our own tooling (one frame of a 60 Hz display)
pub const BUDGET_MS: f64 = 1000.0 / 60.0;

/// The host clock, milliseconds since the program started (the benchmark's
/// times only; never game state).
pub fn now() -> f64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::OnceLock;
        static START: OnceLock<std::time::Instant> = OnceLock::new();
        START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_secs_f64()
            * 1000.0
    }
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.performance())
            .map_or(0.0, |p| p.now())
    }
}

/// A fixed view of the benchmark.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewKind {
    /// The watched house's densest cluster of buildings.
    Base,
    /// The mobile unit with the most units of other player houses in view.
    Battle,
    /// The candidate view with the fewest objects.
    Open,
    /// A sweep across the map's width through its middle, there and back.
    Pan,
}

impl ViewKind {
    /// The view's name in the options and outputs.
    pub fn name(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Battle => "battle",
            Self::Open => "open",
            Self::Pan => "pan",
        }
    }

    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "base" => Ok(Self::Base),
            "battle" => Ok(Self::Battle),
            "open" => Ok(Self::Open),
            "pan" => Ok(Self::Pan),
            other => Err(format!(
                "unknown view {other:?}; expected base, battle, open or pan"
            )),
        }
    }
}

/// The benchmark's settings, from a spec such as
/// `map=2,late=18000,frames=600` (`default` or empty: all defaults).
#[derive(Clone, Debug, PartialEq)]
pub struct BenchOptions {
    /// The install's offered map, by its place in the list.
    pub map: usize,
    /// The game's seed.
    pub seed: u32,
    /// AI houses (at most the map's starts).
    pub players: usize,
    /// Ticks run before the timing starts.
    pub late: u32,
    /// Timed frames per view.
    pub frames: u32,
    /// Untimed frames at the start of each view.
    pub warmup: u32,
    /// The views, in order.
    pub views: Vec<ViewKind>,
    /// The drawing size in physical pixels.
    pub size: (u32, u32),
    /// The outputs' base name; empty: the host's name.
    pub label: String,
    /// The folder of the native outputs.
    pub out: String,
    /// A tick every frame instead of the paced ticks.
    pub every_frame: bool,
    /// The window waits for the display's refresh (off by default, so a
    /// frame's time is its own).
    pub vsync: bool,
    /// The offscreen host writes a picture of each view's last warmup
    /// frame.
    pub shots: bool,
}

impl Default for BenchOptions {
    fn default() -> Self {
        // nocite: our own tooling (the defaults of the benchmark, M8 step C4)
        Self {
            map: 9,
            seed: 7,
            players: 8,
            late: 9_000,
            frames: 600,
            warmup: 60,
            views: vec![
                ViewKind::Base,
                ViewKind::Battle,
                ViewKind::Open,
                ViewKind::Pan,
            ],
            size: (1920, 1080),
            label: String::new(),
            out: "local-lane/m8/c4".into(),
            every_frame: false,
            vsync: false,
            shots: true,
        }
    }
}

impl BenchOptions {
    /// Parses a spec of `key=value` words separated by commas, semicolons
    /// or spaces.
    pub fn parse(spec: &str) -> Result<Self, String> {
        let mut o = Self::default();
        let words = spec
            .split([',', ';', ' '])
            .map(str::trim)
            .filter(|w| !w.is_empty() && *w != "default" && *w != "1" && *w != "on");
        for w in words {
            let (k, v) = w
                .split_once('=')
                .ok_or_else(|| format!("bench option {w:?} is not key=value"))?;
            let num = |v: &str| {
                v.replace('_', "")
                    .parse::<u64>()
                    .map_err(|e| format!("bench option {k}: {e}"))
            };
            let flag = |v: &str| match v {
                "on" | "yes" | "true" | "1" => Ok(true),
                "off" | "no" | "false" | "0" => Ok(false),
                _ => Err(format!("bench option {k}: expected on or off")),
            };
            match k {
                "map" => o.map = num(v)? as usize,
                "seed" => o.seed = num(v)? as u32,
                "players" => o.players = num(v)?.clamp(2, 8) as usize,
                "late" => o.late = num(v)? as u32,
                "frames" => o.frames = num(v)?.max(1) as u32,
                "warmup" => o.warmup = num(v)?.max(1) as u32,
                "views" => {
                    o.views = v
                        .split(['+', '/'])
                        .map(ViewKind::parse)
                        .collect::<Result<_, _>>()?;
                }
                "size" => {
                    let (w, h) = v.split_once('x').ok_or("bench option size: expected WxH")?;
                    o.size = (num(w)?.max(64) as u32, num(h)?.max(64) as u32);
                }
                "label" => o.label = v.to_owned(),
                "out" => o.out = v.to_owned(),
                "tick" => {
                    o.every_frame = match v {
                        "paced" => false,
                        "every" => true,
                        _ => return Err("bench option tick: expected paced or every".into()),
                    }
                }
                "vsync" => o.vsync = flag(v)?,
                "shots" => o.shots = flag(v)?,
                _ => return Err(format!("unknown bench option {k:?}")),
            }
        }
        if o.views.is_empty() {
            return Err("bench option views: no view".into());
        }
        Ok(o)
    }
}

/// What the host measured of a frame after [`Bench::before_frame`].
#[derive(Clone, Debug, Default)]
pub struct FrameEnd {
    /// The drawer's laps.
    pub laps: DrawLaps,
    /// The drawer's counts.
    pub stats: DrawStats,
    /// Waiting for the surface's texture (the window).
    pub acquire: f64,
    /// Recording the frame's pass and finishing the encoder.
    pub encode: f64,
    /// Submitting, and presenting (the window).
    pub submit: f64,
    /// Waiting for the frame before the last (the offscreen host keeps two
    /// frames in flight, as a swap chain does).
    pub wait: f64,
    /// GPU times that came back, by frame.
    pub gpu: Vec<(u64, f64)>,
}

/// One frame's row.
#[derive(Clone, Debug, Default)]
struct Row {
    frame: u64,
    view: usize,
    timed: bool,
    tick: i32,
    ticked: bool,
    objects: usize,
    start: f64,
    /// From this frame's start to the next's.
    total: Option<f64>,
    /// The host's work for the frame: before_frame to after_frame.
    work: f64,
    update: f64,
    tick_ms: f64,
    play: f64,
    scene: f64,
    radar: f64,
    draw: DrawLaps,
    acquire: f64,
    encode: f64,
    submit: f64,
    wait: f64,
    gpu: Option<f64>,
    stats: DrawStats,
    /// The scene's images with pixels, and their pixels, after the frame
    /// (#105).
    scene_images: usize,
    scene_px: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    /// Waiting for the install to be read.
    Install,
    /// The map is loading.
    Loading,
    /// Running the late ticks.
    Forward,
    /// Timing frames.
    Run,
    /// Done, or failed.
    Finished,
}

/// The benchmark's run: drives a [`Client`] through the game and keeps the
/// rows.
pub struct Bench {
    opts: BenchOptions,
    host: String,
    /// The graphics backend and adapter, for the summary.
    pub device: String,
    /// The device has GPU timestamps.
    pub gpu_timer: bool,
    stage: Stage,
    rows: Vec<Row>,
    view: usize,
    in_view: u32,
    /// Frames the host could not draw while timing (#92).
    undrawn: u32,
    centre: (i32, i32),
    pan: Option<(i32, i32)>,
    excluded: f64,
    forward_ticks: u32,
    forward_ms: f64,
    forward_start: Option<f64>,
    notes: Vec<String>,
    failure: Option<String>,
    /// Whether the window host has stored the outputs.
    reported: bool,
    resolution: f64,
    load_before: Option<String>,
    load_after: Option<String>,
    map_hash: String,
    start_tick: i32,
    start_objects: usize,
    census: String,
    speed: u8,
    end_hash: Option<String>,
    size: (u32, u32),
    last_log: f64,
}

impl Bench {
    /// A run of `opts` on host `host` (`native-offscreen`, `native-window`
    /// or `web`).
    pub fn new(opts: BenchOptions, host: &str) -> Self {
        let size = opts.size;
        Self {
            opts,
            host: host.to_owned(),
            device: String::new(),
            gpu_timer: false,
            stage: Stage::Install,
            rows: Vec::new(),
            view: 0,
            in_view: 0,
            undrawn: 0,
            centre: (0, 0),
            pan: None,
            excluded: 0.0,
            forward_ticks: 0,
            forward_ms: 0.0,
            forward_start: None,
            notes: Vec::new(),
            failure: None,
            reported: false,
            resolution: clock_resolution(),
            load_before: load_average(),
            load_after: None,
            map_hash: String::new(),
            start_tick: 0,
            start_objects: 0,
            census: String::new(),
            speed: 0,
            end_hash: None,
            size,
            last_log: 0.0,
        }
    }

    /// The options.
    pub fn options(&self) -> &BenchOptions {
        &self.opts
    }

    /// The outputs' base name.
    pub fn label(&self) -> String {
        if self.opts.label.is_empty() {
            self.host.clone()
        } else {
            self.opts.label.clone()
        }
    }

    /// Sets the size the host draws at (the window's may differ from the
    /// one asked for).
    pub fn set_size(&mut self, size: (u32, u32)) {
        self.size = size;
    }

    /// Whether the run is over (done or failed).
    pub fn finished(&self) -> bool {
        self.stage == Stage::Finished
    }

    /// True once, when the run has finished (done or failed) and its
    /// outputs are still to be stored.
    pub fn take_report(&mut self) -> bool {
        let first = self.finished() && !self.reported;
        self.reported |= first;
        first
    }

    /// Why the run failed, if it did.
    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    /// Whether the frame now being drawn is timed or warming up (its
    /// [`Bench::frame`] numbers the GPU timer's readback).
    pub fn running(&self) -> bool {
        self.stage == Stage::Run
    }

    /// The number of the frame now being drawn (after [`Bench::before_frame`]).
    pub fn frame(&self) -> u64 {
        self.rows.len().saturating_sub(1) as u64
    }

    /// The picture to take of the frame just drawn, if any: each view's
    /// last warmup frame, when shots are on.
    pub fn shot(&self) -> Option<String> {
        let last_warmup = self.in_view + 1 == self.opts.warmup;
        (self.opts.shots && self.stage == Stage::Run && last_warmup)
            .then(|| format!("{}-{}.png", self.label(), self.opts.views[self.view].name()))
    }

    /// Host time spent outside the frame (a picture's readback), left out
    /// of the next frame's total.
    pub fn exclude(&mut self, ms: f64) {
        self.excluded += ms;
    }

    fn fail(&mut self, why: String) {
        log::error!("benchmark: {why}");
        self.failure = Some(why);
        self.stage = Stage::Finished;
    }

    /// Before the frame's drawing, instead of [`Client::update`]: sets the
    /// game up, runs the late ticks, then picks the view and runs the
    /// frame's update.
    pub fn before_frame(&mut self, c: &mut Client) {
        match self.stage {
            Stage::Install => self.install(c),
            Stage::Loading => self.loading(c),
            Stage::Forward => self.forward(c),
            Stage::Run => self.run(c),
            Stage::Finished => {}
        }
    }

    fn install(&mut self, c: &mut Client) {
        if c.job.is_some() || c.busy() {
            return;
        }
        let Some(install) = c.install.as_ref() else {
            if c.vfs.is_some() {
                let why = format!("the install was not read: {}", c.status);
                self.fail(why);
            }
            return;
        };
        let Some(vfs) = c.vfs.take() else {
            return;
        };
        let maps: Vec<MapEntry> = install.maps.iter().map(|(_, m)| m.clone()).collect();
        let Some(entry) = maps.get(self.opts.map) else {
            c.vfs = Some(vfs);
            let why = format!(
                "map {} is not offered (the install offers {})",
                self.opts.map,
                maps.len()
            );
            self.fail(why);
            return;
        };
        let offered: Vec<String> = maps
            .iter()
            .enumerate()
            .map(|(i, m)| format!("{i}:{}", m.starts))
            .collect();
        log::debug!(
            "benchmark: offered maps (index:starts) {}",
            offered.join(" ")
        );
        let players = self.opts.players.min(entry.starts as usize).max(2);
        if players != self.opts.players {
            self.notes.push(format!(
                "{players} houses: the map has {} starts",
                entry.starts
            ));
        }
        let mut setup = Setup::new(&install.setup);
        setup.map = self.opts.map;
        // The benchmark measures the reference ruleset, never our own
        // fixes layer (M8 step C7).
        setup.fixes = false;
        for (i, slot) in setup.slots.iter_mut().enumerate() {
            slot.kind = if i < players {
                SlotKind::Ai
            } else {
                SlotKind::Closed
            };
        }
        let lobby = match setup.lobby(&install.setup, &maps) {
            Ok(l) => l,
            Err(p) => {
                c.vfs = Some(vfs);
                self.fail(format!("the setup was refused: {p:?}"));
                return;
            }
        };
        // The benchmark plays no sound; the events are still worked out.
        c.audio = Some(Audio::new(Box::new(NullOutput::new(
            DEFAULT_RATE,
            "the benchmark plays no sound",
        ))));
        c.session = None;
        let job = Job::Start {
            vfs,
            file: install.maps[self.opts.map].0.file,
            lobby,
            offered: install.setup.countries.clone(),
            seed: self.opts.seed,
            speech: install
                .speech
                .as_ref()
                .map(SpeechCatalog::from_ini)
                .unwrap_or_default(),
            local: None,
            cursor: install.cursor_palette.clone(),
            colour_blind: false,
            watch: true,
        };
        self.map_hash = install
            .map_hashes
            .get(self.opts.map)
            .cloned()
            .unwrap_or_default();
        c.begin_job(job);
        self.stage = Stage::Loading;
        log::info!(
            "benchmark: map {} ({players} AI houses), seed {}",
            self.opts.map,
            self.opts.seed
        );
    }

    fn loading(&mut self, c: &mut Client) {
        if let Some(s) = c.session.as_mut() {
            s.lap_clock = Some(now);
            self.stage = Stage::Forward;
            self.forward_start = Some(now());
            return;
        }
        if c.job.is_none() && !c.busy() && c.shell.screen != Screen::Game {
            let why = format!("the game did not start: {}", c.status);
            self.fail(why);
        }
    }

    fn forward(&mut self, c: &mut Client) {
        let Some(s) = c.session.as_mut() else {
            self.fail("the game went away".into());
            return;
        };
        let t0 = now();
        let mut ended = false;
        // A tenth of a second of ticks per frame keeps the window alive.
        // nocite: our own tooling
        while self.forward_ticks < self.opts.late && now() - t0 < 100.0 {
            s.play.game.step();
            // Nothing draws these ticks: their effects go.
            s.play.game.world.effects.clear();
            self.forward_ticks += 1;
            // nocite: our own tooling (a census line every 1000 late ticks)
            if self.forward_ticks.is_multiple_of(1000) {
                log::info!(
                    "benchmark: tick {}: {}",
                    self.forward_ticks,
                    census(&s.play.game.world)
                );
            }
            if s.play.game.outcome().is_some() {
                ended = true;
                break;
            }
        }
        if now() - self.last_log > 5000.0 {
            self.last_log = now();
            log::info!(
                "benchmark: tick {} of {}",
                self.forward_ticks,
                self.opts.late
            );
        }
        if ended {
            self.notes.push(format!(
                "the game ended after {} late ticks",
                self.forward_ticks
            ));
        }
        if ended || self.forward_ticks >= self.opts.late {
            self.forward_ms = now() - self.forward_start.unwrap_or(t0);
            s.catch_up();
            let w = &s.play.game.world;
            self.start_tick = w.tick;
            self.start_objects = w.objects.values().filter(|o| o.on_map).count();
            self.census = census(w);
            self.speed = s.speed;
            log::info!(
                "benchmark: {} ticks in {:.1} s; timing from tick {} with {} objects",
                self.forward_ticks,
                self.forward_ms / 1000.0,
                self.start_tick,
                self.start_objects
            );
            self.stage = Stage::Run;
        }
    }

    fn run(&mut self, c: &mut Client) {
        let start = now();
        if let Some(last) = self.rows.last_mut() {
            last.total = Some(start - last.start - std::mem::take(&mut self.excluded));
        }
        let frame = self.rows.len() as u64;
        let Some(s) = c.session.as_mut() else {
            self.fail("the game went away".into());
            return;
        };
        if self.in_view == 0 {
            self.pick_view(s);
        }
        let kind = self.opts.views[self.view];
        let centre = match (kind, self.pan) {
            (ViewKind::Pan, Some((left, right))) => {
                // There and back over the view's frames.
                let n = i64::from(self.opts.warmup + self.opts.frames);
                let i = i64::from(self.in_view);
                let half = (n / 2).max(1);
                let t = if i <= half { i } else { (2 * half - i).max(0) };
                let x = i64::from(left) + (i64::from(right - left) * t) / half;
                (x as i32, self.centre.1)
            }
            _ => self.centre,
        };
        s.set_centre(centre);
        let before = s.play.game.world.tick;
        c.focused = true;
        // The virtual clock of a steady 60 frames a second, or a second a
        // frame when every frame ticks.
        // nocite: our own tooling
        let clock = if self.opts.every_frame {
            frame as i64 * 1000
        } else {
            frame as i64 * 1000 / 60
        };
        c.update(clock);
        let update = now() - start;
        let Some(s) = c.session.as_ref() else {
            self.fail("the game went away".into());
            return;
        };
        let w = &s.play.game.world;
        let ticked = w.tick != before;
        let laps = if ticked { s.laps } else { Default::default() };
        self.rows.push(Row {
            frame,
            view: self.view,
            timed: self.in_view >= self.opts.warmup,
            tick: w.tick,
            ticked,
            objects: w.objects.len(),
            start,
            total: None,
            update,
            tick_ms: laps.tick,
            play: laps.play,
            scene: laps.scene,
            radar: laps.radar,
            ..Row::default()
        });
        if c.shell.screen != Screen::Game {
            self.notes
                .push(format!("the game ended at tick {}", w.tick));
            self.finish(c);
        }
    }

    /// After the frame was drawn and submitted; `drawn` is false when the
    /// host had no surface to draw to (a covered or minimised window).
    pub fn after_frame(&mut self, c: &Client, end: FrameEnd, drawn: bool) {
        self.gpu_times(&end.gpu);
        if self.stage != Stage::Run {
            return;
        }
        let Some(row) = self.rows.last_mut() else {
            return;
        };
        if !drawn {
            // Not a frame of the view: its row stays, untimed, and the view
            // still gets all its frames once the window draws again.
            row.timed = false;
            self.undrawn += 1;
            return;
        }
        row.work = now() - row.start;
        row.draw = end.laps;
        row.stats = end.stats;
        (row.scene_images, row.scene_px) = c.session.as_ref().map_or((0, 0), |s| s.scene.held());
        row.acquire = end.acquire;
        row.encode = end.encode;
        row.submit = end.submit;
        row.wait = end.wait;
        self.in_view += 1;
        if self.in_view >= self.opts.warmup + self.opts.frames {
            self.in_view = 0;
            self.view += 1;
            if self.view >= self.opts.views.len() {
                // The last frame's total ends here, with no next frame to
                // start.
                if let Some(last) = self.rows.last_mut() {
                    last.total = Some(now() - last.start - std::mem::take(&mut self.excluded));
                }
                self.finish(c);
            }
        }
    }

    /// GPU times that came back after the run's last frame.
    pub fn gpu_times(&mut self, times: &[(u64, f64)]) {
        for &(f, ms) in times {
            if let Some(r) = self.rows.get_mut(f as usize) {
                r.gpu = Some(ms);
            }
        }
    }

    fn finish(&mut self, c: &Client) {
        if self.undrawn > 0 {
            self.notes.push(format!(
                "{} frame(s) the window could not draw (covered or minimised) left out of the timing",
                self.undrawn
            ));
        }
        self.end_hash = c.session.as_ref().map(|s| s.play.game.state_hash());
        self.load_after = load_average();
        self.stage = Stage::Finished;
    }

    fn pick_view(&mut self, s: &mut Session) {
        let kind = self.opts.views[self.view];
        let (vw, vh) = {
            let v = s.view();
            (v.size.0 as i32, v.size.1 as i32)
        };
        let (left, right, top, bottom) = extent(s);
        let w = &s.play.game.world;
        // The benchmark watches through its first house (M8 step C4).
        let local = s.play.me();
        let at = |o: &glasnost_sim::world::Object| project_lifted(o.pos.x, o.pos.y, o.pos.z);
        let in_view =
            |a: (i32, i32), b: (i32, i32)| (a.0 - b.0).abs() * 2 < vw && (a.1 - b.1).abs() * 2 < vh;
        let player = |w: &World, o: &glasnost_sim::world::Object| {
            o.owner.is_some_and(|h| {
                w.houses
                    .get(usize::from(h.0))
                    .is_some_and(|x| x.role == HouseRole::Player)
            })
        };
        self.pan = None;
        let objects: Vec<_> = w.objects.values().filter(|o| o.on_map).collect();
        match kind {
            ViewKind::Base => {
                let mut buildings: Vec<_> = objects
                    .iter()
                    .filter(|o| o.kind == Kind::Building && o.owner == Some(local))
                    .collect();
                if buildings.is_empty() {
                    buildings = objects
                        .iter()
                        .filter(|o| o.kind == Kind::Building && player(w, o))
                        .collect();
                }
                let best = buildings
                    .iter()
                    .map(|o| {
                        let p = at(o);
                        let n = buildings.iter().filter(|b| in_view(at(b), p)).count();
                        (n, std::cmp::Reverse(o.id), p)
                    })
                    .max();
                if let Some((n, _, p)) = best {
                    self.centre = p;
                    self.notes
                        .push(format!("base view: {n} buildings around {p:?}"));
                }
            }
            ViewKind::Battle => {
                let mobile: Vec<_> = objects
                    .iter()
                    .filter(|o| o.kind.is_mobile() && player(w, o))
                    .collect();
                let best = mobile
                    .iter()
                    .map(|o| {
                        let p = at(o);
                        let (mut own, mut other) = (0usize, 0usize);
                        for m in &mobile {
                            if in_view(at(m), p) {
                                if m.owner == o.owner {
                                    own += 1;
                                } else {
                                    other += 1;
                                }
                            }
                        }
                        (own.min(other), own + other, std::cmp::Reverse(o.id), p)
                    })
                    .max();
                if let Some((fighting, n, _, p)) = best {
                    self.centre = p;
                    self.notes.push(format!(
                        "battle view: {n} units, {fighting} on the smaller side, around {p:?}"
                    ));
                }
            }
            ViewKind::Open => {
                // nocite: our own tooling (an 8 by 8 grid of candidate centres)
                const GRID: i32 = 8;
                let mut best: Option<(usize, (i32, i32))> = None;
                for gy in 0..GRID {
                    for gx in 0..GRID {
                        let p = (
                            left + (right - left) * gx / (GRID - 1),
                            top + (bottom - top) * gy / (GRID - 1),
                        );
                        let n = objects.iter().filter(|o| in_view(at(o), p)).count();
                        if best.is_none_or(|(m, _)| n < m) {
                            best = Some((n, p));
                        }
                    }
                }
                if let Some((n, p)) = best {
                    self.centre = p;
                    self.notes
                        .push(format!("open view: {n} objects around {p:?}"));
                }
            }
            ViewKind::Pan => {
                self.centre = (left, (top + bottom) / 2);
                self.pan = Some((left, right));
                self.notes.push(format!(
                    "pan view: x {left} to {right} at y {}",
                    (top + bottom) / 2
                ));
            }
        }
    }

    /// The per-frame rows as CSV.
    pub fn csv(&self) -> String {
        let mut out = String::from(
            "frame,view,timed,tick,ticked,objects,total_ms,work_ms,update_ms,tick_ms,play_ms,scene_ms,radar_ms,hud_ms,sync_ms,light_ms,map_ms,batch_ms,acquire_ms,encode_ms,submit_ms,wait_ms,gpu_ms,map_items,map_instances,map_vertices,map_draws,map_passes,map_copies,map_copied_px,map_uploads,map_upload_bytes,batch_quads,batch_vertices,batch_draws,batch_uploads,batch_upload_bytes,rebuilds,atlas_layers,atlas_committed_px,map_rows,scene_images,scene_px\n",
        );
        let opt = |v: Option<f64>| v.map_or(String::new(), |v| format!("{v:.3}"));
        for r in &self.rows {
            let m = &r.stats.map;
            let b = &r.stats.batch;
            let _ = writeln!(
                out,
                "{},{},{},{},{},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                r.frame,
                self.opts.views[r.view].name(),
                u8::from(r.timed),
                r.tick,
                u8::from(r.ticked),
                r.objects,
                opt(r.total),
                r.work,
                r.update,
                r.tick_ms,
                r.play,
                r.scene,
                r.radar,
                r.draw.hud,
                r.draw.sync,
                r.draw.light,
                r.draw.map,
                r.draw.batch,
                r.acquire,
                r.encode,
                r.submit,
                r.wait,
                opt(r.gpu),
                m.items,
                m.instances,
                m.instances * 6,
                m.draws,
                m.passes,
                m.copies,
                m.copied_px,
                m.uploads,
                m.upload_bytes,
                b.quads,
                b.quads * 6,
                b.draws,
                b.uploads,
                b.upload_bytes,
                r.stats.rebuilds,
                r.stats.atlas.layers,
                r.stats.atlas.committed_px,
                r.stats.atlas.rows,
                r.scene_images,
                r.scene_px,
            );
        }
        out
    }

    /// The summary text.
    pub fn summary(&self) -> String {
        let mut out = String::new();
        let o = &self.opts;
        let _ = writeln!(out, "Glasnost benchmark (M8 C4): {}", self.label());
        let _ = writeln!(out, "build: {}", glasnost_net::BUILD);
        let _ = writeln!(out, "host: {}; device: {}", self.host, self.device);
        let _ = writeln!(
            out,
            "size: {}x{}; map: index {} (content hash {}); seed {}; {} AI houses asked; {} ticks {}",
            self.size.0,
            self.size.1,
            o.map,
            self.map_hash,
            o.seed,
            o.players,
            if o.every_frame {
                "every frame"
            } else {
                "paced at a virtual 60 frames/s"
            },
            if o.vsync {
                "with vsync"
            } else {
                "without vsync"
            },
        );
        let _ = writeln!(
            out,
            "late ticks: {} in {:.1} s ({:.2} ms a tick, no drawing); timing from tick {} with {} objects on the map",
            self.forward_ticks,
            self.forward_ms / 1000.0,
            self.forward_ms / f64::from(self.forward_ticks.max(1)),
            self.start_tick,
            self.start_objects
        );
        let _ = writeln!(
            out,
            "per view: {} warmup frames left out, {} timed; host clock step {:.4} ms; GPU timestamps: {}",
            o.warmup,
            o.frames,
            self.resolution,
            if self.gpu_timer { "yes" } else { "no" }
        );
        let _ = writeln!(
            out,
            "game speed index {} (paced period {} ms); on the map at the start of timing: {}",
            self.speed,
            glasnost_ui::setup::period_ms(self.speed),
            self.census
        );
        if let Some(h) = &self.end_hash {
            let _ = writeln!(out, "state hash at the end: {h}");
        }
        let _ = writeln!(
            out,
            "load average: before {}; after {}",
            self.load_before.as_deref().unwrap_or("n/a"),
            self.load_after.as_deref().unwrap_or("n/a")
        );
        for n in &self.notes {
            let _ = writeln!(out, "note: {n}");
        }
        if let Some(f) = &self.failure {
            let _ = writeln!(out, "FAILED: {f}");
        }
        let _ = writeln!(out);
        let quantised = refresh_quantised(&self.host);
        let _ = writeln!(
            out,
            "frame time (ms, start to next start)  frames   mean median    p95    p99  worst  {}  ticks",
            if quantised { "  missed" } else { ">16.7ms" }
        );
        let groups: Vec<(String, Vec<&Row>)> = (0..o.views.len())
            .map(|v| {
                (
                    o.views[v].name().to_owned(),
                    self.rows
                        .iter()
                        .filter(|r| r.timed && r.view == v && r.total.is_some())
                        .collect(),
                )
            })
            .chain(std::iter::once((
                "all".to_owned(),
                self.rows
                    .iter()
                    .filter(|r| r.timed && r.total.is_some())
                    .collect(),
            )))
            .collect();
        let mut interval = None;
        for (name, rows) in &groups {
            let totals: Vec<f64> = rows.iter().filter_map(|r| r.total).collect();
            let st = Stats::of(&totals);
            let ticks = rows.iter().filter(|r| r.ticked).count();
            let (missed, refresh) = missed_frames(&totals, quantised);
            interval = refresh;
            let _ = writeln!(
                out,
                "  {name:<35} {:>6} {:>6.2} {:>6.2} {:>6.2} {:>6.2} {:>6.2} {:>8} {:>6}",
                st.n, st.mean, st.median, st.p95, st.p99, st.worst, missed, ticks
            );
        }
        if let Some(i) = interval {
            let _ = writeln!(
                out,
                "note: on this host the totals are whole display refresh intervals plus timer jitter; \
                 \"missed\" counts totals of at least 1.5 refresh intervals, the interval estimated as \
                 the median total ({i:.2} ms over all views); \"work\" leaves out the GPU work the \
                 browser runs later"
            );
        }
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "other times (ms)                      frames   mean median    p95    p99  worst"
        );
        type Pick = fn(&Row) -> Option<f64>;
        type Part = fn(&Row) -> f64;
        type Count = fn(&Row) -> u64;
        let measures: [(&str, Pick, bool); 4] = [
            ("work (CPU, whole frame)", |r| Some(r.work), false),
            ("GPU (timestamps)", |r| r.gpu, false),
            ("tick (frames that ticked)", |r| Some(r.tick_ms), true),
            (
                "update+scene (frames that ticked)",
                |r| Some(r.update),
                true,
            ),
        ];
        for (name, rows) in &groups {
            for (what, pick, ticked) in measures {
                let v: Vec<f64> = rows
                    .iter()
                    .filter(|r| !ticked || r.ticked)
                    .filter_map(|r| pick(r))
                    .collect();
                if v.is_empty() {
                    continue;
                }
                let st = Stats::of(&v);
                let _ = writeln!(
                    out,
                    "  {:<35} {:>6} {:>6.2} {:>6.2} {:>6.2} {:>6.2} {:>6.2}",
                    format!("{name}: {what}"),
                    st.n,
                    st.mean,
                    st.median,
                    st.p95,
                    st.p99,
                    st.worst
                );
            }
        }
        let _ = writeln!(out);
        let _ = writeln!(out, "mean per timed frame (ms), by part:");
        let timed: Vec<&Row> = self.rows.iter().filter(|r| r.timed).collect();
        let n = timed.len().max(1) as f64;
        // (+ 0.0: an empty sum is −0.0, printed as such)
        let mean = |f: fn(&Row) -> f64| timed.iter().map(|r| f(r)).sum::<f64>() / n + 0.0;
        let parts: [(&str, Part); 15] = [
            ("update (all)", |r| r.update),
            ("  tick", |r| r.tick_ms),
            ("  play", |r| r.play),
            ("  scene", |r| r.scene),
            ("  radar", |r| r.radar),
            ("draw list (hud)", |r| r.draw.hud),
            ("scene uploads (sync)", |r| r.draw.sync),
            ("light alpha", |r| r.draw.light),
            ("map draw recording", |r| r.draw.map),
            ("batch", |r| r.draw.batch),
            ("acquire", |r| r.acquire),
            ("encode", |r| r.encode),
            ("submit+present", |r| r.submit),
            ("wait (offscreen)", |r| r.wait),
            ("work (whole frame)", |r| r.work),
        ];
        for (name, f) in parts {
            let _ = writeln!(out, "  {name:<24} {:>8.3}", mean(f));
        }
        let _ = writeln!(out);
        let _ = writeln!(out, "counts per timed frame (mean, max):");
        let counts: [(&str, Count); 13] = [
            ("map items", |r| r.stats.map.items as u64),
            ("map instances", |r| r.stats.map.instances as u64),
            ("map vertices", |r| r.stats.map.instances as u64 * 6),
            ("map draw calls", |r| r.stats.map.draws as u64),
            ("map passes", |r| r.stats.map.passes as u64),
            ("map backdrop copies", |r| r.stats.map.copies as u64),
            ("map copied pixels", |r| r.stats.map.copied_px),
            ("map uploads", |r| r.stats.map.uploads as u64),
            ("map upload bytes", |r| r.stats.map.upload_bytes),
            ("batch quads", |r| r.stats.batch.quads as u64),
            ("batch draw calls", |r| r.stats.batch.draws as u64),
            ("batch uploads", |r| r.stats.batch.uploads as u64),
            ("batch upload bytes", |r| r.stats.batch.upload_bytes),
        ];
        for (name, f) in counts {
            let sum: u64 = timed.iter().map(|r| f(r)).sum();
            let max = timed.iter().map(|r| f(r)).max().unwrap_or(0);
            let _ = writeln!(out, "  {name:<24} {:>12.1} {:>12}", sum as f64 / n, max);
        }
        let rebuilds: usize = timed.iter().map(|r| r.stats.rebuilds).sum();
        let _ = writeln!(out, "  {:<24} {:>12}", "renderer rebuilds", rebuilds);
        // The atlas and the scene's images at the first and the last timed
        // frame, and their largest values (#105).
        let _ = writeln!(out);
        let _ = writeln!(out, "atlas and scene images (first, last, max):");
        let gauges: [(&str, Count); 5] = [
            ("atlas layers", |r| u64::from(r.stats.atlas.layers)),
            ("atlas committed pixels", |r| r.stats.atlas.committed_px),
            ("palette rows", |r| r.stats.atlas.rows as u64),
            ("scene images held", |r| r.scene_images as u64),
            ("scene image pixels", |r| r.scene_px),
        ];
        for (name, f) in gauges {
            let first = timed.first().map_or(0, |r| f(r));
            let last = timed.last().map_or(0, |r| f(r));
            let max = timed.iter().map(|r| f(r)).max().unwrap_or(0);
            let _ = writeln!(out, "  {name:<24} {first:>12} {last:>12} {max:>12}");
        }
        out
    }

    /// The outputs: `<label>.txt` (the summary) and `<label>.csv`.
    pub fn outputs(&self) -> Vec<(String, Vec<u8>)> {
        let label = self.label();
        vec![
            (format!("{label}.txt"), self.summary().into_bytes()),
            (format!("{label}.csv"), self.csv().into_bytes()),
        ]
    }

    /// Writes the outputs into the options' folder; returns their paths.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) -> Result<Vec<String>, String> {
        let dir = std::path::Path::new(&self.opts.out);
        std::fs::create_dir_all(dir).map_err(|e| format!("cannot make {}: {e}", dir.display()))?;
        let mut paths = Vec::new();
        for (name, bytes) in self.outputs() {
            let p = dir.join(name);
            std::fs::write(&p, bytes).map_err(|e| format!("cannot write {}: {e}", p.display()))?;
            paths.push(p.display().to_string());
        }
        Ok(paths)
    }
}

/// The objects on the map by kind, and the player houses' mobile units
/// and buildings.
fn census(w: &World) -> String {
    let on: Vec<_> = w.objects.values().filter(|o| o.on_map).collect();
    let count = |k: Kind| on.iter().filter(|o| o.kind == k).count();
    let mut out = format!(
        "{} buildings, {} vehicles, {} infantry, {} aircraft, {} terrain objects; by house (mobile/buildings):",
        count(Kind::Building),
        count(Kind::Vehicle),
        count(Kind::Infantry),
        count(Kind::Aircraft),
        count(Kind::Terrain)
    );
    for (i, h) in w.houses.iter().enumerate() {
        if h.role != HouseRole::Player {
            continue;
        }
        let mine =
            |o: &&&glasnost_sim::world::Object| o.owner.is_some_and(|x| usize::from(x.0) == i);
        let mobile = on
            .iter()
            .filter(mine)
            .filter(|o| o.kind.is_mobile())
            .count();
        let built = on
            .iter()
            .filter(mine)
            .filter(|o| o.kind == Kind::Building)
            .count();
        let _ = write!(out, " {i}:{mobile}/{built}");
    }
    out
}

/// The view centres' limits: left, right, top and bottom.
fn extent(s: &mut Session) -> (i32, i32, i32, i32) {
    let keep = s.centre();
    // nocite: our own tooling (requests far outside any map, limited by S-0101 F5)
    const FAR: i32 = 1 << 24;
    s.set_centre((-FAR, -FAR));
    let (left, top) = s.centre();
    s.set_centre((FAR, FAR));
    let (right, bottom) = s.centre();
    s.set_centre(keep);
    (left, right, top, bottom)
}

/// Mean, median, p95, p99 and worst of a sample (nearest rank).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Stats {
    n: usize,
    mean: f64,
    median: f64,
    p95: f64,
    p99: f64,
    worst: f64,
}

impl Stats {
    fn of(v: &[f64]) -> Self {
        if v.is_empty() {
            return Self::default();
        }
        let mut s = v.to_vec();
        s.sort_by(f64::total_cmp);
        let rank = |p: f64| s[((p * s.len() as f64).ceil() as usize).clamp(1, s.len()) - 1];
        Self {
            n: s.len(),
            mean: s.iter().sum::<f64>() / s.len() as f64,
            median: rank(0.5),
            p95: rank(0.95),
            p99: rank(0.99),
            worst: s[s.len() - 1],
        }
    }
}

/// The host clock's smallest step seen.
fn clock_resolution() -> f64 {
    let mut best = f64::INFINITY;
    for _ in 0..5 {
        let a = now();
        let mut b = now();
        let mut spins = 0u32;
        // nocite: our own tooling (enough spins for a millisecond clock)
        while b == a && spins < 10_000_000 {
            b = now();
            spins += 1;
        }
        if b > a {
            best = best.min(b - a);
        }
    }
    best
}

/// The system's load averages, where the host shows them.
fn load_average() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/loadavg")
            .ok()
            .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// Runs the benchmark natively without a window: the install in
/// `game_dir`, drawn offscreen at the options' size on a device of
/// `backend` with timestamp queries where it has them, two frames in
/// flight. Writes the outputs and each view's picture; returns the summary.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_offscreen(
    opts: BenchOptions,
    game_dir: &str,
    backend: glasnost_render::gpu::BackendChoice,
) -> Result<String, String> {
    use glasnost_render::gpu;
    use glasnost_render::offscreen::{OFFSCREEN_FORMAT, read_texture};
    use glasnost_render::timer::{GpuTimer, TIMER_FEATURES};
    use glasnost_render::wgpu;

    let (device, queue, info) = pollster::block_on(async {
        let instance = gpu::create_instance(backend, None).await;
        let adapter = gpu::request_adapter(&instance, None)
            .await
            .map_err(|e| e.to_string())?;
        let info = adapter.get_info();
        let (device, queue) = gpu::request_device_with(&adapter, TIMER_FEATURES)
            .await
            .map_err(|e| e.to_string())?;
        Ok::<_, String>((device, queue, info))
    })?;
    let mut timer = GpuTimer::new(&device, &queue);
    let mut bench = Bench::new(opts, "native-offscreen");
    bench.device = format!(
        "{} on {} ({})",
        gpu::backend_label(info.backend),
        info.name,
        info.driver
    );
    bench.gpu_timer = timer.is_some();
    let (w, h) = bench.options().size;
    let mut client = Client::new();
    client.cpu_map = false;
    client.resize((w as i32, h as i32));
    client.mount(crate::viewer::tasks::native_files(std::path::Path::new(
        game_dir,
    ))?);
    let mut draw = super::GpuDraw::new(&device, OFFSCREEN_FORMAT);
    draw.lap_clock = Some(now);
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("benchmark target"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: OFFSCREEN_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let mut last: Option<wgpu::SubmissionIndex> = None;
    let out = std::path::PathBuf::from(&bench.options().out);
    while !bench.finished() {
        while let Some(job) = client.job.take() {
            let done = pollster::block_on(super::client::run_job(job));
            client.done(done);
        }
        bench.before_frame(&mut client);
        if bench.finished() {
            break;
        }
        // Only the run's frames are timed on the GPU, as in the window host:
        // before it, `frame` is 0 and would land on the first row (#92).
        let frame = bench.running().then(|| bench.frame());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("benchmark frame"),
        });
        if let Some(t) = timer.as_mut().filter(|_| frame.is_some()) {
            t.begin(&mut encoder);
        }
        draw.prepare(&device, &queue, &mut client, (w, h), &mut encoder);
        let t0 = now();
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("benchmark frame"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            draw.draw(&mut pass);
        }
        if let Some(t) = timer.as_mut().filter(|_| frame.is_some()) {
            t.end(&mut encoder);
        }
        let commands = encoder.finish();
        let t1 = now();
        let index = queue.submit([commands]);
        if let (Some(t), Some(frame)) = (timer.as_mut(), frame) {
            t.submitted(frame);
        }
        let t2 = now();
        if let Some(prev) = last.replace(index) {
            let _ = device.poll(wgpu::PollType::Wait {
                submission_index: Some(prev),
                timeout: None,
            });
        }
        let t3 = now();
        let gpu = timer
            .as_mut()
            .map(|t| t.collect(&device))
            .unwrap_or_default();
        let shot = bench.shot();
        bench.after_frame(
            &client,
            FrameEnd {
                laps: draw.laps,
                stats: draw.take_stats(),
                acquire: 0.0,
                encode: t1 - t0,
                submit: t2 - t1,
                wait: t3 - t2,
                gpu,
            },
            true,
        );
        if let Some(name) = shot {
            let t = now();
            let rgba = read_texture(&device, &queue, &target, w, h);
            let _ = std::fs::create_dir_all(&out);
            let png = crate::mapview::png::png(w, h, &rgba);
            if let Err(e) = std::fs::write(out.join(&name), png) {
                log::warn!("benchmark: cannot write {name}: {e}");
            }
            bench.exclude(now() - t);
        }
    }
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    if let Some(t) = timer.as_mut() {
        let rest = t.collect(&device);
        bench.gpu_times(&rest);
    }
    let paths = bench.save()?;
    for p in &paths {
        log::info!("benchmark: wrote {p}");
    }
    let summary = bench.summary();
    match bench.failure() {
        Some(f) => Err(format!("{f}\n{summary}")),
        None => Ok(summary),
    }
}

/// Starts the benchmark's game natively without drawing: the install in
/// `game_dir`, the skirmish of `opts` (map, seed, houses) set up exactly as
/// [`Bench`] sets it up, stopped before its first tick. Returns the client
/// holding the session and the map's content hash. The tools' tick
/// benchmark (M8 C4 part 2) times the ticks from here.
#[cfg(not(target_arch = "wasm32"))]
pub fn start_headless(opts: BenchOptions, game_dir: &str) -> Result<(Client, String), String> {
    let mut bench = Bench::new(opts, "ticks");
    let mut client = Client::new();
    client.cpu_map = false;
    client.mount(crate::viewer::tasks::native_files(std::path::Path::new(
        game_dir,
    ))?);
    while !bench.finished() && bench.stage != Stage::Forward {
        while let Some(job) = client.job.take() {
            let done = pollster::block_on(super::client::run_job(job));
            client.done(done);
        }
        bench.before_frame(&mut client);
    }
    if let Some(why) = bench.failure() {
        return Err(why.to_owned());
    }
    Ok((client, bench.map_hash.clone()))
}

/// Whether host `host`'s frame totals are whole display refresh intervals:
/// on the web, redraws come from the browser's animation frames whatever the
/// present mode, so a start-to-next-start time cannot show whether the work
/// fitted a 60 Hz budget (#91).
fn refresh_quantised(host: &str) -> bool {
    host == "web"
}

/// The frames of `totals` that missed their slot, and the refresh interval
/// estimated for a refresh-quantised host: there, the median total is the
/// interval and a total of at least 1.5 intervals missed a refresh;
/// elsewhere, a total over [`BUDGET_MS`] missed.
fn missed_frames(totals: &[f64], quantised: bool) -> (usize, Option<f64>) {
    if !quantised {
        return (totals.iter().filter(|&&t| t > BUDGET_MS).count(), None);
    }
    if totals.is_empty() {
        return (0, None);
    }
    let interval = Stats::of(totals).median;
    // nocite: our own tooling (#91: halfway between one refresh interval and two)
    let late = 1.5 * interval;
    (
        totals.iter().filter(|&&t| t >= late).count(),
        Some(interval),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A run of one view of `warmup` and `frames` frames, already timing.
    fn running(warmup: u32, frames: u32) -> Bench {
        let mut b = Bench::new(
            BenchOptions {
                warmup,
                frames,
                views: vec![ViewKind::Base],
                ..Default::default()
            },
            "native-window",
        );
        b.stage = Stage::Run;
        b
    }

    /// A frame's start as `run` records it.
    fn start_row(b: &mut Bench) {
        if let Some(last) = b.rows.last_mut() {
            last.total = Some(now() - last.start);
        }
        b.rows.push(Row {
            frame: b.rows.len() as u64,
            view: b.view,
            timed: b.in_view >= b.opts.warmup,
            start: now(),
            ..Row::default()
        });
    }

    /// #92: a frame the window could not draw (no surface texture) is not a
    /// timed frame and does not count towards the view's frames.
    #[test]
    fn an_undrawn_frame_is_not_timed() {
        let c = Client::new();
        let mut b = running(0, 2);
        start_row(&mut b);
        b.after_frame(&c, FrameEnd::default(), false);
        assert!(!b.rows[0].timed);
        assert!(!b.finished());
        for _ in 0..2 {
            start_row(&mut b);
            b.after_frame(&c, FrameEnd::default(), true);
        }
        assert!(b.finished());
        assert_eq!(b.rows.iter().filter(|r| r.timed).count(), 2);
        assert!(
            b.notes.iter().any(|n| n.contains("1 frame")),
            "{:?}",
            b.notes
        );
    }

    /// #92: the run's last frame has a total too.
    #[test]
    fn the_last_frame_has_a_total() {
        let c = Client::new();
        let mut b = running(1, 2);
        for _ in 0..3 {
            start_row(&mut b);
            b.after_frame(&c, FrameEnd::default(), true);
        }
        assert!(b.finished());
        assert!(b.rows.iter().all(|r| r.total.is_some()), "{:?}", b.rows);
    }

    /// #91: on the web, frame totals are counted against the refresh
    /// interval they are quantised to; elsewhere against the 60 Hz budget.
    #[test]
    fn web_totals_count_missed_refreshes() {
        assert!(refresh_quantised("web"));
        assert!(!refresh_quantised("native-offscreen"));
        assert!(!refresh_quantised("native-window"));
        // A 120 Hz display: most frames take one refresh (8.33 ms, with
        // jitter), two take two, one takes three. None is over 16.7 ms,
        // which hides the three misses.
        let mut totals = vec![8.3, 8.4, 8.33, 8.2, 8.5, 8.33, 8.3, 8.4];
        totals.extend([16.65, 16.6, 25.0]);
        assert_eq!(missed_frames(&totals, false), (1, None));
        let (missed, interval) = missed_frames(&totals, true);
        assert_eq!(missed, 3);
        assert!((interval.unwrap() - 8.33).abs() < 0.2);
        // A 60 Hz display: a frame of 16.9 ms is one refresh with jitter,
        // not a miss.
        let totals = [16.6, 16.9, 16.7, 16.5, 33.4];
        assert_eq!(missed_frames(&totals, false).0, 3);
        assert_eq!(missed_frames(&totals, true).0, 1);
        assert_eq!(missed_frames(&[], true), (0, None));
    }

    #[test]
    fn options_parse() {
        assert_eq!(BenchOptions::parse("default"), Ok(BenchOptions::default()));
        assert_eq!(BenchOptions::parse(""), Ok(BenchOptions::default()));
        let o =
            BenchOptions::parse("map=3,late=1_000;frames=50 views=pan+base size=1280x720").unwrap();
        assert_eq!(o.map, 3);
        assert_eq!(o.late, 1000);
        assert_eq!(o.frames, 50);
        assert_eq!(o.views, vec![ViewKind::Pan, ViewKind::Base]);
        assert_eq!(o.size, (1280, 720));
        assert!(BenchOptions::parse("views=").is_err());
        assert!(BenchOptions::parse("zoom=2").is_err());
        assert!(BenchOptions::parse("tick=sometimes").is_err());
    }

    #[test]
    fn stats_take_nearest_ranks() {
        let v: Vec<f64> = (1..=100).map(f64::from).collect();
        let s = Stats::of(&v);
        assert_eq!(s.n, 100);
        assert_eq!(s.median, 50.0);
        assert_eq!(s.p95, 95.0);
        assert_eq!(s.p99, 99.0);
        assert_eq!(s.worst, 100.0);
        assert!((s.mean - 50.5).abs() < 1e-9);
        assert_eq!(Stats::of(&[]).n, 0);
    }
}
