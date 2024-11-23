use std::ops::RangeBounds;

use iced::{Border, Color, Element};
use iced_widget::{column, container, row, text};
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
            grid: vec![Symbol { c: ' ', format: Default::default() }; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn set_symbol(&mut self, x: usize, y: usize, symbol: Symbol) -> Result<(), OutOfBoundsError> {
        if x >= self.width {
            return Err(OutOfBoundsError::XOutOfBounds(x, self.width));
        }
        if y >= self.height {
            return Err(OutOfBoundsError::YOutOfBounds(y, self.height));
        }

        self.grid[y * self.width + x] = symbol;

        Ok(())
    }

    pub fn fill_range(&mut self, range: impl RangeBounds<usize>, symbol: Symbol) -> Result<(), OutOfBoundsError> {
        let start = match range.start_bound() {
            std::ops::Bound::Included(x) => *x,
            std::ops::Bound::Excluded(x) => *x + 1,
            std::ops::Bound::Unbounded => 0,
        };
        if start >= self.grid.len() {
            return Err(OutOfBoundsError::IndexOutOfBounds(start, self.grid.len()));
        }

        let end = match range.end_bound() {
            std::ops::Bound::Included(x) => *x +1,
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
        self.grid[(self.height - 1) * self.width..].fill(Symbol { c: ' ', format: Default::default() });
    }

    pub fn rows(&self) -> Vec<&[Symbol]> {
        self.grid.chunks(self.width).collect()
    }

    pub fn view<
    'a,
    Message,
    Theme,
    Renderer,
>(&self) -> Element<'a, Message, Theme, Renderer> where 
        Message: Clone +'a, 
        Renderer: 'a,
        Renderer: iced_core::Renderer ,
        Renderer: iced_core::text::Renderer,
        Theme: text::Catalog + 'a,
        Theme: container::Catalog,
        <Theme as text::Catalog>::Class<'a>: From<iced_core::widget::text::StyleFn<'a, Theme>>,
        <Theme as iced_widget::container::Catalog>::Class<'a>: From<iced_widget::container::StyleFn<'a, Theme>>,
    {
        container(
        column(self.rows().iter().map(|r| {
            row(r.iter().map(|symbol| {
                text(symbol.c.to_string())
                    .color_maybe(symbol.format.foreground)
                    .width(12)
                    .center()
                    .into()
            }))
            .into()
        }))
    ).style(|_| container::Style {
        border: Border {
            radius: Default::default(),
            width: 2.0,
            color: Color::from_rgb(0.5, 0.5, 0.5)
        },
        ..Default::default()
    })
        .into()
    }

}

struct TextGridDisplay<'a> {
    grid: &'a TextGrid
}