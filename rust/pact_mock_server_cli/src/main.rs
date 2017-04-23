//! The `pact_mock_server` crate provides the CLI for the pact mock server for mocking HTTP requests
//! and generating responses based on a pact file. It implements the V1.1 Pact specification
//! (https://github.com/pact-foundation/pact-specification/tree/version-1.1).

#![warn(missing_docs)]

#[macro_use] extern crate clap;
#[macro_use] extern crate pact_matching;
extern crate pact_mock_server;
#[macro_use] extern crate p_macro;
#[macro_use] extern crate log;
#[macro_use] extern crate maplit;
extern crate simplelog;
extern crate uuid;
extern crate rustc_serialize;
extern crate hyper;
extern crate rand;
extern crate webmachine_rust;
extern crate regex;
extern crate lazy_static;

#[cfg(test)]
extern crate quickcheck;

#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;

use clap::{Arg, App, SubCommand, AppSettings, ErrorKind, ArgMatches};
use std::env;
use std::str::FromStr;
use std::fs::{self, File};
use std::io;
use log::{LogLevelFilter};
use simplelog::{CombinedLogger, TermLogger, WriteLogger, SimpleLogger, Config};
use std::path::PathBuf;
use std::fs::OpenOptions;
use uuid::Uuid;
use pact_matching::models::PactSpecification;

fn display_error(error: String, matches: &ArgMatches) -> ! {
    println!("ERROR: {}", error);
    println!();
    println!("{}", matches.usage());
    panic!("{}", error)
}

mod server;
mod create_mock;
mod list;
mod verify;
mod shutdown;

fn print_version() {
    println!("\npact mock server version  : v{}", crate_version!());
    println!("pact specification version: v{}", PactSpecification::V1_1.version_str());
}

fn setup_log_file(output: Option<&str>) -> Result<File, io::Error> {
  let log_file = match output {
    Some(p) => {
      fs::create_dir_all(p)?;
      let mut path = PathBuf::from(p);
      path.push("pact_mock_server.log");
      path
    },
    None => PathBuf::from("pact_mock_server.log")
  };
  OpenOptions::new()
    .read(false)
    .write(true)
    .append(true)
    .create(true)
    .open(log_file)
}

fn setup_loggers(level: &str, command: &str, output: Option<&str>, no_file_log: bool, no_term_log: bool) -> Result<(), String> {
    let log_level = match level {
        "none" => LogLevelFilter::Off,
        _ => LogLevelFilter::from_str(level).unwrap()
    };

    if command == "start" {
      match (no_file_log, no_term_log) {
        (true, true) => {
          SimpleLogger::init(log_level, Config::default()).map_err(|e| format!("{:?}", e))
        },
        (true, false) => {
          TermLogger::init(log_level, Config::default()).map_err(|e| format!("{:?}", e))
        },
        (false, true) => {
          let log_file = setup_log_file(output).map_err(|e| format!("{:?}", e))?;
          WriteLogger::init(log_level, Config::default(), log_file).map_err(|e| format!("{:?}", e))
        },
        _ => {
          let log_file = setup_log_file(output).map_err(|e| format!("{:?}", e))?;
          match TermLogger::new(log_level, Config::default()) {
            Some(logger) => CombinedLogger::init(vec![logger, WriteLogger::new(log_level,
                                                                               Config::default(), log_file)]).map_err(|e| format!("{:?}", e)),
            None => WriteLogger::init(log_level, Config::default(), log_file).map_err(|e| format!("{:?}", e))
          }
        }
      }
    } else if no_term_log {
      SimpleLogger::init(log_level, Config::default()).map_err(|e| format!("{:?}", e))
    } else {
      TermLogger::init(log_level, Config::default()).map_err(|e| format!("{:?}", e))
    }
}

fn lookup_global_option<'a>(option: &str, matches: &'a ArgMatches<'a>) -> Option<&'a str> {
    let global_value = matches.value_of(option);
    let command_value = matches.subcommand().1.unwrap().value_of(option);
    match (global_value, command_value) {
        (Some(v), None) => Some(v),
        (None, Some(v)) => Some(v),
        (Some(_), Some(v)) => {
            println!("WARNING: The value for '{}' has been provided twice, defaulting to the second value '{}'",
                option, v);
            Some(v)
        },
        (None, None) => None
    }
}

fn global_option_present(option: &str, matches: &ArgMatches) -> bool {
  matches.is_present(option) || matches.subcommand().1.unwrap().is_present(option)
}

fn integer_value(v: String) -> Result<(), String> {
    v.parse::<u16>().map(|_| ()).map_err(|e| format!("'{}' is not a valid port value: {}", v, e) )
}

fn uuid_value(v: String) -> Result<(), String> {
    Uuid::parse_str(v.as_str()).map(|_| ()).map_err(|e| format!("'{}' is not a valid UUID value: {}", v, e) )
}

fn main() {
    match handle_command_args() {
        Ok(_) => (),
        Err(err) => std::process::exit(err)
    }
}

fn handle_command_args() -> Result<(), i32> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let version = format!("v{}", crate_version!());
    let app = App::new(program)
        .version(version.as_str())
        .about("Standalone Pact mock server")
        .version_short("v")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::ColoredHelp)
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .takes_value(true)
            .use_delimiter(false)
            .global(true)
            .help("port the master mock server runs on (defaults to 8080)"))
        .arg(Arg::with_name("host")
            .short("h")
            .long("host")
            .takes_value(true)
            .use_delimiter(false)
            .global(true)
            .help("hostname the master mock server runs on (defaults to localhost)"))
        .arg(Arg::with_name("loglevel")
            .short("l")
            .long("loglevel")
            .takes_value(true)
            .use_delimiter(false)
            .global(true)
            .possible_values(&["error", "warn", "info", "debug", "trace", "none"])
            .help("Log level for mock servers to write to the log file (defaults to info)"))
        .arg(Arg::with_name("no-term-log")
          .long("no-term-log")
          .global(true)
          .help("Use a simple logger instead of the term based one"))
        .arg(Arg::with_name("no-file-log")
          .long("no-file-log")
          .global(true)
          .help("Do not log to an output file"))
        .subcommand(SubCommand::with_name("start")
                .about("Starts the master mock server")
                .setting(AppSettings::ColoredHelp)
                .arg(Arg::with_name("output")
                      .short("o")
                      .long("output")
                      .takes_value(true)
                      .use_delimiter(false)
                      .help("the directory where to write files to (defaults to current directory)")))
        .subcommand(SubCommand::with_name("list")
                .about("Lists all the running mock servers")
                .setting(AppSettings::ColoredHelp))
        .subcommand(SubCommand::with_name("create")
                .about("Creates a new mock server from a pact file")
                .arg(Arg::with_name("file")
                    .short("f")
                    .long("file")
                    .takes_value(true)
                    .use_delimiter(false)
                    .required(true)
                    .help("the pact file to define the mock server"))
                .setting(AppSettings::ColoredHelp))
        .subcommand(SubCommand::with_name("verify")
                .about("Verify the mock server by id or port number, and generate a pact file if all ok")
                .arg(Arg::with_name("mock-server-id")
                    .short("i")
                    .long("mock-server-id")
                    .takes_value(true)
                    .use_delimiter(false)
                    .required_unless("mock-server-port")
                    .conflicts_with("mock-server-port")
                    .help("the ID of the mock server")
                    .validator(uuid_value))
                .arg(Arg::with_name("mock-server-port")
                    .short("m")
                    .long("mock-server-port")
                    .takes_value(true)
                    .use_delimiter(false)
                    .required_unless("mock-server-host")
                    .help("the port number of the mock server")
                    .validator(integer_value))
                .setting(AppSettings::ColoredHelp))
        .subcommand(SubCommand::with_name("shutdown")
                .about("Shutdown the mock server by id or port number, releasing all its resources")
                .arg(Arg::with_name("mock-server-id")
                    .short("i")
                    .long("mock-server-id")
                    .takes_value(true)
                    .use_delimiter(false)
                    .required_unless("mock-server-port")
                    .conflicts_with("mock-server-port")
                    .help("the ID of the mock server")
                    .validator(uuid_value))
                .arg(Arg::with_name("mock-server-port")
                    .short("m")
                    .long("mock-server-port")
                    .takes_value(true)
                    .use_delimiter(false)
                    .required_unless("mock-server-host")
                    .help("the port number of the mock server")
                    .validator(integer_value))
                .setting(AppSettings::ColoredHelp));

    let matches = app.get_matches_safe();
    match matches {
        Ok(ref matches) => {
            let log_level = lookup_global_option("loglevel", matches);
            if let Err(err) = setup_loggers(log_level.unwrap_or("info"),
                matches.subcommand_name().unwrap(),
                matches.subcommand().1.unwrap().value_of("output"),
                global_option_present("no-file-log", matches),
                global_option_present("no-term-log", matches)) {
                println!("WARN: Could not setup loggers: {}", err);
                println!();
            }
            let port = lookup_global_option("port", matches).unwrap_or("8080");
            let host = lookup_global_option("host", matches).unwrap_or("localhost");
            match port.parse::<u16>() {
                Ok(p) => {
                    match matches.subcommand() {
                        ("start", Some(sub_matches)) => {
                            server::start_server(p, sub_matches.value_of("output").map(|s| s.to_owned()))
                        },
                        ("list", Some(sub_matches)) => list::list_mock_servers(host, p, sub_matches),
                        ("create", Some(sub_matches)) => create_mock::create_mock_server(host, p, sub_matches),
                        ("verify", Some(sub_matches)) => verify::verify_mock_server(host, p, sub_matches),
                        ("shutdown", Some(sub_matches)) => shutdown::shutdown_mock_server(host, p, sub_matches),
                        _ => Err(3)
                    }
                },
                Err(_) => display_error(format!("{} is not a valid port number", port), matches)
            }
        },
        Err(ref err) => {
            match err.kind {
                ErrorKind::HelpDisplayed => {
                    println!("{}", err.message);
                    Ok(())
                },
                ErrorKind::VersionDisplayed => {
                    print_version();
                    println!();
                    Ok(())
                },
                _ => {
                    println!("{}", err.message);
                    err.exit()
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use quickcheck::{TestResult, quickcheck};
    use rand::Rng;
    use super::{integer_value, uuid_value};
    use expectest::prelude::*;

    #[test]
    fn validates_integer_value() {
        fn prop(s: String) -> TestResult {
            let mut rng = ::rand::thread_rng();
            if rng.gen() && s.chars().any(|ch| !ch.is_numeric()) {
                TestResult::discard()
            } else {
                let validation = integer_value(s.clone());
                match validation {
                    Ok(_) => TestResult::from_bool(!s.is_empty() && s.chars().all(|ch| ch.is_numeric() )),
                    Err(_) => TestResult::from_bool(s.is_empty() || s.chars().find(|ch| !ch.is_numeric() ).is_some())
                }
            }
        }
        quickcheck(prop as fn(_) -> _);

        expect!(integer_value(s!("1234"))).to(be_ok());
        expect!(integer_value(s!("1234x"))).to(be_err());
    }

    #[test]
    fn validates_uuid_value() {
        fn prop(s: String) -> TestResult {
            let mut rng = ::rand::thread_rng();
            if rng.gen() && s.chars().any(|ch| !ch.is_digit(16)) {
                TestResult::discard()
            } else {
                let validation = uuid_value(s.clone());
                match validation {
                    Ok(_) => TestResult::from_bool(!s.is_empty() && s.len() == 32 && s.chars().all(|ch| ch.is_digit(16) )),
                    Err(_) => TestResult::from_bool(s.is_empty() || s.len() != 32 || s.chars().find(|ch| !ch.is_digit(16) ).is_some())
                }
            }
        }
        quickcheck(prop as fn(_) -> _);

        expect!(uuid_value(s!("5159135ceb064af8a6baa447d81e4fbd"))).to(be_ok());
        expect!(uuid_value(s!("1234x"))).to(be_err());
    }

}
