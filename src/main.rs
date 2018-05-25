// Copyright (c) 2017 Snowplow Analytics Ltd. All rights reserved.
//
// This program is licensed to you under the Apache License Version 2.0, and
// you may not use this file except in compliance with the Apache License
// Version 2.0.  You may obtain a copy of the Apache License Version 2.0 at
// http://www.apache.org/licenses/LICENSE-2.0.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the Apache License Version 2.0 is distributed on an "AS
// IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.  See the Apache License Version 2.0 for the specific language
// governing permissions and limitations there under.
//

#[macro_use]
extern crate log;
extern crate log4rs;
extern crate docopt;
extern crate getopts;
extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate url;
extern crate crypto;
extern crate threadpool;
extern crate iron;
#[macro_use(router)]
extern crate router;
extern crate bodyparser;
extern crate persistent;
extern crate logger;
extern crate rustc_serialize;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate consul;
extern crate base64;

use docopt::Docopt;
use log::LogLevelFilter;
use regex::Regex;

mod factotum_server;

#[cfg(test)]
mod tests;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const FACTOTUM: &'static str = "factotum";

const IP_DEFAULT: &'static str = "0.0.0.0";
const PORT_DEFAULT: u32 = 3000;
const MAX_JOBS_DEFAULT: usize = 1000;
const MAX_WORKERS_DEFAULT: usize = 20;

const CONSUL_NAME_DEFAULT: &'static str = FACTOTUM;
const CONSUL_IP_DEFAULT: &'static str = "127.0.0.1";
const CONSUL_PORT_DEFAULT: u32 = 8500;
const CONSUL_NAMESPACE_DEFAULT: &'static str = "com.snowplowanalytics/factotum";

const SERVER_STATE_RUN: &'static str = "run";
const SERVER_STATE_DRAIN: &'static str = "drain";

const JSON_CONTENT_TYPE: &'static str = "application/json; charset=UTF-8";

const VALID_IP_REGEX: &'static str = r"\b(?:(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\b";

const USAGE: &'static str =
    "
Factotum Server.

Usage:
  factotum-server --factotum-bin=<path> [--ip=<address>] [--port=<number>] [--max-jobs=<size>] [--max-workers=<size>] [--webhook=<url>] [--no-colour] [--consul-name=<name>] [--consul-ip=<address>] [--consul-port=<number>] [--consul-namespace=<namespace>] [--log-level=<level>] [--max-stdouterr-size=<bytes>]
  factotum-server (-h | --help)
  factotum-server (-v | --version)

Options:
  -h --help                             Show this screen.
  -v --version                          Display the version of Factotum Server and exit.
  --ip=<address>                        Specify binding IP address.
  --port=<number>                       Specify port number.
  --log-level=<level>                   Specify logging level.
  --max-jobs=<size>                     Max size of job requests queue.
  --max-workers=<size>                  Max number of workers.
  --factotum-bin=<path>                 Path to Factotum binary file.
  --webhook=<url>                       Factotum arg to post updates on job execution to the specified URL.
  --no-colour                           Factotum arg to turn off ANSI terminal colours/formatting in output.
  --consul-name=<name>                  Specify node name of Consul server agent.
  --consul-ip=<address>                 Specify IP address for Consul server agent.
  --consul-port=<number>                Specify port number for Consul server agent.
  --consul-namespace=<namespace>        Specify namespace of job references stored in Consul persistence.
  --max-stdouterr-size=<bytes>          The maximum size of the individual stdout/err sent via the webhook functions for job updates.
";

#[derive(Debug, RustcDecodable)]
pub struct Args {
    flag_version: bool,
    flag_ip: Option<String>,
    flag_port: u32,
    flag_log_level: Option<String>,
    flag_max_jobs: usize,
    flag_max_workers: usize,
    flag_factotum_bin: String,
    flag_webhook: String,
    flag_no_colour: bool,
    flag_consul_name: Option<String>,
    flag_consul_ip: Option<String>,
    flag_consul_port: Option<u32>,
    flag_consul_namespace: Option<String>,
    flag_max_stdouterr_size: Option<usize>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("Factotum Server version [{}]", VERSION);
    } else {
        match check_and_init_logger(&args) {
            Ok(_) => {},
            Err(msg) => {
                println!("{}", msg);
                std::process::exit(1)
            }
        };
        match factotum_server::start(args) {
            Ok(_) => {},
            Err(msg) => println!("{}", msg)
        }
    }
}

// --- Helpers ---

fn check_and_init_logger(args: &Args) -> Result<(), String> {
    match check_factotum_bin_arg(&args.flag_factotum_bin) {
        Ok(..) => {},
        Err(e) => return Err(e),
    };
    match check_ip_arg(&args.flag_ip) {
        Ok(..) => {},
        Err(e) => return Err(e),
    };
    match check_ip_arg(&args.flag_consul_ip) {
        Ok(..) => {},
        Err(e) => return Err(e),
    };
    match init_logger(&args.flag_log_level) {
        Ok(..) => {},
        Err(e) => return Err(e),
    };
    Ok(())
}

fn check_factotum_bin_arg(factotum_bin: &str) -> Result<(), String> {
    if !std::path::Path::new(factotum_bin).exists() {
        return Err(format!("Invalid path for Factotum binary at: '{}'", factotum_bin))
    }
    Ok(())
}

fn check_ip_arg(wrapped_ip: &Option<String>) -> Result<(), String> {
    if let Some(ip) = wrapped_ip.as_ref() {
        if !is_a_valid_ip(&ip) {
            return Err(format!("Invalid IP address: [{}] - Regex mismatch", ip))
        }
    }
    Ok(())
}

fn is_a_valid_ip(text: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(::VALID_IP_REGEX).expect("Error building regex");
    }
    RE.is_match(text)
}

fn init_logger(level_input: &Option<String>) -> Result<(), String> {
    let log_level = match get_log_level(level_input) {
        Ok(level) => level,
        Err(e) => return Err(e),
    };
    let log_config = match get_log_config(log_level) {
        Ok(config) => config,
        Err(e) => return Err(format!("Error building log config: {}", e.to_string())),
    };
    match log4rs::init_config(log_config) {
        Ok(..) => Ok(()),
        Err(e) => return Err(format!("Error initialising log config: {}", e.to_string())),
    }
}

fn get_log_level(level_input: &Option<String>) -> Result<LogLevelFilter, String> {
    let log_level = match level_input.as_ref() {
        Some(input) => input,
        None => return Ok(LogLevelFilter::Warn),
    };
    let result = match log_level.to_lowercase().as_ref() {
        "off" => LogLevelFilter::Off,
        "error" => LogLevelFilter::Error,
        "warn" => LogLevelFilter::Warn,
        "info" => LogLevelFilter::Info,
        "debug" => LogLevelFilter::Debug,
        "trace" => LogLevelFilter::Trace,
        _ => {
            return Err(format!("Unknown log level: '{}'\nPlease select a valid log level.", log_level))
        },
    };
    Ok(result)
}

fn get_log_config(log_level: LogLevelFilter) -> Result<log4rs::config::Config, log4rs::config::Errors> {
    use log4rs::append::console::ConsoleAppender;
    use log4rs::encode::pattern::PatternEncoder;
    use log4rs::config::{Appender, Config, Root};
    
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {l:>5} - {m}{n}")))
        .build();

    let root = Root::builder()
        .appender("stdout")
        .build(log_level);

    Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(root)
}
