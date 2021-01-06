#[macro_use]
mod dict;
#[macro_use]
mod rtf;
mod translation_model;
mod translation_parse;
mod translation;

#[macro_use]
extern crate lazy_static;

use std::convert::From;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

use serde_json::Value;
use structopt::StructOpt;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::dict::{Dictionary, Entry};
use crate::rtf::parse_file;

lazy_static!{
  static ref VERSION_STRING: String =
    format!("rtfcre {0} (https://github.com/sammdot/rtfcre)",
      env!("CARGO_PKG_VERSION"));
}

#[derive(Debug)]
struct CommandParseError {}
impl fmt::Display for CommandParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "command parse error")
  }
}
#[derive(Debug)]
enum CommandMode {
  Convert,
  CountEntries,
}
impl FromStr for CommandMode {
  type Err = CommandParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "convert" => Ok(Self::Convert),
      "count" => Ok(Self::CountEntries),
      _ => Err(CommandParseError {}),
    }
  }
}

#[derive(StructOpt, Debug)]
struct CommandLine {
  #[structopt(short, default_value = "convert")]
  /// The mode this program should run in, one of 'convert' or 'count'.
  mode: CommandMode,
  #[structopt(parse(from_os_str))]
  /// The path of the file to convert. Must have a .rtf or .json extension.
  input: PathBuf,
  #[structopt(parse(from_os_str), required_if("mode", "convert"))]
  /// The path of the output file. Must have a .rtf or .json extension.
  output: Option<PathBuf>,
}

enum Direction {
  RtfToJson,
  JsonToRtf,
}

lazy_static! {
  static ref RED: ColorSpec = {
    let mut color = ColorSpec::new();
    color.set_fg(Some(Color::Red)).set_bold(true);
    color
  };
}

fn error(out: &mut StandardStream, message: String) -> Result<(), io::Error> {
  out.set_color(&RED)?;
  write!(out, "error: ")?;
  out.reset()?;
  writeln!(out, "{}", message)?;

  Ok(())
}

enum RtfCreError {
  InvalidArgument,
  IoError { err: io::Error },
  RtfParseError,
  RtfWriteError,
  JsonParseError,
  JsonWriteError,
}
impl From<io::Error> for RtfCreError {
  fn from(err: io::Error) -> Self { Self::IoError { err } }
}
impl fmt::Display for RtfCreError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::IoError { err } => write!(f, "I/O error: {:?}", err),
      _ => write!(f, "{}", match self {
        Self::InvalidArgument => "invalid arguments",
        Self::RtfParseError => "could not parse RTF file",
        Self::RtfWriteError => "could not write RTF file",
        Self::JsonParseError => "could not parse JSON file",
        Self::JsonWriteError => "could not write JSON file",
        _ => "",
      })
    }
  }
}

fn run_main() -> Result<(), RtfCreError> {
  let args = CommandLine::from_args();

  match args.mode {
    CommandMode::Convert => {
      let output = args.output.unwrap();
      let extensions = (
        match args.input.extension() { Some(x) => x.to_str(), None => None },
        match output.extension() { Some(x) => x.to_str(), None => None });
      let direction = match extensions {
        (Some("rtf"), Some("json")) => Ok(Direction::RtfToJson),
        (Some("json"), Some("rtf")) => Ok(Direction::JsonToRtf),
        (Some(x), Some(y)) if x == y => Err(RtfCreError::InvalidArgument),
        _ => Err(RtfCreError::InvalidArgument),
      }?;

      let mut input = File::open(args.input)?;
      let mut output = File::create(output)?;
      let mut contents = String::new();
      input.read_to_string(&mut contents)?;
      match direction {
        Direction::RtfToJson => {
          let dict = match parse_file(&contents) {
            Err(e) => {
              eprintln!("{:?}", e);
              Err(RtfCreError::RtfParseError)
            },
            Ok((_, d)) => Ok(d),
          }?;
          let mut map = serde_json::Map::with_capacity(dict.len());
          for (steno, Entry { translation, .. }) in dict.entries {
            map.insert(steno, Value::String(translation));
          }
          match serde_json::to_writer_pretty(&mut output, &map) {
            Ok(_) => Ok(()),
            Err(_) => Err(RtfCreError::JsonWriteError),
          }
        },
        Direction::JsonToRtf => {
          let mut dict = Dictionary::new(&VERSION_STRING);
          match serde_json::from_str(&contents) {
            Ok(Value::Object(map)) => {
              for (steno, value) in map.iter() {
                if let Value::String(translation) = value {
                  dict.add_entry(String::from(steno), translation.clone(), None);
                }
              }
              dict.write(&mut output)?;
              Ok(())
            },
            _ => Err(RtfCreError::JsonParseError),
          }
        },
      }
    },
    CommandMode::CountEntries => {
      let _ext = match args.input.extension() {
        Some(x) => match x.to_str() {
          Some("rtf") => Ok("rtf"),
          _ => Err(RtfCreError::InvalidArgument),
        },
        _ => Err(RtfCreError::InvalidArgument),
      }?;

      let mut input = File::open(args.input)?;
      let mut contents = String::new();
      input.read_to_string(&mut contents)?;
      let dict = match parse_file(&contents) {
        Err(e) => {
          eprintln!("{:?}", e);
          Err(RtfCreError::RtfParseError)
        },
        Ok((_, d)) => Ok(d),
      }?;

      println!("{:?}", dict.len());

      Ok(())
    },
  }
}

fn main() {
  let mut stderr = StandardStream::stderr(ColorChoice::Always);

  match run_main() {
    Ok(_) => {},
    Err(e) => {
      let _ = error(&mut stderr, format!("{}", e));
      exit(1);
    }
  };
}
