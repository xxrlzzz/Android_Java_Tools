use base::error::Error;
use clap::Parser;
use class_parser_tui::{app::App, restore_terminal, run_app, setup_terminal};
use simplelog::*;

extern crate simplelog;

use std::{path::Path, time::Duration};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
  #[clap(short, long, default_value = "./resource/Rectangle.class")]
  path: String,
  #[clap(short, long, default_value = "false")]
  class: bool,
  #[clap(short, long, default_value = "false")]
  dex: bool,
}

fn parse_file(path: String) -> Result<Vec<u8>, Error> {
  let path = Path::new(path.as_str());
  let bytes = std::fs::read(path).unwrap();
  Ok(bytes)
}

fn run_class(arg: Args) -> Result<(), Error> {
  let class_file = parse_file(arg.path)?;
  let class_file = class_parser::parse(&class_file)?;
  let mut terminal = setup_terminal()?;

  // create app and run it
  let tick_rate = Duration::from_millis(250);
  let app = App::new(&class_file);
  let res = run_app(&mut terminal, app, tick_rate);

  // restore terminal
  restore_terminal(terminal)?;

  if let Err(err) = res {
    println!("{:?}", err)
  }

  Ok(())
}

fn run_dex(arg: Args) -> Result<(), Error> {
  CombinedLogger::init(vec![TermLogger::new(
    LevelFilter::Info,
    Config::default(),
    TerminalMode::Mixed,
    ColorChoice::Auto,
  )])
  .unwrap();

  let dex_file = parse_file(arg.path)?;
  let dex_file = dex_parser::parse(&dex_file)?;
  print!("{}", dex_file);

  Ok(())
}

fn main() -> Result<(), Error> {
  let arg = Args::parse();
  if arg.class {
    run_class(arg)?
  } else if arg.dex {
    run_dex(arg)?
  }
  Ok(())
}
