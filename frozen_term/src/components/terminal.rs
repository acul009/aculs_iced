use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use iced::{
    border,
    keyboard::key,
    widget::{container, text, Column, Row},
    Element, Shadow,
};
use wezterm_term::{
    color::{ColorAttribute, ColorPalette},
    Line, TerminalConfiguration, TerminalSize,
};

pub struct Terminal {
    term: wezterm_term::Terminal,
}

#[derive(Debug)]
pub struct Config {}

impl TerminalConfiguration for Config {
    fn color_palette(&self) -> wezterm_term::color::ColorPalette {
        ColorPalette::default()
    }
}

impl Terminal {
    pub fn new(rows: u16, cols: u16, writer: Box<dyn std::io::Write + Send>) -> Self {
        let size = TerminalSize {
            rows: rows as usize,
            cols: cols as usize,
            ..Default::default()
        };

        let config = Config {};

        let term =
            wezterm_term::Terminal::new(size, Arc::new(config), "frozen_term", "0.1", writer);

        Self { term }
    }

    pub fn advance_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) {
        self.term.advance_bytes(bytes);
    }

    pub fn view<'a, Message, Theme, Renderer>(
        &'a self,
    ) -> impl Into<Element<'a, Message, Theme, Renderer>>
    where
        Renderer: iced::advanced::text::Renderer<Font = iced::Font> + 'static,
        Message: Clone + 'static,
        Theme: iced::widget::text::Catalog + 'static,
        Theme: iced::widget::container::Catalog,
        <Theme as iced::widget::text::Catalog>::Class<'static>:
            From<iced::widget::text::StyleFn<'static, Theme>>,
        <Theme as iced::widget::container::Catalog>::Class<'static>:
            From<iced::widget::container::StyleFn<'static, Theme>>,
    {
        // Element::new(TerminalWidget::new(&self))
        // let mut basic = String::new();
        let screen = self.term.screen();
        let palette = Arc::new(self.term.palette());
        let lines =
            screen.lines_in_phys_range(screen.phys_range(&(0..screen.physical_rows as i64)));

        let mut col = Column::new();

        for line in lines {
            let row = iced::widget::lazy(LineWrapper(line, palette.clone()), |line_wrapper| {
                let line = &line_wrapper.0;
                let palette = &line_wrapper.1;

                let mut row = Row::new();

                for cell in line.visible_cells() {
                    let foreground = get_color(cell.attrs().foreground(), &palette);
                    let background = get_color(cell.attrs().background(), &palette);

                    let txt = text(cell.str().to_string())
                        .color_maybe(foreground)
                        .font(iced::Font::MONOSPACE);

                    match background {
                        Some(background) => {
                            row = row.push(container(txt).style(move |_| container::Style {
                                text_color: foreground,
                                background: Some(background.into()),
                                border: border::width(0),
                                shadow: Shadow::default(),
                            }));
                        }
                        None => {
                            row = row.push(txt);
                        }
                    }
                }

                row
            });

            col = col.push(row);
        }

        return col;
    }

    pub fn key_press(&mut self, key: iced::keyboard::Key, modifiers: iced::keyboard::Modifiers) {
        if let Some((key, modifiers)) = transform_key(key, modifiers) {
            self.term.key_down(key, modifiers).unwrap();
        }
    }

    pub fn print(&self) {
        let screen = self.term.screen();

        screen.for_each_phys_line(|_, line| {
            println!("{}", line.as_str());
        });
    }
}

struct LineWrapper(Line, Arc<ColorPalette>);

impl Hash for LineWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.current_seqno().hash(state);
    }
}

fn transform_key(
    key: iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Option<(wezterm_term::KeyCode, wezterm_term::KeyModifiers)> {
    let wez_key = match key {
        iced::keyboard::Key::Character(c) => {
            let c = c.chars().next().unwrap();
            Some(wezterm_term::KeyCode::Char(c))
        }
        iced::keyboard::Key::Named(named) => match named {
            key::Named::Enter => Some(wezterm_term::KeyCode::Enter),
            key::Named::Space => Some(wezterm_term::KeyCode::Char(' ')),
            key::Named::Backspace => Some(wezterm_term::KeyCode::Backspace),
            key::Named::Delete => Some(wezterm_term::KeyCode::Delete),
            key::Named::ArrowLeft => Some(wezterm_term::KeyCode::LeftArrow),
            key::Named::ArrowRight => Some(wezterm_term::KeyCode::RightArrow),
            key::Named::ArrowUp => Some(wezterm_term::KeyCode::UpArrow),
            key::Named::ArrowDown => Some(wezterm_term::KeyCode::DownArrow),
            key::Named::Tab => Some(wezterm_term::KeyCode::Tab),
            key::Named::Escape => Some(wezterm_term::KeyCode::Escape),
            key::Named::F1 => Some(wezterm_term::KeyCode::Function(1)),
            key::Named::F2 => Some(wezterm_term::KeyCode::Function(2)),
            key::Named::F3 => Some(wezterm_term::KeyCode::Function(3)),
            key::Named::F4 => Some(wezterm_term::KeyCode::Function(4)),
            key::Named::F5 => Some(wezterm_term::KeyCode::Function(5)),
            key::Named::F6 => Some(wezterm_term::KeyCode::Function(6)),
            key::Named::F7 => Some(wezterm_term::KeyCode::Function(7)),
            key::Named::F8 => Some(wezterm_term::KeyCode::Function(8)),
            key::Named::F9 => Some(wezterm_term::KeyCode::Function(9)),
            key::Named::F10 => Some(wezterm_term::KeyCode::Function(10)),
            key::Named::F11 => Some(wezterm_term::KeyCode::Function(11)),
            key::Named::F12 => Some(wezterm_term::KeyCode::Function(12)),
            _ => None,
        },
        _ => None,
    };

    match wez_key {
        None => None,
        Some(key) => {
            let mut wez_modifiers = wezterm_term::KeyModifiers::empty();

            if modifiers.shift() {
                wez_modifiers |= wezterm_term::KeyModifiers::SHIFT;
            }
            if modifiers.alt() {
                wez_modifiers |= wezterm_term::KeyModifiers::ALT;
            }
            if modifiers.control() {
                wez_modifiers |= wezterm_term::KeyModifiers::CTRL;
            }
            if modifiers.logo() {
                wez_modifiers |= wezterm_term::KeyModifiers::SUPER;
            }

            Some((key, wez_modifiers))
        }
    }
}

fn get_color(color: ColorAttribute, palette: &ColorPalette) -> Option<iced::Color> {
    match color {
        ColorAttribute::TrueColorWithPaletteFallback(srgba_tuple, _)
        | ColorAttribute::TrueColorWithDefaultFallback(srgba_tuple) => {
            let (r, g, b, a) = srgba_tuple.to_tuple_rgba();
            Some(iced::Color::from_rgba(r, g, b, a))
        }
        ColorAttribute::PaletteIndex(index) => {
            let (r, g, b, a) = palette.colors.0[index as usize].to_tuple_rgba();
            Some(iced::Color::from_rgba(r, g, b, a))
        }
        ColorAttribute::Default => None,
    }
}
