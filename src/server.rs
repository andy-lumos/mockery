use tide::{self, listener::Listener, Server, Request, Response};
use colored::Colorize;
use uuid::Uuid;
use std::{io, fs};
use tide_websockets::WebSocket;
use async_std::prelude::*;

use crate::MyResult;
use crate::PUBLIC_DIR;
use crate::HOST;
use crate::PORT;
use crate::WS_CLIENTS;


pub async fn serve() -> MyResult<()> {
  let host = HOST.get().unwrap().to_owned();
  let mut port = PORT.get().unwrap().to_owned();
  let mut listener = create_listener(host, &mut port).await;

  listener.accept().await.unwrap();

  Ok(())
}

fn create_server() -> Server<()> {
  let mut server = tide::new();
  server.at("/").get(static_assets);
  server.at("/*").get(static_assets);
  server.at("/ws-mockery").get(
    WebSocket::new(
      |_request, mut stream| async move {
        let uid = Uuid::new_v4();

        WS_CLIENTS.lock().await.insert(uid, stream.clone());
        while let Some(Ok(_)) = stream.next().await {}
        WS_CLIENTS.lock().await.remove(&uid);

        Ok(())
      }
    )
  );

  server
}

async fn create_listener(host: String, port: &mut u16) -> impl Listener<()> {
  loop {
    let server = create_server();

    match server.bind(format!("{host}:{port}")).await {
      Ok(listener) => {
        println!("[{}] Server is running at: http://{host}:{port}", "SUCCESS".green());
        break listener;
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

async fn static_assets(req: Request<()>) -> tide::Result {
  let public_dir = PUBLIC_DIR.get().unwrap().as_path().to_str().unwrap();
  let req_path = req.url().path();

  let req_path = if req_path.ends_with("/") { 
    format!("{req_path}index.html") 
  } else { 
    format!("{req_path}")
  };

  let abs_path = format!("{public_dir}{req_path}");

  let mut file = match fs::read_to_string(abs_path) {
    Ok(data) => {
      println!("[{}] Get: {req_path}", "SUCCESS".green());
      data
    },
    Err(_) => {
      println!("[ {} ] Can not found: {req_path}", "ERROR".red());
      format!("Can not Get {req_path}")
    }
  };

  let mime = mime_guess::from_path(req_path).first_or_text_plain().to_string();

  file = if mime == "text/html" {
    append_script(file)
  } else { file };

  let mut res: Response = file.into();
  res.set_content_type(mime.as_str());

  Ok(res)
}