use notify::{Watcher, RecursiveMode, RecommendedWatcher, DebouncedEvent};
use std::sync::mpsc::channel;
use std::time::Duration;
use colored::Colorize;
use std::path::PathBuf;

use crate::PUBLIC_DIR;
// use crate::WS_CLIENTS;


async fn broadcast() {
  // let clients = WS_CLIENTS.lock().await;
  // for (_, client) in clients.iter() {
  //   client.send(Message::Text("".into())).await.unwrap();
  // }
}

#[tokio::main]
pub async fn watch() {
  let (tx, rx) = channel();

  let mut watcher: RecommendedWatcher
                  = Watcher::new(tx, Duration::from_millis(400)).unwrap();
  
  let public_dir = PUBLIC_DIR.get().unwrap().as_path().to_str().unwrap();

  watcher.watch(public_dir, RecursiveMode::Recursive).unwrap();

  loop {
    use DebouncedEvent::*;

    match rx.recv() {
      Ok(event) => {
        match event {
          Write(path_buf) => {
            println!("[{:7}] {}", "UPDATE".blue(), get_rltv_path(path_buf).bright_black());
            broadcast().await;
          },
          Create(path_buf) => {
            println!("[{:7}] {}", "CREATE".blue(), get_rltv_path(path_buf).bright_black());
            broadcast().await;
          },
          Remove(path_buf) => {
            println!("[{:7}] {}", "REMOVE".blue(), get_rltv_path(path_buf).bright_black());
            broadcast().await;
          },
          Rename(from_buf, to_buf) => {
            println!(
              "[{:7}] {} -> {}", 
              "RENAME".blue(), 
              get_rltv_path(from_buf), 
              get_rltv_path(to_buf).bright_black()
            );
            broadcast().await;
          },
          Error(err, _) => println!("[ {} ] {err}", "ERROR".red()),
          _ => {}
        }
      },
      Err(e) => println!("[ {} ] Failed to watch file change: {e}", "ERROR".red()),
    }
  }
}

fn get_rltv_path(path_buf: PathBuf) -> String {
  let public_dir = PUBLIC_DIR.get().unwrap().as_path().to_str().unwrap();
  let cur_path = path_buf.as_path().to_str().unwrap();

  cur_path[(public_dir.len() + 1)..].into()
}