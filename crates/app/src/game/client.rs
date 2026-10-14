//! The game client without its window: the screens (`glasnost_ui::shell`),
//! the install, the running [`Session`], pacing, sound and input. The
//! window layer (`super::window`) feeds it events and draws what
//! [`Client::frame`] returns; tests and local snapshots drive it directly
//! and draw on the CPU ([`Client::frame_rgba`]).
//!
//! Work that reads files is asynchronous (the web reads picked files
//! lazily): the client hands out a [`Job`], the window layer runs it
//! ([`run_job`]) and gives the result back ([`Client::done`]).

use std::collections::BTreeMap;

use glasnost_audio::catalog::SoundCatalog;
use glasnost_audio::bank::SoundBank;
use glasnost_audio::events::{Audio, local_voice_set};
use glasnost_audio::mixer::View as SoundView;
use glasnost_audio::speech::SpeechCatalog;
use glasnost_render::batch2d::CpuImage;
use glasnost_sim::rng::Rng;
use glasnost_sim::targeting::{KeyCommand, Modifiers};
use glasnost_ui::draw::{DrawList, colour};
use glasnost_ui::end::End;
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::menu::{self, MenuHit};
use glasnost_ui::setup::{self as ui_setup, Lobby, SetupHit};
use glasnost_ui::shell::{Event, Screen, Shell};
use glasnost_ui::widget::Targets;
use glasnost_vfs::{FileRef, PickedFile, Vfs};

use super::install::{self, Install};
use super::paint::Painter;
use super::play::Button;
use super::session::{MAP_IMAGE, RADAR_IMAGE, SCROLL_STEP, Session, SessionEvent};
use crate::mapview::live::{Pacer, period_ms};

/// Asynchronous work the client asks for.
pub enum Job {
    /// Mount the picked install files.
    Mount(Vec<PickedFile>),
    /// Read the menus' data from the mounted install.
    Prepare(Vfs),
    /// Load a map and start its game.
    Start {
        /// The file system.
        vfs: Vfs,
        /// The map file.
        file: FileRef,
        /// The setup screen's choices.
        lobby: Lobby,
        /// The countries the screen offered.
        offered: Vec<String>,
        /// The game's seed.
        seed: u32,
        /// The announcement data, for loading the local voice set's files
        /// (S-0701 claim 34).
        speech: SpeechCatalog,
    },
}

/// The result of a [`Job`].
pub enum Done {
    /// The install was mounted, or why not.
    Mounted(Result<Vfs, String>),
    /// The menus' data, with the file system back.
    Prepared(Vfs, Result<Box<Install>, String>),
    /// The game started, or why not, with the file system back and the
    /// local voice set's announcement files (S-0701 claim 34).
    Started(Vfs, Box<Result<Session, String>>, SoundBank),
}

/// Runs a job.
pub async fn run_job(job: Job) -> Done {
    match job {
        Job::Mount(files) => {
            let result = match crate::viewer::tasks::mount(files).await {
                crate::viewer::tasks::TaskResult::Mounted(Ok(b)) => Ok(b.0),
                crate::viewer::tasks::TaskResult::Mounted(Err(e)) => Err(e),
                _ => Err("mounting gave no file system".into()),
            };
            Done::Mounted(result)
        }
        Job::Prepare(mut vfs) => {
            let r = install::prepare(&mut vfs).await.map(Box::new);
            Done::Prepared(vfs, r)
        }
        Job::Start {
            mut vfs,
            file,
            lobby,
            offered,
            seed,
            speech,
        } => {
            let make = |rules: &glasnost_sim::rules::SimRules| {
                // spec: S-0202 claim 8 (both streams take the game's seed); the lobby's own stream
                // takes it too (S-0412 claims 6 and 7)
                ui_setup::game_setup(&lobby, rules, &offered, seed, seed).map_err(|e| e.to_string())
            };
            let r = Session::start(&mut vfs, &file, &make).await;
            let mut voices = SoundBank::new();
            if let Ok(s) = &r {
                let set = local_voice_set(&s.play.game.world, Some(s.play.local));
                // spec: S-0701 claim 34 (each announcement a separate file, opened through the file lookup, never from the bank)
                for name in speech.files(set) {
                    if let Some(f) = glasnost_convert::inputs::read(&vfs, &name).await {
                        voices.add_file(&name, std::rc::Rc::from(f.bytes));
                    }
                }
            }
            Done::Started(vfs, Box::new(r), voices)
        }
    }
}

/// A key the client reacts to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// A letter or digit, lower case.
    Char(char),
    /// Escape: the options menu.
    Escape,
    /// An arrow key.
    Left,
    /// An arrow key.
    Right,
    /// An arrow key.
    Up,
    /// An arrow key.
    Down,
}

/// The images a frame shows, by id, when they changed.
pub struct FrameImages {
    /// `(id, width, height, rgba)`.
    pub changed: Vec<(u32, u32, u32, Vec<u8>)>,
}

/// The client.
pub struct Client {
    /// The screens.
    pub shell: Shell,
    /// The install's menu data, once read.
    pub install: Option<Install>,
    /// The mounted file system, while no job holds it.
    pub vfs: Option<Vfs>,
    /// The running game.
    pub session: Option<Session>,
    /// The quads' atlas and palettes.
    pub painter: Painter,
    /// The menu's status line.
    pub status: String,
    /// Sound, created at the first game start.
    pub audio: Option<Audio>,
    /// Whether the window has the focus (S-0201 claim 9).
    pub focused: bool,
    /// Whether the battlefield is composed here on the CPU (else the window
    /// layer draws it on the GPU).
    pub cpu_map: bool,
    /// Counts the games started (the window layer re-uploads the
    /// battlefield's scene when it changes).
    pub game_serial: u64,
    /// Whether Quit is offered (not on the web).
    pub can_quit: bool,
    /// Window size.
    pub size: (i32, i32),
    /// The seed of the next game (the window layer sets it from the host
    /// clock; S-0202 claim 8).
    pub next_seed: u32,
    /// A job waiting to run.
    pub job: Option<Job>,
    /// Set when the person chose Quit.
    pub quit: bool,
    /// The person asked to pick an install folder.
    pub pick_install: bool,
    busy: bool,
    pacer: Pacer,
    unsync: Rng,
    pointer: Option<Point>,
    press: Option<Point>,
    held: [bool; 4],
    /// The main menu's targets of the last frame.
    pub menu_targets: Targets<MenuHit>,
    /// The setup screen's targets of the last frame.
    pub setup_targets: Targets<SetupHit>,
    /// The end screen's targets of the last frame.
    pub end_targets: Targets<()>,
    radar_tick: i32,
    cpu_images: BTreeMap<u32, CpuImage>,
}

impl Client {
    /// A client at the main menu.
    pub fn new() -> Self {
        Self {
            shell: Shell::new(),
            install: None,
            vfs: None,
            session: None,
            painter: Painter::new(),
            status: "Pick the game's install folder to begin.".into(),
            audio: None,
            focused: true,
            cpu_map: true,
            game_serial: 0,
            can_quit: true,
            size: (1024, 768),
            next_seed: 1,
            job: None,
            quit: false,
            pick_install: false,
            busy: false,
            pacer: Pacer::default(),
            unsync: Rng::new(1),
            pointer: None,
            press: None,
            held: [false; 4],
            menu_targets: Targets::new(),
            setup_targets: Targets::new(),
            end_targets: Targets::new(),
            radar_tick: -1,
            cpu_images: BTreeMap::new(),
        }
    }

    /// Mounts picked install files.
    pub fn mount(&mut self, files: Vec<PickedFile>) {
        self.status = "Mounting the install...".into();
        self.busy = true;
        self.job = Some(Job::Mount(files));
    }

    /// The result of a job.
    pub fn done(&mut self, d: Done) {
        self.busy = false;
        match d {
            Done::Mounted(Ok(vfs)) => {
                self.status = "Reading the rules and maps...".into();
                self.busy = true;
                self.job = Some(Job::Prepare(vfs));
            }
            Done::Mounted(Err(e)) => self.status = e,
            Done::Prepared(vfs, r) => {
                self.vfs = Some(vfs);
                match r {
                    Ok(i) => {
                        self.status = format!("Install ready: {}.", i.summary);
                        self.install = Some(*i);
                    }
                    Err(e) => self.status = e,
                }
            }
            Done::Started(vfs, r, voices) => {
                self.vfs = Some(vfs);
                match *r {
                    Ok(mut s) => {
                        for p in &s.problems {
                            log::info!("game load: {p}");
                        }
                        s.resize(self.size);
                        // A fresh atlas per game: the icons of the last one go.
                        self.painter = Painter::new();
                        s.register_art(&mut self.painter);
                        // spec: S-0202 claim 8 (the unsynchronised stream takes the game's seed)
                        self.unsync = Rng::new(s.play.game.replay().setup.seed);
                        self.pacer.reset();
                        self.radar_tick = -1;
                        self.game_serial += 1;
                        self.session = Some(s);
                        self.start_audio(voices);
                        self.shell.screen = Screen::Game;
                    }
                    Err(e) => {
                        self.status = e.clone();
                        log::warn!("the game did not start: {e}");
                        self.shell.start_failed();
                    }
                }
            }
        }
    }

    fn start_audio(&mut self, voices: SoundBank) {
        let Some(install) = self.install.as_ref() else {
            return;
        };
        let Some(s) = self.session.as_ref() else {
            return;
        };
        let w = &s.play.game.world;
        let q = w.rules().quirks;
        let audio = self.audio.get_or_insert_with(Audio::platform);
        audio.set_quirks(q);
        let sounds = install.sounds.as_ref().map(|d| {
            SoundCatalog::from_ini(d, q.default_sound_priority_ignored, q.full_volume_16383)
        });
        audio.set_catalog(sounds.unwrap_or_default());
        let speech = install.speech.as_ref().map(SpeechCatalog::from_ini);
        audio.set_speech(speech.unwrap_or_default());
        audio.set_bank(install.bank.clone());
        audio.set_voices(voices);
        audio.set_names(install.names.clone());
        // interim: Q-0333 (the presentation generator takes the game's seed)
        audio.start(w, Some(s.play.local), s.play.game.replay().setup.seed);
    }

    /// A user gesture happened: the browser lets sound start now.
    pub fn gesture(&mut self) {
        if let Some(a) = self.audio.as_mut() {
            a.resume();
        }
    }

    fn event(&mut self, e: Option<Event>) {
        match e {
            Some(Event::StartGame(lobby)) => self.start_game(lobby),
            Some(Event::PickInstall) => self.pick_install = true,
            Some(Event::Quit) => self.quit = true,
            None => {}
        }
    }

    fn start_game(&mut self, lobby: Lobby) {
        let (Some(install), Some(vfs)) = (self.install.as_ref(), self.vfs.take()) else {
            self.shell.start_failed();
            return;
        };
        let Some((listing, _)) = install.maps.get(lobby.map) else {
            self.vfs = Some(vfs);
            self.shell.start_failed();
            return;
        };
        self.session = None;
        self.status = "Loading the map...".into();
        self.busy = true;
        self.job = Some(Job::Start {
            vfs,
            file: listing.file,
            lobby,
            offered: install.setup.countries.clone(),
            seed: self.next_seed,
            speech: install
                .speech
                .as_ref()
                .map(SpeechCatalog::from_ini)
                .unwrap_or_default(),
        });
    }

    /// Sets the window size.
    pub fn resize(&mut self, size: (i32, i32)) {
        self.size = (size.0.max(64), size.1.max(64));
        if let Some(s) = self.session.as_mut() {
            s.resize(self.size);
        }
    }

    /// The pointer moved.
    pub fn pointer_moved(&mut self, p: Point) {
        self.pointer = Some(p);
        if let Some(s) = self.session.as_mut() {
            s.pointer_moved(p);
        }
    }

    /// The pointer left the window.
    pub fn pointer_left(&mut self) {
        self.pointer = None;
        if let Some(s) = self.session.as_mut() {
            s.pointer_left();
        }
    }

    /// A button went down.
    pub fn pointer_down(&mut self, b: Button, p: Point) {
        self.gesture();
        self.pointer = Some(p);
        self.press = Some(p);
        if self.shell.screen == Screen::Game
            && let Some(s) = self.session.as_mut()
        {
            s.pointer_down(b, p);
        }
    }

    /// A button went up.
    pub fn pointer_up(&mut self, b: Button, p: Point, mods: Modifiers) {
        self.pointer = Some(p);
        let pressed = self.press.take();
        match &self.shell.screen {
            Screen::Game => {
                let Some(s) = self.session.as_mut() else {
                    return;
                };
                if s.pointer_up(b, p, mods) == SessionEvent::Leave {
                    self.session = None;
                    self.shell.leave_game();
                }
            }
            _ if b != Button::Left || pressed.is_none() || self.busy => {}
            Screen::Menu => {
                let ready = self.install.is_some();
                if let Some(hit) = self.menu_targets.hit(p) {
                    if hit == MenuHit::Skirmish && !ready {
                        return;
                    }
                    let rules = self.install.as_ref().map(|i| i.setup.clone());
                    match rules {
                        Some(r) => {
                            let e = self.shell.menu(hit, &r);
                            self.event(e);
                        }
                        None if hit == MenuHit::Install => self.pick_install = true,
                        None if hit == MenuHit::Quit => self.quit = true,
                        None => {}
                    }
                }
            }
            Screen::Setup => {
                let Some(hit) = self.setup_targets.hit(p) else {
                    return;
                };
                let Some(i) = self.install.as_ref() else {
                    return;
                };
                let rules = i.setup.clone();
                let maps: Vec<_> = i.maps.iter().map(|(_, m)| m.clone()).collect();
                let e = self.shell.setup(hit, &rules, &maps);
                self.event(e);
            }
            Screen::End(_) => {
                if self.end_targets.hit(p).is_some() {
                    self.session = None;
                    self.shell.end_done();
                }
            }
        }
    }

    /// A key went down or up.
    pub fn key(&mut self, k: Key, down: bool) {
        if down {
            self.gesture();
        }
        let arrow = match k {
            Key::Left => Some(0),
            Key::Right => Some(1),
            Key::Up => Some(2),
            Key::Down => Some(3),
            _ => None,
        };
        if let Some(i) = arrow {
            self.held[i] = down;
            return;
        }
        if !down || self.shell.screen != Screen::Game {
            return;
        }
        let Some(s) = self.session.as_mut() else {
            return;
        };
        match k {
            Key::Escape => s.toggle_pause(),
            // spec: S-0213 table T2 (the key commands); the default keys are interim (Q-0286)
            Key::Char('s') => s.key(KeyCommand::Stop),
            Key::Char('x') => s.key(KeyCommand::Scatter),
            Key::Char('g') => s.key(KeyCommand::Guard),
            Key::Char('d') => s.key(KeyCommand::Deploy),
            Key::Char('c') if !s.paused => s.play.cheer(),
            _ => {}
        }
    }

    /// The window lost or gained the focus (S-0201 claim 9).
    pub fn focus(&mut self, on: bool) {
        self.focused = on;
        if !on {
            self.held = [false; 4];
        }
    }

    /// Advances the game for a drawn frame at host time `now_ms`: at most
    /// one tick (S-0201 claims 2 to 4), none while paused or without focus
    /// (claims 8 and 9); then scrolling, sound and the end of the game.
    pub fn update(&mut self, now_ms: i64) {
        if self.shell.screen != Screen::Game {
            return;
        }
        let Some(s) = self.session.as_mut() else {
            return;
        };
        // spec: S-0201 claims 8 and 9 (no cycle runs while the options menu is open or without focus)
        if !s.ticking() || !self.focused {
            self.pacer.reset();
        } else if self.pacer.due(now_ms, s.speed) {
            s.step();
            if let Some(a) = self.audio.as_mut() {
                a.tick(&s.play.game.world, &s.play.effects, &mut self.unsync);
            }
        }
        let cues = std::mem::take(&mut s.play.cues);
        let acks = std::mem::take(&mut s.play.acks);
        let selected = std::mem::take(&mut s.play.selected);
        let keys = std::mem::take(&mut s.play.keys);
        if let Some(a) = self.audio.as_mut() {
            let w = &s.play.game.world;
            for cue in cues {
                a.cue(cue);
            }
            for id in selected {
                a.selected(w, id, &mut self.unsync);
            }
            for (selection, click) in acks {
                a.ordered(w, &selection, &click, &mut self.unsync);
            }
            for k in keys {
                a.key(k);
            }
        }
        let mut d = if self.focused { s.edge_scroll() } else { (0, 0) };
        let [l, r, u, dn] = self.held;
        d.0 += SCROLL_STEP * (i32::from(r) - i32::from(l));
        d.1 += SCROLL_STEP * (i32::from(dn) - i32::from(u));
        s.scroll(d);
        if let Some(a) = self.audio.as_mut() {
            let v = s.view();
            a.set_view(SoundView {
                origin: v.origin,
                size: (v.size.0 as i32, v.size.1 as i32),
                // spec: S-0701 F2 (the position's place on screen: S-0101's projection with its height)
                project: glasnost_render::projection::project_lifted,
            });
            a.frame(now_ms);
        }
        if let Some(end) = s.play.end(period_ms(s.speed)) {
            self.shell.game_over(end);
        }
    }

    /// The frame's draw list, and the images that changed (the battlefield
    /// on the CPU when [`Client::cpu_map`], the radar when it moved on).
    pub fn frame(&mut self) -> (DrawList, FrameImages) {
        let size = self.size;
        let hover = self.pointer;
        let mut changed = Vec::new();
        let list = match &self.shell.screen {
            Screen::Menu => {
                let status = if self.busy {
                    format!("{} (working)", self.status)
                } else {
                    self.status.clone()
                };
                let (d, t) = menu::view(size, &status, self.install.is_some(), self.can_quit, hover);
                self.menu_targets = t;
                d
            }
            Screen::Setup => match (self.shell.setup.as_ref(), self.install.as_ref()) {
                (Some(setup), Some(i)) => {
                    let maps: Vec<_> = i.maps.iter().map(|(_, m)| m.clone()).collect();
                    let (mut d, t) = setup.view(size, &i.setup, &maps, hover);
                    if self.busy {
                        busy_veil(&mut d, size, &self.status);
                    }
                    self.setup_targets = t;
                    d
                }
                _ => DrawList::new(),
            },
            Screen::Game => match self.session.as_mut() {
                Some(s) => {
                    s.update_marks();
                    if self.cpu_map {
                        let v = s.view();
                        changed.push((MAP_IMAGE, v.size.0, v.size.1, s.map_rgba()));
                    }
                    let tick = s.play.game.world.tick;
                    if tick != self.radar_tick
                        && let Some(rgba) = s.radar_rgba()
                    {
                        let (w, h) = glasnost_ui::hud::RADAR;
                        changed.push((RADAR_IMAGE, w as u32, h as u32, rgba));
                        self.radar_tick = tick;
                    }
                    let v = s.view();
                    self.painter
                        .set_image_size(MAP_IMAGE, (v.size.0 as i32, v.size.1 as i32));
                    self.painter
                        .set_image_size(RADAR_IMAGE, glasnost_ui::hud::RADAR);
                    s.draw()
                }
                None => {
                    // Loading the map.
                    let mut d = DrawList::new();
                    busy_veil(&mut d, size, &self.status);
                    d
                }
            },
            Screen::End(end) => {
                let (d, t) = end.view(size, hover);
                self.end_targets = t;
                d
            }
        };
        (list, FrameImages { changed })
    }

    /// The whole frame drawn on the CPU (tests and local snapshots).
    pub fn frame_rgba(&mut self) -> Vec<u8> {
        let (list, images) = self.frame();
        for (id, w, h, rgba) in images.changed {
            self.cpu_images.insert(
                id,
                CpuImage {
                    w: w as i32,
                    h: h as i32,
                    rgba,
                },
            );
        }
        let quads = self.painter.quads(&list);
        self.painter.reference(&quads, &self.cpu_images, self.size)
    }

    /// The end screen shown, if any.
    pub fn end(&self) -> Option<&End> {
        match &self.shell.screen {
            Screen::End(e) => Some(e),
            _ => None,
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

fn busy_veil(d: &mut DrawList, size: (i32, i32), status: &str) {
    d.fill(Rect::new(0, 0, size.0, size.1), colour::VEIL);
    d.text_centred(Rect::new(0, size.1 / 2 - 8, size.0, 16), status, colour::TEXT);
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
