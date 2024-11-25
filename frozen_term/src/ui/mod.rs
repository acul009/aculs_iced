use std::{process::Stdio, sync::Arc};

use iced::{
    advanced::graphics::text::font_system,
    futures::SinkExt,
    keyboard::{self, key::Named, Key, Modifiers},
    stream::channel,
    widget::{center, focus_next, rich_text, span, stack, themer},
    Element, Length, Subscription, Task,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{Child, ChildStdin, ChildStdout, Command},
};

use crate::components::{ansi_grid::AnsiGrid, terminal::Terminal};

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    TerminalOutput(String),
    KeyPress(Key, Modifiers),
    Noop,
}

pub struct UI {
    // grid: AnsiGrid,
    term: Terminal,
    shell: Child,
    reader: Arc<tokio::sync::Mutex<BufReader<ChildStdout>>>,
    // writer: Arc<tokio::sync::Mutex<BufWriter<ChildStdin>>>,
}

impl UI {
    pub fn start() -> (Self, Task<Message>) {
        // let grid = AnsiGrid::new(120, 40);

        let mut shell = Command::new("fish")
            // .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let reader = Arc::new(tokio::sync::Mutex::new(BufReader::new(
            shell.stdout.take().unwrap(),
        )));

        // let writer = Arc::new(tokio::sync::Mutex::new(BufWriter::new(
        //     shell.stdin.take().unwrap(),
        // )));

        let mut term = Terminal::new();

        term.advance_bytes(include_bytes!("castle"));

        (
            Self {
                // grid,
                term,
                shell,
                reader,
                // writer,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TerminalOutput(output) => {
                // self.term.advance_bytes(output);
                // self.grid.parse(&output).unwrap();
                Task::none()
            }
            Message::KeyPress(key, modifiers) => {
                // let writer = self.writer.clone();
                // Task::future(async move {
                //     let mut writer = writer.lock().await;
                //     if let Key::Character(c) = key {
                //         writer.write_all(c.as_bytes()).await.unwrap();
                //     }

                //     Message::Noop
                // })
                self.term.print();
                Task::none()
            }
            Message::Noop => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        self.term.view().into()
        // center(self.grid.view())
        //     .width(Length::Fill)
        //     .height(Length::Fill)
        //     .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let reader = self.reader.clone();
        Subscription::batch(vec![
            keyboard::on_key_press(|key, modifiers| Some(Message::KeyPress(key, modifiers))),
            Subscription::run_with_id(
                1,
                channel(1, |mut output| async move {
                    let mut reader = reader.lock().await;
                    let mut buf = vec![0u8; 1024];
                    loop {
                        let read = reader.read(&mut buf).await.unwrap();
                        if read == 0 {
                            continue;
                        }
                        let s = String::from_utf8(buf[..read].to_vec()).unwrap();
                        output.send(Message::TerminalOutput(s)).await.unwrap();
                    }
                }),
            ),
        ])
    }
}
