#![feature(assoc_char_funcs)]

mod dict;
mod rtf;
mod translation_model;
mod translation_parse;
mod translation;

#[macro_use]
extern crate lazy_static;
extern crate criterion;
use criterion::*;

use std::fs::{File, read_to_string};
use std::io::{Read, sink};
use serde_json::Value;

use dict::{Dictionary, Entry};
use rtf::parse_rtf;
use translation::format_plover_to_rtf;
use translation_parse::format_rtf_to_plover;

lazy_static! {
  static ref DICT: Dictionary = {
    let mut dict = Dictionary::new("Plover");

    if let Ok(mut json_dict) = File::open("/Users/sammi/projects/plover/dict/di/dict.json") {
      let mut contents = String::new();
      if let Ok(_) = json_dict.read_to_string(&mut contents) {
        if let Ok(json) = serde_json::from_str(&contents) {
          if let Value::Object(map) = json {
            for (steno, value) in map.iter() {
              if let Value::String(translation) = value {
                dict.add_entry(String::from(steno), translation.clone(), None);
              }
            }
          }
        }
      }
    }

    dict
  };

  static ref FIVE_ITEM_DICT: Dictionary = {
    let mut d = Dictionary::new("Plover");

    add_entry!(d, "TEFT" => "test");
    add_entry!(d, "TEFTS" => "tests");
    add_entry!(d, "TEFTD" => "tested");
    add_entry!(d, "TEFGT" => "testing");
    add_entry!(d, "TEFT/-G" => "testing");

    d
  };

  static ref TEN_ITEM_DICT: Dictionary = {
    let mut d = Dictionary::new("Plover");

    add_entry!(d, "TEFT" => "test");
    add_entry!(d, "TEFTS" => "tests");
    add_entry!(d, "TEFTD" => "tested");
    add_entry!(d, "TEFGT" => "testing");
    add_entry!(d, "TEFT/-G" => "testing");

    add_entry!(d, "TPAEUL" => "fail");
    add_entry!(d, "TPAEULS" => "fails");
    add_entry!(d, "TPAEULD" => "failed");
    add_entry!(d, "TPAEULG" => "failing");
    add_entry!(d, "TPAEUL/-G" => "failing");

    d
  };
}

fn bench_dict(c: &mut Criterion) {
  let mut group = c.benchmark_group("dict");

  let outline = "HROERL";
  group.bench_function("lookup", |b| b.iter(|| DICT.lookup(&outline)));

  let word = "really";
  group.bench_function("rev_lookup", |b| b.iter(|| DICT.rev_lookup(&word)));

  let mut d = Dictionary::new("Plover");
  group.bench_function("add_entry", |b| {
    b.iter(|| add_entry!(d, "RTF/RTF" => "Rich Text Format"));
    remove_entry!(d, "RTF/RTF");
  });

  group.bench_function("remove_entry", |b| {
    add_entry!(d, "RTF/RTF" => "Rich Text Format");
    b.iter(|| remove_entry!(d, "RTF/RTF"));
  });
}

fn bench_entry(c: &mut Criterion) {
  let mut group = c.benchmark_group("entry");

  macro_rules! bench_tl {
    ($name:literal => $tl:expr) => {
      group.bench_function($name, |b| b.iter(|| format_plover_to_rtf($tl)));
    };
  }

  macro_rules! bench_parse {
    ($name:literal => $tl:expr) => {
      group.bench_function($name, |b| b.iter(|| format_rtf_to_plover($tl)));
    };
  }

  bench_tl!("emit/plain" => "testing");
  bench_tl!("emit/long" => "Rich Text Format with Court Reporting Extensions");
  bench_tl!("emit/longer" => "Please be patient. I am typing a very thoughtful \
response using stenography. This shit is hard yo.");
  bench_tl!("emit/extra_long" => "You seem to be talking about steganography \
(hiding information in images and such) while this Discord is about \
stenography (inputting text quickly by using a special type of keyboard).
It's a common mistake, no worries!");
  bench_tl!("emit/punct" => "{.}");
  bench_tl!("emit/complex" => "lookup{PLOVER:LOOKUP}{-|}{^ed}");

  bench_parse!("parse/plain" => "testing");
  bench_parse!("parse/long" => "Rich Text Format with Court Reporting Extensions");
  bench_parse!("parse/longer" => "Please be patient. I am typing a very thoughtful \
response using stenography. This shit is hard yo.");
  bench_parse!("parse/extra_long" => "You seem to be talking about steganography \
(hiding information in images and such) while this Discord is about \
stenography (inputting text quickly by using a special type of keyboard).
It's a common mistake, no worries!");
  bench_parse!("parse/punct" => "{\\cxp. }");
  bench_parse!("parse/complex" => "\\cxds ed{\\cxp. }{\\*\\cxplvrcmd lookup}");

  let entry = Entry::new("TEFGT", "testing", None);
  let mut sink = sink();
  group.bench_function("write", |b| b.iter(|| entry.write(&mut sink)));
}

fn bench_rtf(c: &mut Criterion) {
  let mut group = c.benchmark_group("rtf");
  group.sample_size(10);

  let file: &str = "/tmp/dict.rtf";
  let dicts: Vec<&Dictionary> = vec![&FIVE_ITEM_DICT, &TEN_ITEM_DICT, &DICT];

  for dict in dicts {
    group.bench_with_input(BenchmarkId::new("write", dict.len()), dict, |b, i| {
      if let Ok(mut file) = File::create(file) {
        b.iter(|| i.write(&mut file));
      }
    });

    group.bench_with_input(BenchmarkId::new("read", dict.len()), dict, |b, i| {
      if let Ok(mut file) = File::create(file) {
        let _ = i.write(&mut file);
      }
      if let Ok(contents) = read_to_string(file) {
        b.iter(|| parse_rtf(&contents));
      }
    });
  }
}

criterion_group!(benches, bench_dict, bench_entry, bench_rtf);
criterion_main!(benches);
