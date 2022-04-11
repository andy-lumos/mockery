use colored::Colorize;
use uuid::Uuid;
use std::io::Write;
use std::{io, fs};
use actix_web::{get, web, App, HttpServer, Error};
use actix_files;

use crate::MyResult;
use crate::PUBLIC_DIR;
use crate::HOST;
use crate::PORT;
// use crate::WS_CLIENTS;

pub async fn serve() -> MyResult<()> {
  let host = HOST.get().unwrap().to_owned();
  let mut port = PORT.get().unwrap().to_owned();

  run_server(&host, &mut port).await
}

async fn run_server(host: &str, port: &mut u16) -> MyResult<()> {

  loop {
    let server = HttpServer::new(
      || App::new()
          // .route("/ws-mockery", web::get().to(handle_ws))
          .service(static_assets)
    );
    let p = port.to_owned();
    match server.bind((host, p)) {
      Ok(server) => {
        println!("[{}] Server is running at: http://{host}:{port}", "SUCCESS".green());
        server.run().await?;
        break Ok(());
      },
      Err(e) => {
        match e.kind() {
          io::ErrorKind::AddrInUse => {
            println!("[{}] Port <{port}> is in use.", "WRANING".yellow());
            *port += 1;
          },
          _ => {
            println!("[ {} ] Failed to bind {host}:{port} : {e}", "ERROR".red());
          }
        }
      }
    }
  }
}

fn append_script(mut file: String) -> String {
  let host = HOST.get().unwrap().to_owned();
  let port = PORT.get().unwrap().to_owned();
  file.push_str(
    format!(
      r#"<!-- The code below is injected by Mockery -->
      <script>
        let flag = false;
        function init_connection() {{
          const ws = new WebSocket("ws://{host}:{port}/ws-mockery");
          ws.onopen = () => {{
            console.log("[Mockery Server] Connection Established");
            flag = true;
          }};
          ws.onmessage = () => location.reload();
          ws.onclose = () => {{
            flag = false;
            console.log("[Mockery Server] Connection Closed");
          }};
        }}
        init_connection();
        setInterval(() => {{
          if (!flag) {{
            console.log("[Mockery Server] Re-connecting");
            init_connection();
          }};
        }}, 5000)
      </script>"#)
    .as_str()
  );

  file
}

#[get("/{filename:.*}")]
async fn static_assets(params: web::Path<String>) -> Result<actix_files::NamedFile, Error> {

  let req_path = params.to_string();

  let public_dir = PUBLIC_DIR.get().unwrap().as_path().to_str().unwrap();
  let req_path = if req_path.ends_with("/") || req_path == "" {
    format!("{req_path}index.html") 
  } else { 
    format!("{req_path}")
  };

  let abs_path = format!("{public_dir}/{req_path}");

  let mime = mime_guess::from_path(&req_path).first_or_text_plain().to_string();

  if mime == "text/html" {
    let mut file = match fs::read_to_string(&abs_path) {
      Ok(data) => {
        println!("[{}] Get: {req_path}", "SUCCESS".green());
        data
      },
      Err(_) => {
        println!("[ {} ] Can not found: {req_path}", "ERROR".red());
        format!("Can not Get {req_path}")
      }
    };

    file = append_script(file);

    let fid = Uuid::new_v4().to_string() + "-" + req_path.as_str();

    let mut f = fs::File::create(&fid)?;
    f.write_all(file.as_bytes())?;

    let named_file = actix_files::NamedFile::open(&fid)?;
    fs::remove_file(&fid)?;

    Ok(named_file)
  } else {
    match fs::metadata(&abs_path) {
      Err(_) => println!("[ {} ] Can not found: {req_path}", "ERROR".red()),
      _ => {}
    }
    let named_file = actix_files::NamedFile::open(&abs_path)?;
    println!("[{}] Get: {req_path}", "SUCCESS".green());
    Ok(named_file)
  }
}
