//! The settings screen of the client (M8 step C2): opening it from the main
//! menu or the game's options menu, stepping values, rebinding keys, and
//! keeping the relay address last used. Nothing here reaches the simulation.

use super::{Client, Key};
use crate::settings::{self, ACTIONS, Action, BindError, FIXED_KEYS, KEYBOARD_KEYS, Settings};
use glasnost_audio::output::OutputState;
use glasnost_ui::draw::DrawList;
use glasnost_ui::geom::Point;
use glasnost_ui::settings::{Page, Row, SettingsHit};
use glasnost_ui::shell::Screen;

impl Client {
    /// Shows the settings screen; `from_game` returns to the paused game.
    pub(super) fn open_settings(&mut self, from_game: bool) {
        self.settings_from_game = from_game;
        self.rebinding = None;
        self.settings_note.clear();
        self.shell.screen = Screen::Settings;
    }

    fn close_settings(&mut self) {
        self.rebinding = None;
        self.shell.screen = if std::mem::take(&mut self.settings_from_game) {
            Screen::Game
        } else {
            Screen::Menu
        };
    }

    pub(super) fn settings_click(&mut self, hit: SettingsHit) {
        self.rebinding = None;
        self.settings_note.clear();
        match hit {
            SettingsHit::Step(row, d) => {
                let s = &mut self.settings;
                match row {
                    Row::Sound => s.sound = settings::step_volume(s.sound, d),
                    Row::Voices => s.voices = settings::step_volume(s.voices, d),
                    Row::Music => s.music = settings::step_volume(s.music, d),
                    Row::Scroll => s.step_scroll(d),
                    // design: readable text and display scale (lead's request 2026-11-10): the factor changes at once (`apply_settings`)
                    Row::Display => s.step_display(d),
                    Row::Scale => s.step_scale(d),
                    Row::Captions => s.captions = !s.captions,
                    // spec: S-0111 claim 20 (both options menus change the tooltip option: the main menu's settings and the game's options menu reach this screen) and claim 26 (turning it off hides a tip)
                    Row::ToolTips => s.tool_tips = !s.tool_tips,
                    // design: touch and touchpad scrolling (lead's request 2026-11-10)
                    Row::Touchpad => s.touchpad_scroll = !s.touchpad_scroll,
                    Row::Colours => {
                        s.colour_blind = !s.colour_blind;
                        if self.session.is_some() {
                            self.settings_note =
                                "House colours change from the next game on.".into();
                        }
                    }
                }
                self.settings_changed();
            }
            SettingsHit::Rebind(i) => self.rebinding = Some(i),
            SettingsHit::Reset => {
                // The relay address, a game to rejoin and the map picker's
                // search and filters (kept from its own screen) are not
                // preferences to reset.
                let relay = std::mem::take(&mut self.settings.relay);
                let rejoin = self.settings.rejoin.take();
                let maps = std::mem::take(&mut self.settings.maps);
                self.settings = Settings {
                    relay,
                    rejoin,
                    maps,
                    ..Settings::default()
                };
                self.settings_changed();
            }
            SettingsHit::Back => self.close_settings(),
        }
    }

    pub(super) fn settings_key(&mut self, k: Key) {
        match (self.rebinding, k) {
            (Some(_), Key::Escape) => self.rebinding = None,
            (Some(i), Key::Char(c)) => {
                let Some(a) = ACTIONS.get(i).copied() else {
                    self.rebinding = None;
                    return;
                };
                let mut b = self.settings.bindings;
                match b.rebind(a, c) {
                    Ok(()) => {
                        self.settings.bindings = b;
                        self.rebinding = None;
                        self.settings_note.clear();
                        self.settings_changed();
                    }
                    Err(BindError::Taken(other)) => {
                        // The binding stays open for another try.
                        self.settings_note =
                            format!("{} is already {}.", c.to_ascii_uppercase(), other.name());
                    }
                    Err(BindError::NotBindable) => {
                        self.settings_note = "Use a letter or a digit.".into();
                    }
                }
            }
            (Some(_), _) => {}
            (None, Key::Escape) => self.close_settings(),
            (None, _) => {
                // Our own keys (M8 step C3): the focus moves through the
                // values, the bindings and the buttons.
                if let Some(nav) = self.nav_of(k)
                    && let Some(hit) = self.settings_focus.nav(&self.settings_targets, nav)
                {
                    self.settings_click(hit);
                }
            }
        }
    }

    /// Why nothing is heard, when the sound output is silent: the output's
    /// own reason once it exists, else one known before opening it (a
    /// silent build).
    pub(super) fn no_sound(&self) -> Option<String> {
        match self.audio.as_ref().map(glasnost_audio::Audio::state) {
            Some(OutputState::Silent(why)) => Some(why),
            Some(_) => None,
            None => glasnost_audio::output::known_silent().map(str::to_owned),
        }
    }

    pub(super) fn settings_frame(&mut self, hover: Option<Point>) -> DrawList {
        // design: native audio (the volume rows and the note say when there is no sound)
        let no_sound = self.no_sound();
        let level = |v: u8| match no_sound {
            Some(_) => format!("{v}% (no sound)"),
            None => format!("{v}%"),
        };
        let note = match &no_sound {
            Some(why) if self.settings_note.is_empty() => format!("No sound: {why}."),
            _ => self.settings_note.clone(),
        };
        let s = &self.settings;
        let fits = self.ui_scale().percent;
        let on = |b: bool| if b { "On" } else { "Off" }.to_owned();
        let page = Page {
            values: [
                level(s.sound),
                level(s.voices),
                level(s.music),
                s.scroll_label(),
                self.display().label(s.display_scale),
                if fits == i32::from(s.ui_scale) {
                    format!("{}%", s.ui_scale)
                } else {
                    format!("{}% ({fits}% fits)", s.ui_scale)
                },
                on(s.captions),
                on(s.colour_blind),
                on(s.tool_tips),
                on(s.touchpad_scroll),
            ],
            bindings: ACTIONS
                .iter()
                .map(|a| {
                    (
                        label(*a).to_owned(),
                        s.bindings.key(*a).to_ascii_uppercase().to_string(),
                    )
                })
                .collect(),
            rebinding: self.rebinding,
            fixed: FIXED_KEYS
                .iter()
                .chain(&KEYBOARD_KEYS)
                .map(|(a, k)| ((*a).to_owned(), (*k).to_owned()))
                .collect(),
            note: &note,
        };
        let (mut d, t) = glasnost_ui::settings::view(self.ui_size(), &page, hover);
        self.settings_focus.mark(&mut d, &t);
        self.settings_targets = t;
        d
    }

    /// Remembers the relay address of a connection that was tried.
    pub(super) fn remember_relay(&mut self) {
        let relay = self.netform.relay.trim().to_owned();
        if relay != self.settings.relay {
            self.settings.relay = relay;
            self.settings_changed();
        }
    }
}

fn label(a: Action) -> &'static str {
    match a {
        Action::Stop => "Stop",
        Action::Scatter => "Scatter",
        Action::Guard => "Guard",
        Action::Deploy => "Deploy",
        Action::Cheer => "Cheer",
        Action::RouteMode => "Route mode (hold)",
        Action::TypeSelect => "Select type (tap: extend)",
        Action::PathReport => "Save pathing report",
        Action::Mark => "Mark (pathing report)",
        Action::MakeGroup => "Make a group, web (+digit)",
    }
}
