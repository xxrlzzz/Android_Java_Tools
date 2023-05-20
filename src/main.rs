use clap::Parser;
use class_parser::{
  error::Error,
  raw_class::ClassFile,
  ui::{app::App, restore_terminal, run_app, setup_terminal},
};

use std::{path::Path, time::Duration};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
  #[clap(short, long, default_value = "./resource/Rectangle.class")]
  path: String,
}

fn parse_file(path: String) -> Result<ClassFile, Error> {
  let path = Path::new(path.as_str());
  let bytes = std::fs::read(path).unwrap();
  let class_file = class_parser::parse(&bytes)?;
  Ok(class_file)
}

fn main() -> Result<(), Error> {
  let arg = Args::parse();
  let class_file = parse_file(arg.path)?;
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
