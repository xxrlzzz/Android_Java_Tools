use std::iter;
use tui::{
  buffer::Buffer,
  layout::{Alignment, Rect},
  style::Style,
  text::{StyledGrapheme, Text},
  widgets::{Block, StatefulWidget, Widget, Wrap},
};
use unicode_width::UnicodeWidthStr;

use super::reflow::{LineComposer, LineTruncator, WordWrapper};

const fn get_line_offset(line_width: u16, text_area_width: u16, alignment: Alignment) -> u16 {
  match alignment {
    Alignment::Center => (text_area_width / 2).saturating_sub(line_width / 2),
    Alignment::Right => text_area_width.saturating_sub(line_width),
    Alignment::Left => 0,
  }
}

#[derive(Debug, Clone)]
pub struct StatefulParagraph<'a> {
  /// A block to wrap the widget in
  block: Option<Block<'a>>,
  /// Widget style
  style: Style,
  /// How to wrap the text
  wrap: Option<Wrap>,
  /// The text to display
  text: Text<'a>,
  /// Alignment of the text
  alignment: Alignment,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ScrollPos {
  pub x: u16,
  pub y: u16,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct ParagraphState {
  /// Scroll
  scroll: ScrollPos,
  /// after all wrapping this is the amount of lines
  lines: u16,
  /// last visible height
  height: u16,
}

impl ParagraphState {
  // const fn lines(self) -> u16 {
  //   self.lines
  // }

  // const fn height(self) -> u16 {
  //   self.height
  // }

  pub const fn scroll(self) -> ScrollPos {
    self.scroll
  }

  pub fn set_scroll(&mut self, scroll: ScrollPos) {
    self.scroll = scroll;
  }

  // pub fn set_scroll_horizontal(&mut self, x: u16) {
  //   self.scroll.x = x;
  // }

  pub fn set_scroll_vertical(&mut self, y: u16) {
    let y = y.min(self.lines.saturating_sub(self.height.saturating_sub(2)));
    self.scroll.y = y;
  }
}

impl<'a> StatefulParagraph<'a> {
  pub fn new<T>(text: T) -> Self
  where
    T: Into<Text<'a>>,
  {
    Self {
      block: None,
      style: Style::default(),
      wrap: None,
      text: text.into(),
      alignment: Alignment::Left,
    }
  }

  #[allow(clippy::missing_const_for_fn)]
  pub fn block(mut self, block: Block<'a>) -> Self {
    self.block = Some(block);
    self
  }

  pub const fn wrap(mut self, wrap: Wrap) -> Self {
    self.wrap = Some(wrap);
    self
  }

  // pub const fn style(mut self, style: Style) -> Self {
  // 	self.style = style;
  // 	self
  // }

  // pub const fn alignment(mut self, alignment: Alignment) -> Self {
  // 	self.alignment = alignment;
  // 	self
  // }
}

impl<'a> StatefulWidget for StatefulParagraph<'a> {
  type State = ParagraphState;

  fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    buf.set_style(area, self.style);
    let text_area = self.block.take().map_or(area, |b| {
      let inner_area = b.inner(area);
      b.render(area, buf);
      inner_area
    });

    if text_area.height < 1 {
      return;
    }

    let style = self.style;
    let mut styled = self.text.lines.iter().flat_map(|spans| {
      spans
        .0
        .iter()
        .flat_map(|span| span.styled_graphemes(style))
        // Required given the way composers work but might be refactored out if we change
        // composers to operate on lines instead of a stream of graphemes.
        .chain(iter::once(StyledGrapheme {
          symbol: "\n",
          style: self.style,
        }))
    });

    let mut line_composer: Box<dyn LineComposer> = if let Some(Wrap { trim }) = self.wrap {
      Box::new(WordWrapper::new(&mut styled, text_area.width, trim))
    } else {
      let mut line_composer = Box::new(LineTruncator::new(&mut styled, text_area.width));
      if self.alignment == Alignment::Left {
        line_composer.set_horizontal_offset(state.scroll.x);
      }
      line_composer
    };
    let mut y = 0;
    let mut end_reached = false;
    while let Some((current_line, current_line_width)) = line_composer.next_line() {
      if !end_reached && y >= state.scroll.y {
        let mut x = get_line_offset(current_line_width, text_area.width, self.alignment);
        for StyledGrapheme { symbol, style } in current_line {
          buf
            .get_mut(text_area.left() + x, text_area.top() + y - state.scroll.y)
            .set_symbol(if symbol.is_empty() {
              // If the symbol is empty, the last char which rendered last time will
              // leave on the line. It's a quick fix.
              " "
            } else {
              symbol
            })
            .set_style(*style);
          x += symbol.width() as u16;
        }
      }
      y += 1;
      if y >= text_area.height + state.scroll.y {
        end_reached = true;
      }
    }

    state.lines = y;
    state.height = area.height;
  }
}
