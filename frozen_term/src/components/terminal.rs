use core::f32;
use std::{
    cmp::min_by,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

use iced::{
    advanced::{layout::Node, renderer::Quad, Text},
    alignment::{Horizontal, Vertical},
    border,
    keyboard::key,
    widget::{
        container, responsive,
        text::{self, LineHeight, Shaping, Wrapping},
        Column, Row,
    },
    Color, Element, Length, Shadow, Size, Vector,
};
use wezterm_term::{
    color::{ColorAttribute, ColorPalette},
    Line, TerminalConfiguration, TerminalSize,
};

pub struct Terminal {
    term: Mutex<wezterm_term::Terminal>,
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

        Self {
            term: Mutex::new(term),
        }
    }

    pub fn advance_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) {
        self.term.lock().unwrap().advance_bytes(bytes);
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
        Element::new(TerminalWidget::new(&self))
        // responsive(move |size| {
        //     let cols = size.width / 8.0;
        //     let rows = size.height / 16.0;
        //     let mut term = self.term.lock().unwrap();
        //     term.resize(TerminalSize {
        //         rows: rows as usize,
        //         cols: cols as usize,
        //         pixel_height: size.height as usize,
        //         pixel_width: size.width as usize,
        //         ..Default::default()
        //     });
        //     let screen = term.screen();
        //     let palette = Arc::new(term.palette());
        //     let lines =
        //         screen.lines_in_phys_range(screen.phys_range(&(0..screen.physical_rows as i64)));

        //     let mut col = Column::new();

        //     for line in lines {
        //         let row = iced::widget::lazy(LineWrapper(line, palette.clone()), |line_wrapper| {
        //             let line = &line_wrapper.0;
        //             let palette = &line_wrapper.1;

        //             let mut row = Row::new();

        //             for cell in line.visible_cells() {
        //                 let foreground = get_color(cell.attrs().foreground(), &palette);
        //                 let background = get_color(cell.attrs().background(), &palette);

        //                 let txt = text(cell.str().to_string())
        //                     .color_maybe(foreground)
        //                     .font(iced::Font::MONOSPACE);

        //                 match background {
        //                     Some(background) => {
        //                         row = row.push(container(txt).style(move |_| container::Style {
        //                             text_color: foreground,
        //                             background: Some(background.into()),
        //                             border: border::width(0),
        //                             shadow: Shadow::default(),
        //                         }));
        //                     }
        //                     None => {
        //                         row = row.push(txt);
        //                     }
        //                 }
        //             }

        //             row
        //         });

        //         col = col.push(row);
        //     }

        //     col.into()
        // })
    }

    pub fn key_press(&mut self, key: iced::keyboard::Key, modifiers: iced::keyboard::Modifiers) {
        if let Some((key, modifiers)) = transform_key(key, modifiers) {
            self.term.lock().unwrap().key_down(key, modifiers).unwrap();
        }
    }

    pub fn print(&self) {
        let term = self.term.lock().unwrap();
        let screen = term.screen();

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

pub struct TerminalWidget<'a> {
    term: &'a Terminal,
}

impl<'a> TerminalWidget<'a> {
    pub fn new(term: &'a Terminal) -> Self {
        Self { term }
    }
}

impl<'a, Message, Theme, Renderer> iced::advanced::widget::Widget<Message, Theme, Renderer>
    for TerminalWidget<'a>
where
    Renderer: iced::advanced::text::Renderer,
{
    fn size(&self) -> iced::Size<iced::Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        Node::new(limits.max())
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
        let Some(bounds) = layout.bounds().intersection(viewport) else {
            return;
        };

        let text = Text {
            content: "Test\nTest2\nTest3".to_string(),
            bounds: bounds.size(),
            size: renderer.default_size(),
            line_height: LineHeight::default(),
            font: renderer.default_font(),
            horizontal_alignment: Horizontal::Left,
            vertical_alignment: Vertical::Top,
            shaping: Shaping::default(),
            wrapping: Wrapping::default(),
        };

        let mut text_bounds = bounds;
        text_bounds.height = min_by(text_bounds.height, 21.0, f32::total_cmp);

        renderer.fill_quad(
            Quad {
                bounds: bounds,
                ..Default::default()
            },
            Color::from_rgb(1.0, 0.0, 0.0),
        );
        renderer.fill_text(text, bounds.position(), Color::WHITE, bounds);
    }
}
