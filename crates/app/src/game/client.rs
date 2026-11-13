//! The game client without its window: the screens (`glasnost_ui::shell`),
//! the install, the running [`Session`], pacing, sound and input. The
//! window layer (`super::window`) feeds it events and draws what
//! [`Client::frame`] returns; tests and local snapshots drive it directly
//! and draw on the CPU ([`Client::frame_rgba`]).
//!
//! Work that reads files is asynchronous (the web reads picked files
//! lazily): the client hands out a [`Job`], the window layer runs it
//! ([`run_job`]) and gives the result back ([`Client::done`]).
//!
//! Network games (M9 N3) live in the `net` part of this module
//! (`client_net.rs`): the multiplayer screen, a room on a relay through a
//! [`super::link::Link`], the lockstep-paced game and chat.

use std::collections::BTreeMap;

use glasnost_audio::bank::SoundBank;
use glasnost_audio::catalog::SoundCatalog;
use glasnost_audio::events::{Audio, local_voice_set};
use glasnost_audio::mixer::View as SoundView;
use glasnost_audio::music::{Music, Phase as MusicPhase};
use glasnost_audio::speech::SpeechCatalog;
use glasnost_audio::stand::Stand;
use glasnost_render::batch2d::CpuImage;
use glasnost_sim::rng::Rng;
use glasnost_sim::targeting::{KeyCommand, Modifiers};
use glasnost_sim::world::HouseId;
use glasnost_ui::draw::{DrawList, colour};
use glasnost_ui::end::End;
use glasnost_ui::geom::{Point, Rect, Scale};
use glasnost_ui::menu::{self, MenuHit};
use glasnost_ui::netmenu::{NetForm, NetHit};
use glasnost_ui::setup::{self as ui_setup, Lobby, SetupHit};
use glasnost_ui::shell::{Event, Screen, Shell};
use glasnost_ui::widget::{Focus, Nav, Targets};
use glasnost_vfs::{FileRef, PickedFile, Vfs};

use super::captions::Captions;
use super::direct::{Ice, OpenPair, Shape, Timeouts, platform_direct};
use super::display::Display;
use super::install::{self, Install};
use super::link::{Connect, DEFAULT_RELAY, platform_connect};
use super::paint::Painter;
use super::play::Button;
use super::report::{SaveReport, platform_save};
use super::session::{GroupCommand, MAP_IMAGE, RADAR_IMAGE, Session, SessionEvent, SideKey};
use super::touch::{self, Carry, Gesture, Strip, Touches, WheelDelta};
use crate::mapview::live::Pacer;
use crate::settings::{Action, Bindings, Settings, Store};
use glasnost_ui::settings::SettingsHit;

/// Asynchronous work the client asks for. One job is in flight at a time,
/// so the size of its largest variant does not matter.
#[allow(clippy::large_enum_variant)]
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
        /// The house played from here (a network game's slot); `None`: the
        /// first person's house.
        local: Option<HouseId>,
        /// The session's cursor palette keep (S-0111 claim 19a).
        cursor: super::session::CursorPaletteKeep,
        /// The player houses show our colour-blind colours (the person's
        /// setting; presentation only).
        colour_blind: bool,
        /// `local` is an AI house the client watches (the performance
        /// benchmark, M8 step C4).
        watch: bool,
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
            local,
            cursor,
            colour_blind,
            watch,
        } => {
            let make = |rules: &glasnost_sim::rules::SimRules| {
                // spec: S-0202 claim 8 (both streams take the game's seed); the lobby's own stream
                // takes it too (S-0412 claims 6 and 7)
                ui_setup::game_setup(&lobby, rules, &offered, seed, seed).map_err(|e| e.to_string())
            };
            let r = Session::start(
                &mut vfs,
                &file,
                &make,
                local,
                &cursor,
                colour_blind,
                watch,
                lobby.layers,
            )
            .await;
            let mut voices = SoundBank::new();
            if let Ok(s) = &r {
                let set = local_voice_set(&s.play.game.world, s.play.local);
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
    /// Enter: sends a chat line, opens the game's chat; presses the focused
    /// control on a menu; places at the keyboard cell.
    Enter,
    /// Backspace in a text field; holds or cancels the focused sidebar
    /// icon.
    Backspace,
    /// Tab (with Shift: backwards): the next control.
    Tab,
    /// Space: presses the focused control.
    Space,
    /// Page Up: the previous sidebar tab.
    PageUp,
    /// Delete: deletes the selected route point (S-0213 claim 16e).
    Delete,
    /// Page Down: the next sidebar tab.
    PageDown,
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
    /// The side of the painter's atlas: the device's sprite atlas side
    /// ([`Client::fit_atlas`]).
    atlas_side: u32,
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
    /// The view's size in view pixels: the window's size divided by the
    /// display factor ([`Client::set_window`]); the window's size where
    /// the client was only given [`Client::resize`].
    pub size: (i32, i32),
    /// The window as the host gave it ([`Client::set_window`]); `None` for
    /// a client sized only through [`Client::resize`] (tests, the
    /// benchmark), which draws one view pixel per screen pixel.
    window: Option<Display>,
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
    /// The multiplayer screen's fields.
    pub netform: NetForm,
    /// How the client opens a link to a relay (`None`: no network here).
    pub connect: Option<Connect>,
    /// How the client opens a direct link to another peer of its game
    /// (M9 N7; `None`: the relay carries everything).
    pub direct: Option<OpenPair>,
    /// The direct links' ICE servers when set here (a command-line or URL
    /// option); else the settings' (none by default).
    pub ice: Ice,
    /// A bad network on the direct path (the soak, M9 N7).
    pub direct_shape: Option<Shape>,
    /// The direct links' timeouts (the defaults but in tests).
    pub direct_timeouts: Timeouts,
    /// The build this client names in a room ([`BUILD`]).
    pub build: String,
    /// How the client stores a desync report (`None`: it cannot here).
    pub save_report: Option<SaveReport>,
    /// How the client stores a pathing report (M8 A4; `None`: it cannot
    /// here).
    pub save_path_report: Option<SaveReport>,
    /// The test-only perturbation a test client makes in a network game
    /// ([`Client::set_test`]).
    perturb: Option<glasnost_net::desync::Perturb>,
    /// The multiplayer screen's targets of the last frame.
    pub net_targets: Targets<NetHit>,
    /// The person's settings (M8 step C2): presentation and input only, never
    /// read by the simulation.
    pub settings: Settings,
    /// The settings screen's targets of the last frame.
    pub settings_targets: Targets<SettingsHit>,
    /// The binding waiting for a key on the settings screen.
    pub rebinding: Option<usize>,
    /// The settings screen's last note (a refusal or a storage problem).
    pub settings_note: String,
    /// The settings screen was opened from the game's options menu.
    settings_from_game: bool,
    store: Option<Box<dyn Store>>,
    net: Option<net::Net>,
    net_status: (String, bool),
    /// The map of a network game is loading: its start is dropped if the
    /// game's link went meanwhile (M9 N5).
    net_loading: bool,
    chat_field: Option<Rect>,
    /// The host time of the last update.
    clock: i64,
    /// The modifiers held (Shift turns Tab backwards; Ctrl with an arrow
    /// moves the keyboard cell).
    mods: Modifiers,
    /// Whether the make-group key stands in for Ctrl with a digit: on in
    /// the web build only, where the browser may take Ctrl+digit first
    /// (design, M8 A4 round 5 seams); native keeps Ctrl+digit alone.
    pub group_fallback: bool,
    /// The make-group key is held.
    group_held: bool,
    /// The keyboard focus on the main menu.
    pub menu_focus: Focus<MenuHit>,
    /// The keyboard focus on the setup and room screens.
    pub setup_focus: Focus<SetupHit>,
    /// The keyboard focus on the multiplayer screen.
    pub net_focus: Focus<NetHit>,
    /// The keyboard focus on the settings screen.
    pub settings_focus: Focus<SettingsHit>,
    /// The keyboard focus on the end screen.
    pub end_focus: Focus<()>,
    /// The screen of the last frame (a new screen starts without focus).
    last_screen: Option<std::mem::Discriminant<Screen>>,
    /// The announcements' captions (M8 step C3).
    pub captions: Captions,
    /// The touch screen's gestures (`touch`; our own addition).
    touches: Touches,
    /// A touchpad's pixel fractions between swipe events.
    wheel_carry: Carry,
    /// A touchpad swipe's pixels toward the next icon-strip row.
    wheel_strip: Strip,
    /// The map picker's decoded previews (design: map picker (lead's
    /// request 2026-11-13)).
    previews: maps::Previews,
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
            atlas_side: glasnost_render::batch2d::ATLAS_SIZE,
            status: "Pick the game's install folder to begin.".into(),
            audio: None,
            focused: true,
            cpu_map: true,
            game_serial: 0,
            can_quit: true,
            size: (1024, 768),
            window: None,
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
            netform: NetForm {
                build: net::BUILD.into(),
                ..NetForm::new(DEFAULT_RELAY)
            },
            connect: platform_connect(),
            direct: platform_direct(),
            ice: Ice::default(),
            direct_shape: None,
            direct_timeouts: Timeouts::default(),
            build: net::BUILD.into(),
            save_report: platform_save(),
            save_path_report: super::capture::platform_save(),
            perturb: None,
            net_targets: Targets::new(),
            settings: Settings::default(),
            settings_targets: Targets::new(),
            rebinding: None,
            settings_note: String::new(),
            settings_from_game: false,
            store: None,
            net: None,
            net_status: (String::new(), false),
            net_loading: false,
            chat_field: None,
            clock: 0,
            mods: Modifiers::default(),
            // design: the web stand-in for Ctrl+digit (M8 A4 round 5 seams)
            group_fallback: cfg!(target_arch = "wasm32"),
            group_held: false,
            menu_focus: Focus::new(),
            setup_focus: Focus::new(),
            net_focus: Focus::new(),
            settings_focus: Focus::new(),
            end_focus: Focus::new(),
            last_screen: None,
            captions: Captions::new(),
            touches: Touches::new(),
            wheel_carry: Carry::default(),
            wheel_strip: Strip::default(),
            previews: maps::Previews::default(),
        }
    }

    /// The interface scale for the window: the settings' scale, lowered
    /// where the screen would be smaller than the smallest interface.
    pub fn ui_scale(&self) -> Scale {
        Scale::fit(i32::from(self.settings.ui_scale), self.size)
    }

    /// Whether a job is running (on the web a job leaves `job` when it
    /// starts and reports back later).
    pub fn busy(&self) -> bool {
        self.busy
    }

    /// The interface's size in its logical pixels.
    pub fn ui_size(&self) -> (i32, i32) {
        self.ui_scale().ui
    }

    /// Whether a text field or the chat takes the keyboard's text now (the
    /// window repeats a held Backspace only then).
    pub fn typing_text(&self) -> bool {
        self.typing()
    }

    /// Reads the settings from `store` and keeps every later change there.
    /// A store with nothing readable leaves the defaults.
    pub fn attach_store(&mut self, store: Box<dyn Store>) {
        if let Some(text) = store.read() {
            self.settings = Settings::from_text(&text);
        }
        self.shell.picker.query = self.settings.maps.clone();
        self.store = Some(store);
        self.apply_settings();
        if !self.settings.relay.is_empty() {
            self.netform.relay = self.settings.relay.clone();
        }
    }

    /// Puts the settings to use (sound levels, the game's scroll speed) and
    /// keeps them in the store.
    fn settings_changed(&mut self) {
        self.apply_settings();
        if let Some(store) = self.store.as_ref()
            && let Err(e) = store.write(&self.settings.to_text())
        {
            log::warn!("settings not kept: {e}");
            self.settings_note = format!("Settings could not be kept: {e}");
        }
    }

    fn apply_settings(&mut self) {
        self.refit_display();
        let st = &self.settings;
        if let Some(a) = self.audio.as_mut() {
            let m = a.mixer_mut();
            m.set_volume(Settings::mixer_level(st.sound));
            m.set_voice_volume(Settings::mixer_level(st.voices));
            m.set_music_volume(Settings::mixer_level(st.music));
        }
        let scale = self.ui_scale();
        if !self.settings.captions {
            self.captions.clear();
        }
        if let Some(s) = self.session.as_mut() {
            s.scroll_index = self.settings.scroll_index;
            s.resize(self.size, scale);
        }
    }

    /// Hands `job` to the host's runner; the client is busy until it is
    /// done.
    pub fn begin_job(&mut self, job: Job) {
        self.busy = true;
        self.job = Some(job);
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
                        // Another install's maps: its previews are not these.
                        self.previews = maps::Previews::default();
                    }
                    Err(e) => self.status = e,
                }
            }
            Done::Started(vfs, r, voices) => {
                self.vfs = Some(vfs);
                let net_start = std::mem::take(&mut self.net_loading);
                match *r {
                    // The network game's link went while its map loaded
                    // (the multiplayer screen says so): no game.
                    Ok(_) if net_start && self.net.is_none() => {}
                    Ok(mut s) => {
                        // spec: S-0015 State and ordering (the string table is read once, when the program starts; each game gets the install's copy)
                        if s.strings.is_none() {
                            s.strings = self.install.as_ref().and_then(|i| i.strings.clone());
                        }
                        for p in &s.problems {
                            log::info!("game load: {p}");
                        }
                        s.resize(self.size, self.ui_scale());
                        self.captions.clear();
                        // A fresh atlas per game: the icons of the last one go.
                        self.painter = Painter::with_side(self.atlas_side);
                        s.register_art(&mut self.painter);
                        // spec: S-0202 claim 8 (the unsynchronised stream takes the game's seed)
                        self.unsync = Rng::new(s.play.game.replay().setup.seed);
                        self.pacer.reset();
                        self.radar_tick = -1;
                        self.game_serial += 1;
                        self.session = Some(s);
                        self.apply_settings();
                        self.net_started();
                        self.start_audio(voices);
                        self.shell.screen = Screen::Game;
                    }
                    Err(e) => {
                        self.status = e.clone();
                        log::warn!("the game did not start: {e}");
                        if self.net.is_some() {
                            self.net_leave();
                            self.net_status = (format!("The game did not start: {e}"), true);
                            self.shell.screen = Screen::Multiplayer;
                        } else {
                            self.shell.start_failed();
                        }
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
        // spec: S-0703 claim 2 (the side restriction is matched against the sides as built for this game, S-0004 claim 1)
        let sides = s.loaded.import.outcome.sides.iter();
        audio.set_sides(
            sides
                .map(|r| r.name.iter().map(|&b| char::from(b)).collect())
                .collect(),
        );
        audio.set_music(
            install
                .music
                .as_ref()
                .map(|m| m.player_with(super::music_share::track_source(&m.files))),
        );
        // spec: S-0703 claim 11 (the map theme names the first track)
        audio.set_theme(s.loaded.theme.clone());
        audio.start(w, s.play.local);
        if let Some(m) = audio.music() {
            for warning in m.warnings() {
                log::warn!("{warning}");
            }
        }
        self.apply_settings();
    }

    /// A game starts to load (S-0703 claim 11): the music playing in the
    /// menus stops at once and the loading track, if any, starts.
    pub(super) fn loading_music(&mut self) {
        let player = self
            .install
            .as_ref()
            .and_then(|i| i.music.as_ref())
            .map(|m| m.player_with(super::music_share::track_source(&m.files)));
        let Some(player) = player else {
            if let Some(a) = self.audio.as_mut() {
                a.stop_music();
            }
            return;
        };
        let audio = self.audio.get_or_insert_with(Audio::platform);
        audio.set_music(Some(player));
        audio.loading_music();
        self.apply_settings();
    }

    /// The music for the screen shown, off the game's own frames (S-0703
    /// claims 11 to 13): the end screen's score track and the menus' track
    /// are fed to the output, our settings screen opened from a game counts
    /// as the options menu (the music plays on, sound stands still), and
    /// music left over from a game is stopped on a screen it does not belong
    /// to.
    fn screen_music(&mut self, now_ms: i64) {
        let Some(a) = self.audio.as_mut() else {
            return;
        };
        let Some(phase) = a.music().map(Music::phase) else {
            return;
        };
        let loading = self.session.is_none();
        let networked = self.net.as_ref().is_some_and(|n| n.begin.is_some());
        match &self.shell.screen {
            // spec: S-0703 claim 13 and S-0701 claim 30 (our engine's own settings screen opened from a game counts as the options menu: the music plays on with music-only passes, sound stands still, and lost focus stops both)
            Screen::Settings if self.settings_from_game => {
                // design: a network game's settings screen does not stand still for lost focus, as its game does not (S-0201 claim 16)
                let stand = if self.focused || networked {
                    Stand::Menu
                } else {
                    Stand::Unfocused
                };
                a.frame_in(now_ms, stand, &mut self.unsync);
            }
            // A game loading: its loading track, with no audio pass.
            Screen::Game if loading => a.menu_frame(),
            Screen::Game => {}
            // spec: S-0703 claim 12 (our engine: the end screen's score track, timed by the output, with no audio pass)
            Screen::End(_) => a.menu_frame(),
            // The menus: the menu track, if one started; music of a game
            // left by a way that starts no menu track stops at once.
            _ => {
                if !matches!(phase, MusicPhase::Menu | MusicPhase::Off) {
                    a.stop_music();
                }
                a.menu_frame();
            }
        }
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
            local: None,
            cursor: install.cursor_palette.clone(),
            colour_blind: self.settings.colour_blind,
            watch: false,
        });
        self.loading_music();
    }

    /// The window as the host gave it, with the display factor in use; one
    /// view pixel per screen pixel for a client sized only through
    /// [`Client::resize`].
    pub fn display(&self) -> Display {
        self.window.unwrap_or_else(|| Display::one(self.size))
    }

    /// The host's window: `physical` its drawable size in device pixels,
    /// `host_scale` its scale factor (the browser's `devicePixelRatio`).
    /// The view becomes the window divided by the display factor the
    /// display-scale setting picks (`super::display`); the window layer
    /// calls it at the start, on every resize and on every change of the
    /// scale factor (design: readable text and display scale (lead's
    /// request 2026-11-10)).
    pub fn set_window(&mut self, physical: (i32, i32), host_scale: f64) {
        let d = Display::new(physical, host_scale, self.settings.display_scale);
        self.window = Some(d);
        self.resize_view(d.view());
    }

    /// Sets the window size in view pixels, one per screen pixel (tests,
    /// local snapshots and the benchmark; the window layer uses
    /// [`Client::set_window`]).
    pub fn resize(&mut self, size: (i32, i32)) {
        self.window = None;
        self.resize_view(size);
    }

    /// Picks the display factor again after the setting changed.
    fn refit_display(&mut self) {
        if let Some(d) = self.window {
            let n = Display::new(d.physical, d.host_scale, self.settings.display_scale);
            if n != d {
                self.window = Some(n);
                self.resize_view(n.view());
            }
        }
    }

    fn resize_view(&mut self, size: (i32, i32)) {
        self.size = (size.0.max(64), size.1.max(64));
        let scale = self.ui_scale();
        if let Some(s) = self.session.as_mut() {
            s.resize(self.size, scale);
        }
    }

    /// The modifiers held changed.
    pub fn modifiers(&mut self, m: Modifiers) {
        self.mods = m;
        if let Some(s) = self.session.as_mut() {
            s.mods = m;
        }
    }

    /// Whether the game draws its own battlefield cursor this frame, so the
    /// window hides the system pointer (S-0214 claims 21 and 22).
    pub fn draws_cursor(&self) -> bool {
        self.shell.screen == Screen::Game && self.session.as_ref().is_some_and(|s| s.draws_cursor())
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

    /// The middle button went down or up (S-0111 claim 20: it only hides
    /// the name tip).
    pub fn middle_button(&mut self, p: Point) {
        self.pointer = Some(p);
        if self.shell.screen == Screen::Game
            && let Some(s) = self.session.as_mut()
        {
            s.middle_button(p);
        }
    }

    /// A mouse-wheel event with the sign of its vertical delta (S-0411
    /// claim 18).
    pub fn wheel(&mut self, delta: i32) {
        if self.picker_scrolls() {
            self.picker_wheel(delta);
        } else if self.shell.screen == Screen::Game
            && let Some(s) = self.session.as_mut()
        {
            s.wheel(delta);
        }
    }

    /// A host wheel event. A line delta (a browser's wheel notch included,
    /// [`WheelDelta::from_pixels`]), and any delta with touchpad scrolling
    /// off, is one [`Client::wheel`] step with the sign of its vertical
    /// delta (S-0411 claim 18); a pixel delta with the setting on is
    /// [`Client::pixel_wheel`].
    pub fn wheel_event(&mut self, d: WheelDelta) {
        match d {
            // design: touch and touchpad scrolling (lead's request 2026-11-10): only a pixel delta with the setting on swipes
            WheelDelta::Pixels(x, y) if self.settings.touchpad_scroll => self.pixel_wheel((x, y)),
            WheelDelta::Lines(y) | WheelDelta::Pixels(_, y) => {
                // Host input only: the wheel's direction (S-0411 claim 18).
                if y != 0.0 {
                    self.wheel(if y > 0.0 { 1 } else { -1 });
                }
            }
        }
    }

    /// A wheel event with a pixel delta (a touchpad swipe), `d` the
    /// content's movement in screen pixels (positive right and down), with
    /// touchpad scrolling on. Over the battlefield view it pans
    /// the view with the content; elsewhere it steps the icon strip one row
    /// per row height gathered (design: touch and touchpad scrolling
    /// (lead's request 2026-11-10)). [`Client::wheel_event`] sends it only
    /// with the setting on.
    pub fn pixel_wheel(&mut self, d: (f64, f64)) {
        let (dx, dy) = self.wheel_carry.take(d);
        if self.picker_scrolls() {
            let rows = self.wheel_strip.add(dy, self.picker_pitch());
            self.picker_rows(rows);
            return;
        }
        if self.shell.screen != Screen::Game {
            return;
        }
        let Some(s) = self.session.as_mut() else {
            return;
        };
        if self.pointer.is_some_and(|p| s.on_map(p)) {
            // design: touch and touchpad scrolling (lead's request 2026-11-10): the content follows the fingers, so the centre moves against the delta
            s.pan((-dx, -dy));
        } else {
            let rows = self.wheel_strip.add(dy, s.strip_pitch());
            self.wheel_rows(rows);
        }
    }

    /// Steps the icon strip `rows` rows, one S-0411 claim 18 wheel step
    /// each, signed as the wheel's delta.
    fn wheel_rows(&mut self, rows: i32) {
        for _ in 0..rows.unsigned_abs() {
            self.wheel(rows.signum());
        }
    }

    /// Whether the window drops mouse events at host time `now_ms`: while a
    /// finger is down and shortly after a touch ([`touch::MOUSE_QUIET_MS`]),
    /// so a system that also makes mouse events of the touches does not
    /// click twice. Only seen touches start it; the setting does not.
    pub fn mouse_quiet(&self, now_ms: i64) -> bool {
        self.touches.mouse_quiet(now_ms)
    }

    /// A touch screen's event for contact `id` at screen pixel `p` at host
    /// time `now_ms` (design: touch and touchpad scrolling (lead's request
    /// 2026-11-10)), whatever the touchpad setting. A drag on the
    /// battlefield pans the view with the finger, one on the sidebar steps
    /// the icon strip; a tap is a left click and a two-finger tap a right
    /// click, delivered as the mouse's are, on every screen. A touch never
    /// leaves the pointer anywhere, so no edge scrolling starts from it.
    pub fn touch(&mut self, phase: touch::Phase, id: u64, p: Point, now_ms: i64, mods: Modifiers) {
        if phase == touch::Phase::Start {
            self.gesture();
        }
        let (region, pitch) = match (&self.shell.screen, self.session.as_ref()) {
            (Screen::Game, Some(s)) if !s.paused => (
                if s.on_map(p) {
                    touch::Region::Map
                } else {
                    touch::Region::Bar
                },
                s.strip_pitch(),
            ),
            // design: map picker (lead's request 2026-11-13): a drag on the open picker scrolls its list a row per row height; a tap still clicks
            (Screen::Setup, _) if self.shell.picker.open => {
                (touch::Region::Bar, self.picker_pitch())
            }
            _ => (touch::Region::Other, 1),
        };
        match self.touches.event(phase, id, p, now_ms, region, pitch) {
            Some(Gesture::Pan(d)) => {
                if self.shell.screen == Screen::Game
                    && let Some(s) = self.session.as_mut()
                {
                    s.pan(d);
                }
            }
            Some(Gesture::Rows(n)) if self.picker_scrolls() => self.picker_rows(n),
            Some(Gesture::Rows(n)) => self.wheel_rows(n),
            Some(Gesture::Click(b, at)) => {
                self.pointer_moved(at);
                self.pointer_down(b, at);
                self.pointer_up(b, at, mods);
                // No pointer stays behind at the touch point (an edge
                // there is no mouse at the edge).
                self.pointer_left();
            }
            None => {}
        }
    }

    /// The running game's view centre and selection count (the browser
    /// checks' probe, `GameOptions::probe`); `None` outside a game.
    pub fn probe(&self) -> Option<((i32, i32), usize)> {
        if self.shell.screen != Screen::Game {
            return None;
        }
        let s = self.session.as_ref()?;
        Some((s.centre(), s.play.selection.len()))
    }

    /// A button went down.
    pub fn pointer_down(&mut self, b: Button, p: Point) {
        self.gesture();
        self.pointer = Some(p);
        self.press = Some(p);
        // The pointer takes over from the keyboard: the focus marks go.
        self.menu_focus.shown = false;
        self.setup_focus.shown = false;
        self.net_focus.shown = false;
        self.settings_focus.shown = false;
        self.end_focus.shown = false;
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
        // The screens outside the game are laid out in the interface's
        // logical pixels.
        let q = self.ui_scale().to_ui(p);
        match &self.shell.screen {
            Screen::Game => {
                let Some(s) = self.session.as_mut() else {
                    return;
                };
                let e = s.pointer_up(b, p, mods);
                self.session_event(e);
            }
            _ if b != Button::Left || pressed.is_none() || self.busy => {}
            Screen::Menu => {
                if let Some(hit) = self.menu_targets.hit(q) {
                    self.menu_focus.at = Some(hit);
                    self.menu_press(hit);
                }
            }
            Screen::Setup => {
                let hit = self.setup_targets.hit(q);
                if self.shell.picker.open {
                    // design: map picker (lead's request 2026-11-13): as on the multiplayer screen, a click off the search field ends its typing
                    self.shell.picker.typing =
                        matches!(hit, Some(SetupHit::SearchField | SetupHit::ClearSearch));
                }
                if let Some(hit) = hit {
                    self.setup_focus.at = Some(hit);
                    self.setup_press(hit);
                }
            }
            Screen::Multiplayer => {
                if let Some(hit) = self.net_targets.hit(q) {
                    self.net_focus.at = Some(hit);
                    self.net_menu(hit);
                } else {
                    self.netform.focus = None;
                }
            }
            Screen::Settings => {
                if let Some(hit) = self.settings_targets.hit(q) {
                    self.settings_focus.at = Some(hit);
                    self.settings_click(hit);
                }
            }
            Screen::Room => self.room_click(q),
            Screen::End(_) => {
                if self.end_targets.hit(q).is_some() {
                    self.end_press();
                }
            }
        }
    }

    /// What the client does after a gesture in the game.
    fn session_event(&mut self, e: SessionEvent) {
        match e {
            SessionEvent::Leave => {
                self.session = None;
                self.captions.clear();
                if self.net.is_some() {
                    // Left on purpose: no rejoin (M9 N5).
                    self.keep_rejoin(None);
                }
                self.net_leave();
                self.shell.leave_game();
                if let Some(a) = self.audio.as_mut() {
                    // spec: S-0703 claim 13 (our engine: leaving through the options menu stops the track at once, with no music update or pick; the main menu starts the menu track)
                    a.main_menu_music();
                }
            }
            SessionEvent::Settings => self.open_settings(true),
            SessionEvent::None => {}
        }
    }

    /// A main-menu control pressed (a click or the keyboard).
    fn menu_press(&mut self, hit: MenuHit) {
        let ready = self.install.is_some();
        if hit == MenuHit::Skirmish && !ready {
            return;
        }
        if hit == MenuHit::Settings {
            self.open_settings(false);
            return;
        }
        let rules = self.install.as_ref().map(|i| i.setup.clone());
        match rules {
            Some(r) => {
                let e = self.shell.menu(hit, &r);
                if hit == MenuHit::Skirmish {
                    // A kept search and filters may leave out the map chosen.
                    self.refilter_maps();
                }
                self.event(e);
            }
            None if hit == MenuHit::Install => self.pick_install = true,
            None if hit == MenuHit::Quit => self.quit = true,
            None => {}
        }
    }

    /// The end screen's control pressed.
    fn end_press(&mut self) {
        self.session = None;
        self.captions.clear();
        self.net_leave();
        self.shell.end_done();
        if let Some(a) = self.audio.as_mut() {
            // spec: S-0703 claim 12 (leaving the end screen for the main menu stops the score track at once; the menu track, if any, starts)
            a.main_menu_music();
        }
    }

    /// A key went down or up.
    pub fn key(&mut self, k: Key, down: bool) {
        if down {
            self.gesture();
        }
        // The type-select role ends when its key comes up, wherever the
        // focus is (S-0214 claim 12: held).
        if !down
            && bound_with(&self.settings.bindings, k) == Some(Bound::TypeSelect)
            && let Some(s) = self.session.as_mut()
        {
            s.type_select_key(false);
        }
        // The route-mode key's release commits, wherever the focus is
        // (S-0213 claim 16).
        if !down
            && bound_with(&self.settings.bindings, k) == Some(Bound::RouteMode)
            && let Some(s) = self.session.as_mut()
        {
            s.route_key(false);
        }
        // The make-group key ends when it comes up, wherever the focus is.
        if !down && bound_with(&self.settings.bindings, k) == Some(Bound::MakeGroup) {
            self.group_held = false;
        }
        // Enter presses the open options menu's focused control, or places
        // at the keyboard cell, before it opens a network game's chat.
        if down
            && k == Key::Enter
            && self.shell.screen == Screen::Game
            && !self.typing()
            && self
                .session
                .as_ref()
                .is_some_and(|s| s.paused || s.aiming())
        {
            self.game_key(k, down);
            return;
        }
        // Text fields and the chat take their keys first; a letter typed
        // into one is no command.
        if down && self.net_key(k) {
            return;
        }
        if down && self.picker_key(k) {
            return;
        }
        if self.shell.screen == Screen::Settings {
            if down {
                self.settings_key(k);
            }
            return;
        }
        if matches!(k, Key::Char(_) | Key::Space) && self.typing() {
            return;
        }
        if self.shell.screen != Screen::Game {
            if down && !self.busy {
                self.menu_key(k);
            }
        } else if self.game_key(k, down) {
            return;
        }
        // spec: S-0213 claim 23 (holding Left, Right, Up or Down scrolls the view that way; releasing an arrow ends its contribution)
        let arrow = match k {
            Key::Left => Some(0),
            Key::Right => Some(1),
            Key::Up => Some(2),
            Key::Down => Some(3),
            _ => None,
        };
        if let Some(i) = arrow {
            // spec: S-0213 claim 23 and S-0201 claim 14 (a press or release during an order block is discarded: an accepted contribution stays, a new press adds none)
            let accepted = self.shell.screen != Screen::Game
                || self
                    .session
                    .as_ref()
                    .is_none_or(|s| s.arrow_event_accepted());
            if accepted {
                self.held[i] = down;
            }
            return;
        }
        if !down || self.shell.screen != Screen::Game {
            return;
        }
        let Some(s) = self.session.as_mut() else {
            return;
        };
        if k == Key::Delete {
            s.delete_key();
            return;
        }
        match bound_with(&self.settings.bindings, k) {
            Some(Bound::Options) => {
                s.open_options();
            }
            Some(Bound::Command(c)) => s.key(c),
            Some(Bound::Cheer) if !s.paused => s.play.cheer(),
            Some(Bound::PathReport) => {
                s.save_path_report(self.save_path_report);
            }
            Some(Bound::Mark) => s.mark(),
            Some(Bound::TypeSelect) => s.type_select_key(true),
            Some(Bound::RouteMode) => s.route_key(true),
            Some(Bound::MakeGroup) => self.group_held = self.group_fallback,
            None => {
                // spec: S-0213 claim 22 (the control-group defaults: the digit alone selects the group, Shift with it adds the group to the selection, Ctrl with it makes the group, Alt with it selects the group and centres the view; digits 1 to 9 for groups 1 to 9 and 0 for group 10)
                // design: a digit no other binding takes; with several modifiers Ctrl wins, then Alt, then Shift. On the web the held make-group key plays Ctrl's part (M8 A4 round 5 seams).
                if let Key::Char(c) = k
                    && let Some(g) = c.to_digit(10)
                {
                    let m = self.mods;
                    let cmd = if m.force_fire || self.group_held {
                        GroupCommand::Make
                    } else if m.force_move {
                        GroupCommand::Centre
                    } else if m.force_select {
                        GroupCommand::Add
                    } else {
                        GroupCommand::Select
                    };
                    s.group_key(g as usize, cmd);
                }
            }
            _ => {}
        }
    }

    /// Our own keyboard play in the game (M8 step C3): the options menu's
    /// controls while it is open, else the sidebar and keyboard aiming.
    /// Returns whether the key was taken. None of these keys is one of
    /// S-0213's bindings: letters, digits and Escape keep their meaning,
    /// and an arrow still scrolls as it always does (on the open menu it
    /// also moves the focus) unless Ctrl makes it move the keyboard cell.
    fn game_key(&mut self, k: Key, down: bool) -> bool {
        let shift = self.mods.force_select;
        let ctrl = self.mods.force_fire;
        let arrow = matches!(k, Key::Left | Key::Right | Key::Up | Key::Down);
        if self.typing() || (arrow && !down) {
            // An arrow's release always reaches the scrolling.
            return false;
        }
        let Some(s) = self.session.as_mut() else {
            return false;
        };
        if s.paused {
            let nav = match k {
                Key::Tab if shift => Nav::Prev,
                Key::Tab | Key::Down => Nav::Next,
                Key::Up => Nav::Prev,
                Key::Left => Nav::Left,
                Key::Right => Nav::Right,
                Key::Enter | Key::Space => Nav::Activate,
                _ => return false,
            };
            if down {
                let e = s.pause_key(nav);
                self.session_event(e);
            }
            return !arrow;
        }
        let cell = |dx: i32, dy: i32| SideKey::Cell(dx, dy);
        let side = match k {
            Key::Tab => SideKey::Step(!shift),
            Key::Space => SideKey::Press { shift },
            Key::Backspace => SideKey::Back,
            Key::PageUp => SideKey::Tab(-1),
            Key::PageDown => SideKey::Tab(1),
            Key::Enter if s.aiming() => SideKey::Place,
            Key::Left if ctrl && s.tool_aims() => cell(-1, 0),
            Key::Right if ctrl && s.tool_aims() => cell(1, 0),
            Key::Up if ctrl && s.tool_aims() => cell(0, -1),
            Key::Down if ctrl && s.tool_aims() => cell(0, 1),
            _ => return false,
        };
        if down {
            s.sidebar_key(side);
        }
        true
    }

    /// A key on a screen outside the game: the keyboard focus moves
    /// through its controls and presses them (our own keys, M8 step C3).
    fn menu_key(&mut self, k: Key) {
        let nav = self.nav_of(k);
        if k == Key::Escape {
            self.menu_back();
            return;
        }
        let Some(nav) = nav else {
            return;
        };
        match self.shell.screen {
            Screen::Menu => {
                if let Some(hit) = self.menu_focus.nav(&self.menu_targets, nav) {
                    self.menu_press(hit);
                }
            }
            Screen::Setup => {
                let hit = self.setup_focus.nav(&self.setup_targets, nav);
                self.picker_focus_moved();
                if let Some(hit) = hit {
                    self.setup_press(hit);
                }
            }
            Screen::Room => {
                if nav == Nav::Activate
                    && !self.setup_focus.shown
                    && let Some(n) = self.net.as_mut()
                {
                    // Enter with no control focused opens the chat line.
                    n.typing = true;
                    return;
                }
                if let Some(n) = self.net.as_mut() {
                    n.typing = false;
                }
                if let Some(hit) = self.setup_focus.nav(&self.setup_targets, nav) {
                    self.room_press(hit);
                }
            }
            Screen::Multiplayer => {
                let hit = self.net_focus.nav(&self.net_targets, nav);
                // A text field takes the typing while it has the focus.
                self.netform.focus = match self.net_focus.target(&self.net_targets) {
                    Some(NetHit::Field(f)) => Some(f),
                    _ => None,
                };
                if let Some(hit) = hit {
                    self.net_menu(hit);
                }
            }
            Screen::End(_) => {
                if self.end_focus.nav(&self.end_targets, nav).is_some() {
                    self.end_press();
                }
            }
            Screen::Settings | Screen::Game => {}
        }
    }

    /// The focus step a key takes on a screen of controls (our own keys,
    /// listed on the settings screen as `KEYBOARD_KEYS`).
    pub(super) fn nav_of(&self, k: Key) -> Option<Nav> {
        match k {
            Key::Tab if self.mods.force_select => Some(Nav::Prev),
            Key::Tab | Key::Down => Some(Nav::Next),
            Key::Up => Some(Nav::Prev),
            Key::Left => Some(Nav::Left),
            Key::Right => Some(Nav::Right),
            Key::Enter | Key::Space => Some(Nav::Activate),
            _ => None,
        }
    }

    /// Escape on a screen outside the game: back, as the screen's back
    /// control does (the main menu has none).
    fn menu_back(&mut self) {
        match self.shell.screen {
            Screen::Setup => self.setup_press(SetupHit::Back),
            Screen::Room => self.room_press(SetupHit::Back),
            Screen::Multiplayer => self.net_menu(NetHit::Back),
            Screen::End(_) => self.end_press(),
            Screen::Menu | Screen::Settings | Screen::Game => {}
        }
    }

    /// The window lost or gained the focus (S-0201 claim 9).
    pub fn focus(&mut self, on: bool) {
        self.focused = on;
        if !on {
            self.held = [false; 4];
            self.group_held = false;
            // The key-up of a held type-select key may not come back: the
            // role ends without a tap (our design). A held route-mode key
            // is released, which commits (our design).
            if let Some(s) = self.session.as_mut() {
                s.type_select_cancel();
                s.route_key(false);
            }
        }
    }

    /// Advances the game for a drawn frame at host time `now_ms`: at most
    /// one tick (S-0201 claims 2 to 4), none while paused or without focus
    /// (claims 8 and 9), with that cycle's scrolling (S-0101 claims 12 to
    /// 15); then sound and the end of the game.
    pub fn update(&mut self, now_ms: i64) {
        self.clock = now_ms;
        self.net_pump(now_ms);
        self.screen_music(now_ms);
        // design: map picker (lead's request 2026-11-13): the chosen map's preview is decoded here, not while a frame is drawn
        self.decode_preview();
        if self.shell.screen != Screen::Game {
            return;
        }
        // A network game ticks only as its lockstep session says, also before
        // the session exists (a rejoin waiting for the game sent, M9 N5).
        let networked = self.net.as_ref().is_some_and(|n| n.begin.is_some());
        let Some(s) = self.session.as_mut() else {
            return;
        };
        s.clock_ms = now_ms;
        s.focused = self.focused;
        // spec: S-0213 claim 25 (status lines expire at display updates, also while paused or unfocused)
        s.play.update_messages(now_ms);
        // spec: S-0213 Edge cases (Unfittable status text: an importer warning for the offending text)
        for p in s.play.status_problems.drain(..) {
            log::warn!("status text: {p}");
        }
        // spec: S-0414 claim 23 (revision 16: in a skirmish the active clock pauses while the options menu is open and while the game is frozen, and time without the input focus counts; our engine has no freeze in the client loop) and S-0201 claim 18 (in a network game it stops for neither the menu nor lost focus)
        let active = networked || !s.paused;
        s.play.clock(now_ms, active);
        let ticked = if networked {
            // spec: S-0201 claims 15 and 16 (in a network game neither the options menu nor lost focus stops the ticks; the lockstep paces them, and lost focus slows no one, our engine's departure)
            self.net_tick(now_ms)
        } else if !s.ticking() || !self.focused {
            // spec: S-0201 claims 8 and 9 (no cycle runs while the options menu is open or without focus)
            self.pacer.reset();
            // spec: S-0111 claim 20 (Blocked: the view moving) and S-0101 claims 13 to 15 (no scrolling while the game stands still; #84)
            s.view_still();
            false
        } else if self.pacer.due(now_ms, s.speed) {
            s.step();
            true
        } else {
            false
        };
        let Some(s) = self.session.as_mut() else {
            return;
        };
        if ticked {
            // spec: S-0101 claims 13 to 15 (scrolling is requested once per unfrozen cycle, not per drawn frame; none while the game stands still)
            s.scroll_cycle(now_ms.max(0) as u64, self.held, self.focused);
            if let Some(a) = self.audio.as_mut() {
                a.tick(&s.play.game.world, &s.play.effects, &mut self.unsync);
                // spec: S-0410 claim 15, S-0201 claim 6 (radar-event zoom advances once per drawn frame, one per cycle, frozen or not, with or without radar; none in a stand-still)
                a.radar_frame(&s.play.game.world, s.radar_origin());
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
        if let Some(a) = self.audio.as_mut() {
            let v = s.view();
            a.set_view(SoundView {
                origin: v.origin,
                size: (v.size.0 as i32, v.size.1 as i32),
                // spec: S-0701 F2 (the position's place on screen: S-0101's projection with its height)
                project: glasnost_render::projection::project_lifted,
            });
            let stand = if networked {
                // design: a network game stands still neither for its options menu nor for lost focus (S-0201 claims 15 and 16: the ticks go on), so its audio stays play; S-0701 claim 30 covers a skirmish
                Stand::Play
            } else if !self.focused {
                // spec: S-0701 claim 30 (lost focus is a full stand-still, for music as for sound, also while the options menu is open)
                Stand::Unfocused
            } else if s.paused {
                // spec: S-0701 claim 30 and S-0703 claim 13 (the options menu of a skirmish is a stand-still for sound, while the music plays on with music-only passes)
                Stand::Menu
            } else {
                Stand::Play
            };
            a.frame_in(now_ms, stand, &mut self.unsync);
            // spec: S-0701 claim 26 (each started announcement's caption: its text key when nonempty, otherwise its entry's name)
            for name in a.take_started() {
                if self.settings.captions {
                    self.captions.push(&name, now_ms);
                }
            }
        }
        self.captions.update(now_ms);
        if let Some(end) = s.play.end() {
            self.shell.game_over(end);
            if let Some(a) = self.audio.as_mut() {
                // spec: S-0703 claim 12 and S-0701 claim 30 (our engine: the end screen opens at the end of the cycle that runs the end delay's final tick, after that cycle's audio pass; no music update after it; the score track starts)
                a.game_over_music();
            }
            if networked {
                // The game is over: nothing to rejoin (M9 N5).
                self.keep_rejoin(None);
            }
        }
    }

    /// The frame's draw list, and the images that changed (the battlefield
    /// on the CPU when [`Client::cpu_map`], the radar when it moved on).
    /// The screens are laid out in the interface's logical pixels and
    /// drawn at the interface scale; in the game only the sidebar, the
    /// options menu and the notices over the battlefield are (the
    /// battlefield keeps its own pixels).
    pub fn frame(&mut self) -> (DrawList, FrameImages) {
        let scale = self.ui_scale();
        let size = scale.ui;
        let hover = self.pointer.map(|p| scale.to_ui(p));
        let mut changed = Vec::new();
        let here = std::mem::discriminant(&self.shell.screen);
        if self.last_screen != Some(here) {
            // A new screen starts without keyboard focus.
            self.last_screen = Some(here);
            self.menu_focus = Focus::new();
            self.setup_focus = Focus::new();
            self.net_focus = Focus::new();
            self.settings_focus = Focus::new();
            self.end_focus = Focus::new();
        }
        let list = match &self.shell.screen {
            Screen::Menu => {
                let status = if self.busy {
                    format!("{} (working)", self.status)
                } else {
                    self.status.clone()
                };
                let (mut d, t) =
                    menu::view(size, &status, self.install.is_some(), self.can_quit, hover);
                self.menu_focus.mark(&mut d, &t);
                self.menu_targets = t;
                d
            }
            Screen::Multiplayer => {
                let mut d = self.net_menu_frame(hover);
                self.net_focus.mark(&mut d, &self.net_targets);
                d
            }
            Screen::Settings => self.settings_frame(hover),
            Screen::Room => {
                let mut d = self.room_frame(hover);
                self.setup_focus.mark(&mut d, &self.setup_targets);
                if self.busy {
                    busy_veil(&mut d, size, &self.status);
                }
                d
            }
            Screen::Setup => self.setup_frame(size, hover, &mut changed),
            Screen::Game => {
                // The notices over the battlefield, in logical pixels.
                let mut overlay = DrawList::new();
                self.net_overlay(&mut overlay);
                let map = glasnost_ui::hud::Layout::new(size).map;
                self.captions.draw(&mut overlay, map);
                match self.session.as_mut() {
                    Some(s) => {
                        s.tool_tips = self.settings.tool_tips;
                        s.update_marks();
                        s.update_cursor(&mut self.painter);
                        s.update_status_glyphs(&mut self.painter);
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
                        // The session scales its own interface layer.
                        return (s.draw(overlay), FrameImages { changed });
                    }
                    None => {
                        // Loading the map.
                        let mut d = DrawList::new();
                        busy_veil(&mut d, size, &self.status);
                        d
                    }
                }
            }
            Screen::End(end) => {
                let (mut d, t) = end.view(size, hover);
                self.end_focus.mark(&mut d, &t);
                self.end_targets = t;
                d
            }
        };
        let mut out = DrawList::new();
        out.scaled(scale, list);
        (out, FrameImages { changed })
    }

    /// A new GPU device draws the frames from now on (the first, or one
    /// rebuilt after a loss): it holds none of what was uploaded, so the
    /// palettes, the sprite atlas and the radar image are sent again with
    /// the next frame (design: render (#100)).
    pub fn new_device(&mut self) {
        self.painter.dirty = true;
        self.painter.atlas.touch();
        self.radar_tick = -1;
        self.previews.forget_upload();
    }

    /// Gives the painter an atlas of `side` × `side`, the device's sprite
    /// atlas side, from now on (design: render (#103)). A painter of
    /// another side is replaced: the running game's art is registered in
    /// the new one, and its cursor frames and status glyphs come back as
    /// they are next drawn. Nothing changes when the side is the same.
    pub fn fit_atlas(&mut self, side: u32) {
        self.atlas_side = side;
        if self.painter.atlas.size() == side {
            return;
        }
        log::info!("the sprite atlas is {side} × {side} on this device");
        self.painter = Painter::with_side(side);
        if let Some(s) = self.session.as_mut() {
            s.register_art(&mut self.painter);
        }
    }

    /// The setup values as the screens show them: with the colour-blind
    /// swatches when the person asked for them.
    fn shown_setup_rules(&self, rules: &ui_setup::SetupRules) -> ui_setup::SetupRules {
        let mut r = rules.clone();
        if self.settings.colour_blind {
            r.colours = install::colour_blind_swatches();
        }
        r
    }

    /// The whole frame drawn on the CPU (tests and local snapshots).
    pub fn frame_rgba(&mut self) -> Vec<u8> {
        // The CPU images are this path's own: the shown preview is sent
        // again, whatever a GPU frame got before.
        self.previews.forget_upload();
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
    d.text_centred(
        Rect::new(0, size.1 / 2 - 8, size.0, 16),
        status,
        colour::TEXT,
    );
}

/// What a pressed key does in a running game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    /// Opens the in-game options menu.
    Options,
    /// A key command of S-0213 table T2.
    Command(KeyCommand),
    /// The cheer command.
    Cheer,
    /// Save a pathing report (M8 A4, our own diagnostics).
    PathReport,
    /// Mark the moment for the pathing report (M8 A4).
    Mark,
    /// The type-select role (S-0214 claim 12), held or pressed.
    TypeSelect,
    /// The route-mode key (S-0213 claims 16 and 22).
    RouteMode,
    /// Held with a digit, makes that control group (the web stand-in
    /// for Ctrl+digit; design, M8 A4 round 5 seams).
    MakeGroup,
}

/// The key bindings of a fresh profile (S-0213 claims 22 and 23).
pub fn bound(k: Key) -> Option<Bound> {
    bound_with(&Bindings::default(), k)
}

/// What a pressed key does under the person's bindings (the settings
/// screen's choices; [`bound`] is a fresh profile).
pub fn bound_with(b: &Bindings, k: Key) -> Option<Bound> {
    if let Key::Char(c) = k {
        return b.action(c).map(|a| match a {
            Action::Stop => Bound::Command(KeyCommand::Stop),
            Action::Scatter => Bound::Command(KeyCommand::Scatter),
            Action::Guard => Bound::Command(KeyCommand::Guard),
            Action::Deploy => Bound::Command(KeyCommand::Deploy),
            Action::Cheer => Bound::Cheer,
            Action::PathReport => Bound::PathReport,
            Action::Mark => Bound::Mark,
            Action::TypeSelect => Bound::TypeSelect,
            Action::RouteMode => Bound::RouteMode,
            Action::MakeGroup => Bound::MakeGroup,
        });
    }
    match k {
        // spec: S-0213 claim 23 (Escape opens the in-game options menu, reserved for it; closing keys are not specified, so the menu closes through its return control)
        Key::Escape => Some(Bound::Options),
        // spec: S-0213 claim 22 (fresh-profile bindings: S stop, X scatter, G guard, D deploy, C cheer, acting on key press)
        Key::Char('s') => Some(Bound::Command(KeyCommand::Stop)),
        Key::Char('x') => Some(Bound::Command(KeyCommand::Scatter)),
        Key::Char('g') => Some(Bound::Command(KeyCommand::Guard)),
        Key::Char('d') => Some(Bound::Command(KeyCommand::Deploy)),
        Key::Char('c') => Some(Bound::Cheer),
        _ => None,
    }
}

#[path = "client_net.rs"]
mod net;

#[path = "client_settings.rs"]
mod settings_screen;

#[path = "client_maps.rs"]
mod maps;
pub use maps::PREVIEW_IMAGE;

pub use net::{BUILD, PairReport, test_build};

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "client_net_tests.rs"]
mod net_tests;

#[cfg(test)]
#[path = "client_access_tests.rs"]
mod access_tests;

#[cfg(test)]
#[path = "client_maps_tests.rs"]
mod maps_tests;

#[cfg(test)]
#[path = "client_music_tests.rs"]
mod music_tests;

#[cfg(test)]
#[path = "client_play_notes_tests.rs"]
mod play_notes_tests;

#[cfg(test)]
#[path = "client_watch_tests.rs"]
mod watch_tests;

#[cfg(test)]
#[path = "client_name_tip_tests.rs"]
mod name_tip_tests;

#[cfg(test)]
#[path = "client_sidebar_tip_tests.rs"]
mod sidebar_tip_tests;

#[cfg(test)]
#[path = "client_touch_tests.rs"]
mod touch_tests;

#[cfg(test)]
#[path = "client_display_tests.rs"]
mod display_tests;
