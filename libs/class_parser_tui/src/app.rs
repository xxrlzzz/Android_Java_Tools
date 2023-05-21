use std::cell::Cell;

use base::RenderSource;
use class_parser::{
  attribute::{AttributeInfo, CODE_ATTRIBUTE_NAME},
  raw_class::ClassFile,
};
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
  backend::Backend,
  layout::{Constraint, Direction, Layout},
  style::{Color, Modifier, Style},
  text::{Span, Spans, Text},
  widgets::{Block, Borders, Paragraph, Tabs, Wrap},
  Frame,
};

use super::{
  stateful_paragraph::{ParagraphState, StatefulParagraph},
  stateful_select_list::SelectableList,
};

pub struct App<'a> {
  pub titles: Vec<&'a str>,
  pub index: usize,

  pub class_file: &'a dyn RenderSource,
  list: SelectableList<'a, String>,
  constant_pool_state: Cell<ParagraphState>,
}

impl<'a> App<'a> {
  pub fn new(class_file: &'a ClassFile) -> App<'a> {
    let method_list: Vec<(&str, String)> = class_file
      .render_methods_verbose()
      .into_iter()
      .map(|method| {
        let code: Vec<&AttributeInfo> = method
          .attributes
          .iter()
          .filter(|attr| attr.type_filter(CODE_ATTRIBUTE_NAME))
          .collect();
        if code.len() > 0 {
          return (method.name(), format!("{} (code)", method.to_string()));
        }
        return (method.name(), method.to_string());
      })
      .collect();
    App {
      titles: vec![
        "FileInfo",
        "ClassInfo",
        "Interfaces",
        "Fields",
        "Methods",
        "Attributes",
        "ConstantPool",
        "Detail",
      ],
      index: 0,
      class_file,
      list: SelectableList::new(method_list, "method"),
      constant_pool_state: Cell::new(ParagraphState::default()),
    }
  }

  pub fn next(&mut self) {
    self.index = (self.index + 1) % self.titles.len();
  }

  pub fn previous(&mut self) {
    if self.index > 0 {
      self.index -= 1;
    } else {
      self.index = self.titles.len() - 1;
    }
  }

  pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
    let size = f.size();
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .margin(2)
      .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
      .split(size);

    let block = Block::default().style(Style::default().bg(Color::White).fg(Color::Black));
    f.render_widget(block, size);
    let titles = self
      .titles
      .iter()
      .map(|t| {
        let (first, rest) = t.split_at(1);
        Spans::from(vec![
          Span::styled(first, Style::default().fg(Color::Yellow)),
          Span::styled(rest, Style::default().fg(Color::Green)),
        ])
      })
      .collect();
    let tabs = Tabs::new(titles)
      .block(Block::default().borders(Borders::ALL).title("Tabs"))
      .select(self.index)
      .style(Style::default().fg(Color::Cyan))
      .highlight_style(
        Style::default()
          .add_modifier(Modifier::BOLD)
          .bg(Color::Black),
      );
    f.render_widget(tabs, chunks[0]);

    let inner_block = Block::default().title("Content").borders(Borders::ALL);
    if self.index < 6 {
      let text = self.render_content();
      let paragraph = Paragraph::new(text)
        .style(Style::default().bg(Color::White).fg(Color::Black))
        .block(inner_block);
      f.render_widget(paragraph, chunks[1]);
    } else if self.index == 6 {
      let text: Text = self.render_content().into();
      let paragraph: StatefulParagraph = StatefulParagraph::new(text)
        .wrap(Wrap { trim: false })
        .block(Block::default().title("ConstantPool").borders(Borders::ALL));
      let mut state = self.constant_pool_state.get();
      f.render_stateful_widget(paragraph, chunks[1], &mut state);
      self.constant_pool_state.set(state);
    } else {
      self.list.draw(f, chunks[1]);
    }
  }

  fn render_content(&self) -> Vec<Spans> {
    let strings = match self.index {
      0 => self.class_file.render_file_info(),
      1 => self.class_file.render_class_info(),
      2 => self.class_file.render_interfaces(),
      3 => self.class_file.render_fields(),
      4 => self.class_file.render_methods(),
      5 => self.class_file.render_attributes(),
      6 => self.class_file.render_constant_pool(),
      _ => unreachable!(),
    };
    strings.into_iter().map(|s| Spans::from(s)).collect()
  }

  pub fn handle_key(&mut self, key: KeyEvent) {
    match key.code {
      KeyCode::Right => self.next(),
      KeyCode::Left => self.previous(),
      KeyCode::Up => {
        if self.index == 6 {
          let mut state = self.constant_pool_state.get();
          state.set_scroll_vertical(state.scroll().y.saturating_sub(1));
          self.constant_pool_state.set(state);
        } else if self.index == 7 {
          self.list.items.previous()
        }
      }
      KeyCode::Down => {
        if self.index == 6 {
          let mut state = self.constant_pool_state.get();
          state.set_scroll_vertical(state.scroll().y.saturating_add(1));
          self.constant_pool_state.set(state);
        } else if self.index == 7 {
          self.list.items.next()
        }
      }
      KeyCode::Enter => {
        if self.index == 7 {
          self.list.items.toggle();
        }
      }
      _ => {}
    }
  }

  pub fn on_tick(&mut self) {}
}
