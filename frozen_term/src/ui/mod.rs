use iced::{
    keyboard::{self, key::Named}, widget::{focus_next, stack}, Element, Subscription, Task
};

use crate::components::ansi_grid::AnsiGrid;


/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
}

pub struct UI {
    grid: AnsiGrid
}

impl UI {
    pub fn start() -> (Self, Task<Message>) {
        let grid = AnsiGrid::new(80, 30);

        (
            Self {
                grid
            },
            Task::none()
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
        }
    }

    pub fn view(&self) -> Element<Message> {
        self.grid.view()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
