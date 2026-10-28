//! A picture of the settings screen on the CPU path, with synthetic data
//! only. Writes `settings.png` into the folder named by
//! `GLASNOST_SHOT_DIR` when it is set; otherwise it only checks that the
//! screen draws.

use glasnost_app::game::client::Client;
use glasnost_app::game::play::Button;
use glasnost_app::mapview::png;
use glasnost_sim::targeting::Modifiers;
use glasnost_ui::geom::Point;
use glasnost_ui::menu::MenuHit;
use glasnost_ui::shell::Screen;

#[test]
fn the_settings_screen_draws() {
    let mut c = Client::new();
    let _ = c.frame_rgba();
    let r = c.menu_targets.rect_of(&MenuHit::Settings).unwrap();
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
    assert_eq!(c.shell.screen, Screen::Settings);
    let rgba = c.frame_rgba();
    assert_eq!(rgba.len(), (c.size.0 * c.size.1 * 4) as usize);
    if let Ok(dir) = std::env::var("GLASNOST_SHOT_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let path = std::path::Path::new(&dir).join("settings.png");
        std::fs::write(path, png::png(c.size.0 as u32, c.size.1 as u32, &rgba)).unwrap();
        // The largest interface scale on a large window (fitted to it).
        c.settings.ui_scale = 300;
        c.resize((2560, 1440));
        let rgba = c.frame_rgba();
        let path = std::path::Path::new(&dir).join("settings-300.png");
        std::fs::write(path, png::png(c.size.0 as u32, c.size.1 as u32, &rgba)).unwrap();
    }
}
