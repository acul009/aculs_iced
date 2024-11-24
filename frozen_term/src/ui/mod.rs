use iced::{
    keyboard::{self, key::Named},
    widget::{center, focus_next, stack},
    Element, Length, Subscription, Task,
};

use crate::components::ansi_grid::AnsiGrid;

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {}

pub struct UI {
    grid: AnsiGrid,
}

impl UI {
    pub fn start() -> (Self, Task<Message>) {
        let mut grid = AnsiGrid::new(80, 30);

        grid.parse(include_str!("castle")).unwrap();

        (Self { grid }, Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {}
    }

    pub fn view(&self) -> Element<Message> {
        center(self.grid.view())
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
