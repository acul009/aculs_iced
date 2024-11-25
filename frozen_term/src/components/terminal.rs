use std::sync::Arc;

use iced::{
    widget::{container, rich_text},
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
    pub fn new() -> Self {
        let size = TerminalSize {
            rows: 40,
            cols: 100,
            ..Default::default()
        };

        let config = Config {};

        let writer = Box::new(std::io::stdout());

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
        let screen = self.term.screen();
        let palette = self.term.palette();

        let width = screen.physical_cols;

        let mut spans =
            Vec::with_capacity(self.term.screen().physical_cols * self.term.screen().physical_rows);

        screen.for_each_phys_line(|index, line| {
            println!("linewidth: {}", line.visible_cells().count());

            let mut lines_found = 0;

            for cell in line.visible_cells() {
                lines_found += 1;
                let style = cell.attrs();

                let span = iced::widget::span(cell.str().to_string())
                    .color(get_color(style.foreground(), &palette, true))
                    .background(get_color(style.background(), &palette, false));

                spans.push(span);
            }

            spans.push(iced::widget::span(" ".repeat(width - lines_found)));
            spans.push(iced::widget::span("\n"));
        });

        let (r, g, b, a) = palette.background.to_tuple_rgba();

        container(rich_text(spans).font(iced::Font::MONOSPACE)).style(move |theme| {
            container::Style {
                text_color: None,
                background: Some(iced::Color::from_rgba(r, g, b, a).into()),
                border: Border::default().color(iced::Color::from_rgb(1.0, 1.0, 1.0)),
                shadow: Shadow::default(),
            }
        })
    }

    pub fn print(&self) {
        let screen = self.term.screen();

        screen.for_each_phys_line(|size, line| {
            println!("{}", line.as_str());
        });
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
