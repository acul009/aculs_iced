use std::ops::RangeBounds;

use iced::{
    advanced::{
        self, layout,
        renderer::{self, Quad},
        text::{self, Paragraph},
        widget::Widget,
    },
    alignment::{Horizontal, Vertical},
    theme,
    widget::{container, rich_text, span},
    Background, Border, Color, Element, Font, Length, Point, Rectangle, Shadow, Size,
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub c: char,
    pub format: Format,
}

#[derive(Debug, Clone)]
pub struct Format {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub bold: bool,
    pub faint: bool,
    pub italic: bool,
    pub underline: bool,
    pub blinking: bool,
    pub inverse: bool,
    pub hidden: bool,
    pub strikethrough: bool,
}

impl Default for Format {
    fn default() -> Self {
        Self {
            foreground: None,
            background: None,
            bold: false,
            faint: false,
            italic: false,
            underline: false,
            blinking: false,
            inverse: false,
            hidden: false,
            strikethrough: false,
        }
    }
}

#[derive(Error, Debug)]
pub enum OutOfBoundsError {
    #[error("x: {0} is out of bounds: {1}")]
    XOutOfBounds(usize, usize),
    #[error("y: {0} is out of bounds: {1}")]
    YOutOfBounds(usize, usize),
    #[error("index: {0} is out of bounds: {1}")]
    IndexOutOfBounds(usize, usize),
}

pub struct TextGrid {
    width: usize,
    height: usize,
    grid: Vec<Symbol>,
}

impl TextGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            grid: vec![
                Symbol {
                    c: ' ',
                    format: Default::default()
                };
                width * height
            ],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn set_symbol(
        &mut self,
        x: usize,
        y: usize,
        symbol: Symbol,
    ) -> Result<(), OutOfBoundsError> {
        if x >= self.width {
            return Err(OutOfBoundsError::XOutOfBounds(x, self.width));
        }
        if y >= self.height {
            return Err(OutOfBoundsError::YOutOfBounds(y, self.height));
        }

        self.grid[y * self.width + x] = symbol;

        Ok(())
    }

    pub fn fill_range(
        &mut self,
        range: impl RangeBounds<usize>,
        symbol: Symbol,
    ) -> Result<(), OutOfBoundsError> {
        let start = match range.start_bound() {
            std::ops::Bound::Included(x) => *x,
            std::ops::Bound::Excluded(x) => *x + 1,
            std::ops::Bound::Unbounded => 0,
        };
        if start >= self.grid.len() {
            return Err(OutOfBoundsError::IndexOutOfBounds(start, self.grid.len()));
        }

        let end = match range.end_bound() {
            std::ops::Bound::Included(x) => *x + 1,
            std::ops::Bound::Excluded(x) => *x,
            std::ops::Bound::Unbounded => self.grid.len(),
        };

        if end > self.grid.len() {
            return Err(OutOfBoundsError::IndexOutOfBounds(end, self.grid.len()));
        }

        self.grid[start..end].fill(symbol);

        Ok(())
    }

    pub fn push_row(&mut self) {
        // copy each row to the next in reverse order to avoid losing data
        for y in 0..(self.height - 1) {
            for x in 0..self.width {
                self.grid[y * self.width + x] = self.grid[(y + 1) * self.width + x].clone();
            }
        }
        self.grid[(self.height - 1) * self.width..].fill(Symbol {
            c: ' ',
            format: Default::default(),
        });
    }

    pub fn rows(&self) -> impl Iterator<Item = &[Symbol]> {
        self.grid.chunks(self.width)
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
        let mut spans = Vec::with_capacity((self.width + 1) * self.height);

        for row in self.rows() {
            for symbol in row {
                let c = match symbol.format.hidden {
                    true => ' ',
                    false => symbol.c,
                };

                let span = span(c)
                    .background_maybe(symbol.format.background)
                    .color_maybe(symbol.format.foreground)
                    .underline(symbol.format.underline)
                    .strikethrough(symbol.format.strikethrough)
                    .font(Self::retrieve_font(&symbol.format));
                spans.push(span);
            }
            spans.push(span('\n'));
        }

        container(rich_text(spans).font(Font::MONOSPACE)).style(|theme| container::Style {
            text_color: None,
            background: None,
            border: Border::default().color(Color::from_rgb(1.0, 1.0, 1.0)),
            shadow: Shadow::default(),
        })
    }

    fn retrieve_font(format: &Format) -> Font {
        let mut font = Font::MONOSPACE;
        if format.bold {
            font.weight = iced::font::Weight::Bold;
        } else if format.faint {
            font.weight = iced::font::Weight::Light;
        }

        if format.italic {
            font.style = iced::font::Style::Italic;
        }
        font
    }

    pub fn print(&self) {
        for row in self.rows() {
            for symbol in row {
                print!("{}", symbol.c);
            }
            println!();
        }
    }
}

pub struct TextGridDisplay<'a> {
    grid: &'a TextGrid,
}

impl<'a> TextGridDisplay<'a> {
    pub fn new(grid: &'a TextGrid) -> Self {
        Self { grid }
    }

    pub fn char_bounds(&self) -> iced::Size<f32> {
        Size::new(10.0, 20.0)
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for TextGridDisplay<'a>
where
    Renderer: advanced::text::Renderer<Font = Font>,
    Renderer: advanced::Renderer,
{
    fn size(&self) -> Size<Length> {
        let char_bounds = self.char_bounds();
        Size {
            width: Length::Fixed(char_bounds.width * self.grid.width as f32),
            height: Length::Fixed(char_bounds.height * self.grid.height as f32),
        }
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let char_bounds = self.char_bounds();

        layout::atomic(limits, char_bounds.width, char_bounds.height)

        // layout::Node::new(Size {
        //     width: char_bounds.width * self.grid.width as f32,
        //     height: char_bounds.height * self.grid.height as f32,
        // })

        // let char_bounds = Renderer::Paragraph::with_text(advanced::Text {
        //     content: " ",
        //     bounds: Default::default(),
        //     size: 20.into(),
        //     line_height: Default::default(),
        //     font: Font::MONOSPACE,
        //     horizontal_alignment: Horizontal::Center,
        //     vertical_alignment: Vertical::Center,
        //     shaping: text::Shaping::Basic,
        //     wrapping: text::Wrapping::None,
        // })
        // .min_bounds();

        // println!("min bounds: {:?}", char_bounds);

        // let mut children: Vec<layout::Node> =
        //     Vec::with_capacity(self.grid.width * self.grid.height);
        // for y in 0..self.grid.height {
        //     for x in 0..self.grid.width {
        //         children.push(
        //             layout::Node::new(char_bounds)
        //                 .move_to((char_bounds.width * x as f32, char_bounds.height * y as f32)),
        //         );
        //     }
        // }

        // // Todo
        // layout::Node::with_children(
        //     Size {
        //         width: char_bounds.width * self.grid.width as f32,
        //         height: char_bounds.height * self.grid.height as f32,
        //     },
        //     children,
        // )
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: layout::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let char_bounds = self.char_bounds();

        println!("text color: {:?}", style.text_color);

        for (y, row) in self.grid.rows().enumerate() {
            for (x, symbol) in row.iter().enumerate() {
                let paragraph = Renderer::Paragraph::with_spans(advanced::Text {
                    content: &[advanced::text::Span::<'_, (), _>::new(symbol.c.to_string())
                        .strikethrough(symbol.format.strikethrough)
                        .underline(symbol.format.underline)
                        .background_maybe(symbol.format.background.map(Background::Color))
                        .color_maybe(symbol.format.foreground)
                        .underline(symbol.format.underline)],
                    bounds: Default::default(),
                    size: char_bounds.height.into(),
                    line_height: Default::default(),
                    font: Font::MONOSPACE,
                    horizontal_alignment: Horizontal::Center,
                    vertical_alignment: Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::None,
                });

                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point::new(
                                x as f32 * char_bounds.width + viewport.x,
                                y as f32 * char_bounds.height + viewport.y,
                            ),
                            char_bounds,
                        ),
                        ..Default::default()
                    },
                    Background::Color(Color::from_rgb(1.0 * ((x + y) % 2) as f32, 0.0, 0.0)),
                );

                renderer.fill_paragraph(
                    &paragraph,
                    Point::new(x as f32 * char_bounds.width, y as f32 * char_bounds.height),
                    style.text_color,
                    *viewport,
                );
            }
        }
    }
}

impl<'a, Message, Theme, Renderer> From<TextGridDisplay<'a>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: advanced::text::Renderer<Font = Font>,
{
    fn from(display: TextGridDisplay<'a>) -> Self {
        Self::new(display)
    }
}
