use core::f32;
use std::{
    cmp::min_by,
    fmt::format,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

use iced::{
    Color, Element, Length, Shadow, Size, Vector,
    advanced::{
        Shell, Text,
        graphics::{core::widget, text::paragraph},
        layout::Node,
        renderer::Quad,
        text::{Paragraph, Renderer, paragraph::Plain},
    },
    alignment::{Horizontal, Vertical},
    border,
    keyboard::key,
    widget::{
        Column, Row, container, responsive,
        text::{self, LineHeight, Shaping, Wrapping},
    },
};
use wezterm_term::{
    CellAttributes, Line, TerminalConfiguration,
    color::{ColorAttribute, ColorPalette},
};

pub use wezterm_term::TerminalSize;

pub struct Terminal<Message> {
    term: wezterm_term::Terminal,
    on_resize: Box<dyn Fn(TerminalSize) -> Message>,
}

#[derive(Debug)]
pub struct Config {}

impl TerminalConfiguration for Config {
    fn color_palette(&self) -> wezterm_term::color::ColorPalette {
        ColorPalette::default()
    }
}

impl<Message> Terminal<Message> {
    pub fn new(
        rows: u16,
        cols: u16,
        writer: Box<dyn std::io::Write + Send>,
        on_resize: impl Fn(TerminalSize) -> Message + 'static,
    ) -> Self {
        let size = TerminalSize {
            rows: rows as usize,
            cols: cols as usize,
            ..Default::default()
        };

        let config = Config {};

        let term =
            wezterm_term::Terminal::new(size, Arc::new(config), "frozen_term", "0.1", writer);

        Self {
            term,
            on_resize: Box::new(on_resize),
        }
    }

    pub fn advance_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) {
        self.term.advance_bytes(bytes);
    }

    pub fn key_press(&mut self, key: iced::keyboard::Key, modifiers: iced::keyboard::Modifiers) {
        if let Some((key, modifiers)) = transform_key(key, modifiers) {
            self.term.key_down(key, modifiers).unwrap();
        }
    }

    pub fn resize(&mut self, size: TerminalSize) {
        self.term.resize(size)
    }

    pub fn view<'a, Theme, Renderer>(&'a self) -> impl Into<Element<'a, Message, Theme, Renderer>>
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
        Element::new(TerminalWidget::new(self, iced::Font::MONOSPACE))
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

    pub fn print(&mut self) {
        let term = &self.term;
        let screen = term.screen();

        screen.for_each_phys_line(|_, line| {
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

pub struct TerminalWidget<'a, R: iced::advanced::text::Renderer, Message> {
    term: &'a Terminal<Message>,
    font: R::Font,
}

impl<'a, R, Message> TerminalWidget<'a, R, Message>
where
    R: iced::advanced::text::Renderer,
{
    pub fn new(term: &'a Terminal<Message>, font: impl Into<R::Font>) -> Self {
        Self {
            term,
            font: font.into(),
        }
    }
}

struct TerminalWidgetState<R: Renderer> {
    paragraph: R::Paragraph,
    spans: Vec<iced::advanced::text::Span<'static, (), R::Font>>,
}

impl<Message, Theme, Renderer> iced::advanced::widget::Widget<Message, Theme, Renderer>
    for TerminalWidget<'_, Renderer, Message>
where
    Renderer: iced::advanced::text::Renderer,
    Renderer: 'static,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<TerminalWidgetState<Renderer>>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(TerminalWidgetState::<Renderer> {
            paragraph: Renderer::Paragraph::default(),
            spans: Vec::new(),
        })
    }

    fn size(&self) -> iced::Size<iced::Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let state = tree.state.downcast_mut::<TerminalWidgetState<Renderer>>();
        let term = &self.term.term;
        let screen = term.screen();

        let range = screen.phys_range(&(0..screen.physical_rows as i64));
        let term_lines = screen.lines_in_phys_range(range);

        let mut current_text = String::new();
        let mut current_attrs = CellAttributes::default();
        state.spans.clear();

        let palette = term.palette();

        for line in term_lines {
            for cell in line.visible_cells() {
                if cell.attrs() != &current_attrs {
                    if !current_text.is_empty() {
                        let foreground = get_color(current_attrs.foreground(), &palette);
                        let background = get_color(current_attrs.background(), &palette);

                        let span = iced::advanced::text::Span::new(current_text.clone())
                            .color_maybe(foreground)
                            .background_maybe(background);

                        state.spans.push(span);
                        current_text.clear();
                    }
                    current_attrs = cell.attrs().clone();
                }

                current_text.push_str(cell.str());
            }
            current_text.push('\n');
        }

        if current_text.len() > 1 {
            let foreground = get_color(current_attrs.foreground(), &palette);
            let background = get_color(current_attrs.background(), &palette);

            let span = iced::advanced::text::Span::new(current_text)
                .color_maybe(foreground)
                .background_maybe(background);

            state.spans.push(span);
        }

        // for (index, line) in term_lines.iter().enumerate() {
        //     let span = iced::advanced::text::Span::new(line.as_str());

        //     if let Some(lines) = state.spans.get_mut(index) {
        //         *lines = span;
        //     } else {
        //         state.spans.push(span);
        //     }
        // }

        let text = Text {
            content: state.spans.as_ref(),
            bounds: limits.max(),
            size: renderer.default_size(),
            line_height: LineHeight::default(),
            font: self.font,
            horizontal_alignment: Horizontal::Left,
            vertical_alignment: Vertical::Top,
            shaping: Shaping::Advanced,
            wrapping: Wrapping::None,
        };

        state.paragraph = Paragraph::with_spans(text);

        Node::new(limits.max())
    }

    fn on_event(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        event: iced::Event,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) -> iced::advanced::graphics::core::event::Status {
        if let iced::Event::Window(iced::window::Event::RedrawRequested(_event)) = event {
            let term = &self.term.term;
            let screen = term.screen();

            let widget_width = layout.bounds().width;
            let widget_height = layout.bounds().height;
            let line_height = renderer.default_size().0;
            let char_width = line_height * 0.6;

            let target_line_count = (0.78 * widget_height / line_height) as usize;
            let target_col_count = (widget_width / char_width) as usize;

            if screen.physical_rows != target_line_count || screen.physical_cols != target_col_count
            {
                let size = TerminalSize {
                    rows: target_line_count,
                    cols: target_col_count,
                    pixel_height: widget_height as usize,
                    pixel_width: widget_width as usize,
                    ..Default::default()
                };
                let message = (self.term.on_resize)(size);
                shell.publish(message);
            }

            return iced::advanced::graphics::core::event::Status::Captured;
        }

        iced::advanced::graphics::core::event::Status::Ignored
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

        let state = tree.state.downcast_ref::<TerminalWidgetState<Renderer>>();

        renderer.fill_paragraph(&state.paragraph, bounds.position(), Color::WHITE, bounds);
    }
}
