use std::{fmt::Display, iter};
use tui::{
  backend::Backend,
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Span, Spans, StyledGrapheme, Text},
  widgets::{Block, Borders, List, ListItem, ListState},
  Frame,
};

use super::reflow::{LineComposer, WordWrapper};

#[derive(Clone)]
pub struct StatefulList<T> {
  state: ListState,
  items: Vec<T>,
  toggle: bool,
}

impl<T> StatefulList<T> {
  fn with_items(items: Vec<T>) -> StatefulList<T> {
    let mut state = ListState::default();
    state.select(Some(0));
    StatefulList {
      state,
      items,
      toggle: false,
    }
  }

  pub fn next(&mut self) {
    let i = match self.state.selected() {
      Some(i) => {
        if i >= self.items.len() - 1 {
          0
        } else {
          i + 1
        }
      }
      None => 0,
    };
    self.state.select(Some(i));
  }

  pub fn previous(&mut self) {
    let i = match self.state.selected() {
      Some(i) => {
        if i == 0 {
          self.items.len() - 1
        } else {
          i - 1
        }
      }
      None => 0,
    };
    self.state.select(Some(i));
  }

  // fn unselect(&mut self) {
  //   self.state.select(None);
  // }

  pub fn toggle(&mut self) {
    self.toggle = !self.toggle;
  }

  fn is_toggled(&self) -> bool {
    self.toggle
  }
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is a wrapper
/// around `ListState`. Keeping track of the items state let us render the associated widget with its state
/// and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
#[derive(Clone)]
pub struct SelectableList<'a, T: Display> {
  pub items: StatefulList<(&'a str, T)>,
  title: &'a str,
}

impl<'a, T: Display> SelectableList<'a, T> {
  pub fn new(items: Vec<(&'a str, T)>, title: &'a str) -> SelectableList<'a, T> {
    SelectableList {
      items: StatefulList::with_items(items),
      title,
    }
  }

  pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, r: Rect) {
    let chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
      .split(r);

    let style = Style::default()
      .bg(Color::LightGreen)
      .add_modifier(Modifier::BOLD);

    let selected_idx = self.items.state.selected().unwrap_or(0);
    let methods: Vec<ListItem> = self
      .items
      .items
      .iter()
      .zip(0..)
      .map(|(e, idx)| {
        let idx_str = format!("# {}. ", idx);
        let mut lines = vec![Spans::from(Span::raw(idx_str)), Spans::from(Span::raw(e.0))];
        if selected_idx == idx && self.items.is_toggled() {
          // TODO add more information for the selected item.
          lines.push(Spans::from(Span::raw(format!(" "))));
        }
        ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
      })
      .collect();

    let items = List::new(methods)
      .block(Block::default().borders(Borders::ALL).title(self.title))
      .highlight_style(style)
      .highlight_symbol(if self.items.is_toggled() { "- " } else { "> " });
    f.render_stateful_widget(items, chunks[0], &mut self.items.state);

    let selected_idx = self.items.state.selected().unwrap_or(0);
    let (item, content) = &self.items.items[selected_idx];
    let text: Text = content.to_string().into();
    let items: ListItem = {
      let mut lines = vec![Spans::from(Span::styled(*item, style.clone()))];
      let mut styled = text.lines.iter().flat_map(|spans| {
        spans
          .0
          .iter()
          .flat_map(|span| span.styled_graphemes(style))
          // Required given the way composers work but might be refactored out if we change
          // composers to operate on lines instead of a stream of graphemes.
          .chain(iter::once(StyledGrapheme {
            symbol: "\n",
            style: Style::default(),
          }))
      });
      // break lines to fit the length.
      let mut line_composer = WordWrapper::new(&mut styled, chunks[1].width, false);
      while let Some((current_line, _)) = line_composer.next_line() {
        let str = current_line
          .iter()
          .fold(String::new(), |mut acc, grapheme| {
            acc = format!("{}{}", acc, grapheme.symbol);
            acc
          });
        lines.push(Spans::from(Span::raw(str)));
      }
      ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
    };

    let items = List::new(vec![items])
      .block(Block::default().borders(Borders::ALL).title("Content"))
      .highlight_style(style);
    f.render_widget(items, chunks[1])
    // b.render_widget(items, chunks[1]);
  }
}
