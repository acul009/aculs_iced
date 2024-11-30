use std::sync::Arc;

use iced::{
    advanced::Widget,
    keyboard::key,
    widget::{container, rich_text, text},
    Border, Element, Length, Shadow,
};
use wezterm_term::{
    color::{ColorAttribute, ColorPalette},
    TerminalConfiguration, TerminalSize,
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
        &self,
    ) -> impl Into<Element<'a, Message, Theme, Renderer>>
    where
        Renderer: iced::advanced::text::Renderer<Font = iced::Font> + 'static,
        Message: Clone + 'static,
        Theme: iced::widget::text::Catalog + 'static,
        Theme: iced::widget::container::Catalog,
        <Theme as iced::widget::container::Catalog>::Class<'a>:
            From<iced::widget::container::StyleFn<'a, Theme>>,
    {
        let mut basic = String::new();
        let screen = self.term.screen();
        let lines =
            screen.lines_in_phys_range(screen.phys_range(&(0..screen.physical_rows as i64)));

        for line in lines {
            basic.push_str(&line.as_str());
            basic.push('\n');
        }

        return text(basic).font(iced::Font::MONOSPACE);

        // let screen = self.term.screen();
        // let palette = self.term.palette();

        // let lines =
        //     screen.lines_in_phys_range(screen.phys_range(&(0..screen.physical_rows as i64)));

        // let width = screen.physical_cols;

        // let mut spans =
        //     Vec::with_capacity(self.term.screen().physical_cols * self.term.screen().physical_rows);

        // // screen.for_each_phys_line(|index, line| {
        // for line in lines {
        //     let mut lines_found = 0;

        //     for cell in line.visible_cells() {
        //         lines_found += 1;
        //         let style = cell.attrs();

        //         let span = iced::widget::span(cell.str().to_string())
        //             .color(get_color(style.foreground(), &palette, true))
        //             .background(get_color(style.background(), &palette, false));
        //         spans.push(span);
        //     }

        //     spans.push(iced::widget::span(" ".repeat(width - lines_found)));
        //     spans.push(iced::widget::span("\n"));
        // }
        // // });

        // let (r, g, b, a) = palette.background.to_tuple_rgba();

        // container(rich_text(spans).font(iced::Font::MONOSPACE)).style(move |theme| {
        //     container::Style {
        //         text_color: None,
        //         background: Some(iced::Color::from_rgba(r, g, b, a).into()),
        //         border: Border::default().color(iced::Color::from_rgb(1.0, 1.0, 1.0)),
        //         shadow: Shadow::default(),
        //     }
        // })
    }

    pub fn key_press(&mut self, key: iced::keyboard::Key, modifiers: iced::keyboard::Modifiers) {
        if let Some((key, modifiers)) = transform_key(key, modifiers) {
            self.term.key_down(key, modifiers).unwrap();
        }
    }

    pub fn print(&self) {
        let screen = self.term.screen();

        screen.for_each_phys_line(|size, line| {
            println!("{}", line.as_str());
        });
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
            key::Named::ArrowLeft => Some(wezterm_term::KeyCode::LeftArrow),
            key::Named::ArrowRight => Some(wezterm_term::KeyCode::RightArrow),
            key::Named::ArrowUp => Some(wezterm_term::KeyCode::UpArrow),
            key::Named::ArrowDown => Some(wezterm_term::KeyCode::DownArrow),
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

fn get_color(color: ColorAttribute, palette: &ColorPalette, foreground: bool) -> iced::Color {
    match color {
        ColorAttribute::TrueColorWithPaletteFallback(srgba_tuple, _)
        | ColorAttribute::TrueColorWithDefaultFallback(srgba_tuple) => {
            let (r, g, b, a) = srgba_tuple.to_tuple_rgba();
            iced::Color::from_rgba(r, g, b, a)
        }
        ColorAttribute::PaletteIndex(index) => {
            let (r, g, b, a) = palette.colors.0[index as usize].to_tuple_rgba();
            iced::Color::from_rgba(r, g, b, a)
        }
        ColorAttribute::Default => {
            let color = match foreground {
                true => palette.foreground,
                false => palette.background,
            };
            let (r, g, b, a) = color.to_tuple_rgba();
            iced::Color::from_rgba(r, g, b, a)
        }
    }
}

pub struct TerminalWidget<'a> {
    terminal: &'a Terminal,
}

impl<'a> TerminalWidget<'a> {
    pub fn new(terminal: &'a Terminal) -> TerminalWidget<'a> {
        TerminalWidget { terminal }
    }
}

impl<'a, Message, Theme, Renderer: iced::advanced::Renderer> Widget<Message, Theme, Renderer>
    for TerminalWidget<'a>
{
    fn size(&self) -> iced::Size<Length> {
        todo!()
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        todo!()
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        todo!()
    }
}
