extern crate parse_zoneinfo;

use parse_zoneinfo::line::{LineParser, Line};
use parse_zoneinfo::table::TableBuilder;
use parse_zoneinfo::transitions::TableTransitions;
use std::path::Path;
use std::{io, env};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::collections::{BTreeSet, HashSet};
use std::collections::hash_map::Entry;
use maplit::*;
use std::collections::HashMap;

fn strip_comments(mut line: String) -> String {
  if let Some(pos) = line.find('#') { line.truncate(pos) }
  line
}

fn main() -> io::Result<()> {
  let parser = LineParser::default();
  let mut table = TableBuilder::new();

  for entry in fs::read_dir(Path::new("tzdata"))? {
    let entry = entry?;
    let f = File::open(entry.path())?;
    let lines = BufReader::new(f).lines();
    for line in lines {
      match parser.parse_str(&strip_comments(line?)).unwrap() {
        Line::Zone(zone) => table.add_zone_line(zone).unwrap(),
        Line::Continuation(cont) => table.add_continuation_line(cont).unwrap(),
        Line::Rule(rule) => table.add_rule_line(rule).unwrap(),
        Line::Link(link) => table.add_link_line(link).unwrap(),
        Line::Space => ()
      }
    }
  }

  let table = table.build();
  let timezone_db_path = Path::new(&env::var("OUT_DIR").unwrap()).join("timezone_db.rs");
  let mut timezone_db_file = File::create(&timezone_db_path)?;
  writeln!(timezone_db_file, "use lazy_static::*;")?;
  writeln!(timezone_db_file, "use maplit::*;")?;
  writeln!(timezone_db_file, "use std::collections::HashSet;")?;
  writeln!(timezone_db_file, "use std::collections::HashMap;")?;
  writeln!(timezone_db_file)?;
  writeln!(timezone_db_file, "lazy_static!{{")?;
  writeln!(timezone_db_file, "  pub static ref ZONES: HashSet<&'static str> = hashset!(")?;

  let zones = table.zonesets.keys().chain(table.links.keys()).collect::<BTreeSet<_>>();
  for zone in &zones {
    writeln!(timezone_db_file, "    \"{}\",", zone)?;
  }

  writeln!(timezone_db_file, "  );")?;
  writeln!(timezone_db_file, "  pub static ref ZONES_ABBR: HashMap<&'static str, Vec<&'static str>> = hashmap!(")?;

  let mut abbr : HashMap<String, HashSet<String>> = hashmap!{};
  for zone in &zones {
    let timespans = table.timespans(zone).unwrap().rest;
    for (_, timespan) in timespans {
      match abbr.entry(timespan.name) {
        Entry::Vacant(e) => { e.insert(hashset![zone.to_string()]); },
        Entry::Occupied(mut e) => { e.get_mut().insert(zone.to_string()); }
      }
    }
  }

  for (key, val) in abbr {
    writeln!(timezone_db_file, "    \"{}\" => vec![", key)?;
    for v in val {
      writeln!(timezone_db_file, "      \"{}\",", v)?;
    }
    writeln!(timezone_db_file, "    ],")?;
  }

  writeln!(timezone_db_file, "  );")?;
  writeln!(timezone_db_file, "}}")?;

  Ok(())
}
