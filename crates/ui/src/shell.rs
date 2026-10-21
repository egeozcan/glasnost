//! The screens and how one leads to the next: main menu → skirmish setup
//! → game → end screen → main menu; or main menu → multiplayer → a
//! network room → game → end screen → main menu (the client drives the
//! network screens, which depend on the relay's answers).

use crate::end::End;
use crate::menu::MenuHit;
use crate::setup::{Lobby, MapEntry, Setup, SetupHit, SetupRules};

/// The screen shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Screen {
    /// The main menu.
    Menu,
    /// The skirmish setup.
    Setup,
    /// The multiplayer screen: name, relay address, room code.
    Multiplayer,
    /// The settings screen.
    Settings,
    /// A network room: the setup screen of a room on a relay.
    Room,
    /// A game is running (or paused in its options menu).
    Game,
    /// The end screen of the last game.
    End(End),
}

/// What the client must do after an input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// Create the game of this lobby.
    StartGame(Lobby),
    /// Ask for the game folder again.
    PickInstall,
    /// Leave the program.
    Quit,
}

/// The shell's state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shell {
    /// The screen shown.
    pub screen: Screen,
    /// The setup screen's state, kept between games.
    pub setup: Option<Setup>,
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl Shell {
    /// The shell at the main menu.
    pub fn new() -> Self {
        Self {
            screen: Screen::Menu,
            setup: None,
        }
    }

    /// A click on a main-menu target.
    pub fn menu(&mut self, hit: MenuHit, rules: &SetupRules) -> Option<Event> {
        match hit {
            MenuHit::Skirmish => {
                if self.setup.is_none() {
                    self.setup = Some(Setup::new(rules));
                }
                self.screen = Screen::Setup;
                None
            }
            MenuHit::Multiplayer => {
                self.screen = Screen::Multiplayer;
                None
            }
            MenuHit::Settings => {
                self.screen = Screen::Settings;
                None
            }
            MenuHit::Install => Some(Event::PickInstall),
            MenuHit::Quit => Some(Event::Quit),
        }
    }

    /// A click on a setup target.
    pub fn setup(&mut self, hit: SetupHit, rules: &SetupRules, maps: &[MapEntry]) -> Option<Event> {
        let s = self.setup.get_or_insert_with(|| Setup::new(rules));
        match hit {
            SetupHit::Back => {
                self.screen = Screen::Menu;
                None
            }
            SetupHit::Start_ => match s.lobby(rules, maps) {
                Ok(l) => {
                    self.screen = Screen::Game;
                    Some(Event::StartGame(l))
                }
                Err(_) => None,
            },
            h => {
                s.apply(&h, rules, maps);
                None
            }
        }
    }

    /// The client could not create the game: back to the setup.
    pub fn start_failed(&mut self) {
        self.screen = Screen::Setup;
    }

    /// The game is over (S-0414): show the end screen.
    pub fn game_over(&mut self, end: End) {
        self.screen = Screen::End(end);
    }

    /// The person left the game from its options menu.
    pub fn leave_game(&mut self) {
        self.screen = Screen::Menu;
    }

    /// The end screen's button.
    pub fn end_done(&mut self) {
        self.screen = Screen::Menu;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::SlotKind;

    fn rules() -> SetupRules {
        SetupRules {
            money: 1000,
            money_range: (500, 2000, 100),
            units: 2,
            units_range: (1, 4),
            bases: true,
            flags: [false; 5],
            game_speed: 6,
            countries: vec!["Alphaland".into()],
            colours: vec![[1, 2, 3, 255]; 8],
        }
    }

    #[test]
    fn menu_setup_game_end_menu() {
        let r = rules();
        let maps = vec![MapEntry {
            label: "m".into(),
            starts: 2,
        }];
        let mut s = Shell::new();
        assert_eq!(s.screen, Screen::Menu);
        assert_eq!(s.menu(MenuHit::Skirmish, &r), None);
        assert_eq!(s.screen, Screen::Setup);
        assert_eq!(s.setup(SetupHit::Credits(1), &r, &maps), None);
        assert_eq!(s.setup.as_ref().unwrap().credits, 1100);
        let Some(Event::StartGame(l)) = s.setup(SetupHit::Start_, &r, &maps) else {
            panic!("a game starts");
        };
        assert_eq!(l.settings.credits, Some(1100));
        assert_eq!(s.screen, Screen::Game);
        s.game_over(End {
            ticks: 1,
            units: 1,
            rows: Vec::new(),
        });
        assert!(matches!(s.screen, Screen::End(_)));
        s.end_done();
        assert_eq!(s.screen, Screen::Menu);
        // The setup is kept for the next game.
        s.menu(MenuHit::Skirmish, &r);
        assert_eq!(s.setup.as_ref().unwrap().credits, 1100);
        s.setup(SetupHit::Back, &r, &maps);
        assert_eq!(s.screen, Screen::Menu);
        assert_eq!(s.menu(MenuHit::Multiplayer, &r), None);
        assert_eq!(s.screen, Screen::Multiplayer);
        assert_eq!(s.menu(MenuHit::Quit, &r), Some(Event::Quit));
        assert_eq!(s.menu(MenuHit::Install, &r), Some(Event::PickInstall));
    }

    #[test]
    fn an_invalid_setup_does_not_start() {
        let r = rules();
        let maps = vec![MapEntry {
            label: "m".into(),
            starts: 2,
        }];
        let mut s = Shell::new();
        s.menu(MenuHit::Skirmish, &r);
        s.setup.as_mut().unwrap().slots[1].kind = SlotKind::Closed;
        assert_eq!(s.setup(SetupHit::Start_, &r, &maps), None);
        assert_eq!(s.screen, Screen::Setup);
    }
}
