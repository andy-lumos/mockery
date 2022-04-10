use clap::{Command, Arg};
use std::fs;
use local_ip_address::local_ip;
use std::path::PathBuf;
use colored::Colorize;
use once_cell::sync::{OnceCell, Lazy};
use async_std::sync::Mutex;
use async_std::task::block_on;
use std::collections::HashMap;
use uuid::Uuid;
use tide_websockets::WebSocketConnection;
use std::thread;

mod server;
mod watcher;

static PUBLIC_DIR: OnceCell<PathBuf> = OnceCell::new();
static HOST: OnceCell<String> = OnceCell::new();
static PORT: OnceCell<u16> = OnceCell::new();
static WS_CLIENTS: Lazy<Mutex<HashMap<Uuid, WebSocketConnection>>> 
                      = Lazy::new(|| Mutex::new(HashMap::new()));

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct Config {
  target: PathBuf,
  port: u16,
  host: String,
}

pub fn run(config: Config) -> MyResult<()> {

  PUBLIC_DIR.set(config.target.clone()).unwrap();
  HOST.set(config.host).unwrap();
  PORT.set(config.port).unwrap();

  thread::spawn(|| block_on(watcher::watch()));

  block_on(server::serve())?;

  Ok(())
}

pub fn get_args() -> MyResult<Config> {
  let matches = Command::new("mockery")
    .about("A server with mock data")
    .author("Andy Anderson <andy.anderson.dev@hotmail.com>")
    .version(env!("CARGO_PKG_VERSION"))
    .args([
      Arg::new("dir")
         .value_name("DIR")
         .help("The directory to watch.")
         .multiple_values(false)
         .required(false)
         .default_value("."),
      Arg::new("port")
         .value_name("PORT")
         .short('p')
         .long("port")
         .help("Server port")
         .multiple_values(false)
         .required(false)
         .default_value("8000")
    ])
    .get_matches();


  let config = Config {
    target: matches.value_of("dir")
                   .map(valid_path)
                   .transpose()?
                   .unwrap(),
    port  : matches.value_of("port")
                   .map(parse_u16)
                   .transpose()?
                   .unwrap(),
    host  : match local_ip() {
              Ok(ip) => ip.to_string(),
              _ => "localhost".into()
            }
  };

  Ok(config)
}

fn parse_u16(val: &str) -> MyResult<u16> {
  match val.parse::<u16>() {
    Ok(n) if n > 0 => Ok(n),
    _ => Err(From::from(format!("[ {} ] Invalid <Port>: {val}", "ERROR".red())))
  }
}

fn valid_path(path: &str) -> MyResult<PathBuf> {
  match fs::metadata(path) {
    Ok(metadata) if metadata.is_dir() => {
      match fs::canonicalize(PathBuf::from(path)) {
        Ok(buf) => Ok(buf),
        _ => Err(From::from(format!("[ {} ] Can't access <DIR>: {path}", "ERROR".red())))
      }
    },
    Ok(metadata) if !metadata.is_dir() => 
         Err(From::from(format!("[ {} ] Directory required: {path} ", "ERROR".red()))),
    _ => Err(From::from(format!("[ {} ] Can't access <DIR>: {path}", "ERROR".red())))
  }
}
