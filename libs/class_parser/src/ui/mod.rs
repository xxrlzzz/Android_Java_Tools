use crossterm::{
  event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
  io::{self, Stdout},
  time::{Duration, Instant},
};
use tui::{
  backend::{Backend, CrosstermBackend},
  Terminal,
};

use crate::method::MethodInfo;

use self::app::App;

pub mod app;
mod reflow;
mod stateful_paragraph;
mod stateful_select_list;

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, io::Error> {
  // setup terminal
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  let backend = CrosstermBackend::new(stdout);
  let terminal = Terminal::new(backend)?;
  Ok(terminal)
}

pub fn restore_terminal<B: Backend + std::io::Write>(
  mut terminal: Terminal<B>,
) -> Result<(), io::Error> {
  // restore terminal
  disable_raw_mode()?;
  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
  )?;
  terminal.show_cursor()?;
  Ok(())
}

pub fn run_app<B: Backend>(
  terminal: &mut Terminal<B>,
  mut app: App,
  tick_rate: Duration,
) -> io::Result<()> {
  let mut last_tick = Instant::now();
  loop {
    terminal.draw(|f| app.draw(f))?;

    let timeout = tick_rate
      .checked_sub(last_tick.elapsed())
      .unwrap_or_else(|| Duration::from_secs(0));
    if crossterm::event::poll(timeout)? {
      if let Event::Key(key) = event::read()? {
        match key.code {
          KeyCode::Char('q') => return Ok(()),
          _ => app.handle_key(key),
        }
      }
    }

    if last_tick.elapsed() >= tick_rate {
      app.on_tick();
      last_tick = Instant::now();
    }
  }
}

pub trait RenderSource {
  fn render_file_info(&self) -> Vec<String>;
  fn render_class_info(&self) -> Vec<String>;
  fn render_interfaces(&self) -> Vec<String>;
  fn render_fields(&self) -> Vec<String>;
  fn render_methods(&self) -> Vec<String>;
  fn render_methods_verbose(&self) -> Vec<&MethodInfo>;
  fn render_attributes(&self) -> Vec<String>;
  fn render_constant_pool(&self) -> Vec<String>;
}
