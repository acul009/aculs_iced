use core::str;
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use iced::{
    futures::SinkExt,
    keyboard::{self, Key, Modifiers},
    stream::channel,
    Element, Subscription, Task,
};
use portable_pty::{Child, PtyPair, PtySize};
use tokio::task::{spawn_blocking, JoinHandle};

use crate::components::{ansi_grid::AnsiGrid, terminal::Terminal};

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    TerminalOutput(Vec<u8>),
    KeyPress(Key, Modifiers),
    Noop,
}

pub struct UI {
    term: Terminal,
    term_cols: u16,
    term_rows: u16,
    child: Box<dyn Child + Send + Sync>,
    copy_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    pty: PtyPair,
}

impl Drop for UI {
    fn drop(&mut self) {
        println!("Dropping UI");
        self.child.kill().unwrap();
        if let Some(handle) = self.copy_handle.lock().unwrap().deref() {
            handle.abort();
        }
    }
}

impl UI {
    pub fn start() -> (Self, Task<Message>) {
        // let grid = AnsiGrid::new(120, 40);
        let cols = 80;
        let rows = 25;

        let command = portable_pty::CommandBuilder::new("fish");

        let pty = portable_pty::native_pty_system()
            .openpty(PtySize {
                cols,
                rows,
                ..Default::default()
            })
            .unwrap();

        let child = pty.slave.spawn_command(command).unwrap();

        let writer = pty.master.take_writer().unwrap();

        let term = Terminal::new(rows, cols, writer);

        (
            Self {
                // grid,
                term,
                pty,
                child,
                term_cols: cols,
                term_rows: rows,
                copy_handle: Arc::new(Mutex::new(None)),
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TerminalOutput(output) => {
                let str = str::from_utf8(&output).unwrap();
                print!("{}", str);
                self.term.advance_bytes(output);
                // self.grid.parse(&output).unwrap();
                Task::none()
            }
            Message::KeyPress(key, modifiers) => {
                self.term.key_press(key, modifiers);
                // let writer = self.writer.clone();
                // Task::future(async move {
                //     let mut writer = writer.lock().await;
                //     if let Key::Character(c) = key {
                //         writer.write_all(c.as_bytes()).await.unwrap();
                //     }

                //     Message::Noop
                // })
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
        let mut reader = self.pty.master.try_clone_reader().unwrap();
        let copy_handle = self.copy_handle.clone();
        Subscription::batch(vec![
            keyboard::on_key_press(|key, modifiers| Some(Message::KeyPress(key, modifiers))),
            Subscription::run_with_id(
                1,
                channel(1, |mut output| async move {
                    let (send, mut recv) = tokio::sync::mpsc::unbounded_channel();

                    let handle = spawn_blocking(move || {
                        let mut buf = vec![0u8; 1024];
                        loop {
                            let read = reader.read(&mut buf).unwrap();
                            if read == 0 {
                                println!("EOF");
                                break;
                            }
                            send.send(buf[..read].to_vec()).unwrap();
                        }
                    });

                    {
                        *copy_handle.lock().unwrap() = Some(handle);
                    }

                    while let Some(s) = recv.recv().await {
                        output.send(Message::TerminalOutput(s)).await.unwrap();
                    }
                }),
            ),
        ])
    }
}
