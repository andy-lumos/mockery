use notify::{Watcher, RecursiveMode, RecommendedWatcher, DebouncedEvent};
use std::sync::mpsc::channel;
use std::time::Duration;
use colored::Colorize;
use std::path::PathBuf;

use crate::PUBLIC_DIR;
use crate::ws::notify_ws;


#[tokio::main]
pub async fn watch() {
  let (tx, rx) = channel();

  let mut watcher: RecommendedWatcher
                  = Watcher::new(tx, Duration::from_millis(400)).unwrap();
  
  let public_dir = PUBLIC_DIR.get().unwrap().as_path().to_str().unwrap();

  watcher.watch(public_dir, RecursiveMode::Recursive).unwrap();

  println!("[{}] Watcher is watching at {}", "SUCCESS".green(), public_dir);

  loop {
    use DebouncedEvent::*;

    match rx.recv() {
      Ok(event) => {
        match event {
          Write(path_buf) => {
            println!("[{}] {}", "UPDATE".blue(), get_rltv_path(path_buf).bright_black());
            notify_ws().await;
          },
          Create(path_buf) => {
            println!("[{}] {}", "CREATE".blue(), get_rltv_path(path_buf).bright_black());
            notify_ws().await;
          },
          Remove(path_buf) => {
            println!("[{}] {}", "REMOVE".blue(), get_rltv_path(path_buf).bright_black());
            notify_ws().await;
          },
          Rename(from_buf, to_buf) => {
            let message = format!(
              "{} -> {}", 
              get_rltv_path(from_buf), 
              get_rltv_path(to_buf)
            );
            println!("[{}] {}", "RENAME".blue(), message.bright_black());
            notify_ws().await;
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