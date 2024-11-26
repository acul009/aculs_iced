pub mod components;
mod ui;
pub mod widgets;

use ui::UI;

fn main() {
    iced::application("frozen term", UI::update, UI::view)
        .subscription(UI::subscription)
        .theme(|_| iced::Theme::Dark)
        .antialiasing(true)
        .centered()
        .run_with(UI::start)
        .unwrap();
}
