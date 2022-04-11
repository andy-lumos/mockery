use actix::prelude::*;
use actix_web_actors::ws;
use actix_web::{web, HttpRequest, Responder, HttpResponse};
use uuid::Uuid;

use crate::WS_ACTORS;


#[derive(Message)]
#[rtype(result = "()")]
struct Update;

pub struct WS;

impl Actor for WS {
  type Context = ws::WebsocketContext<Self>;
}

impl Handler<Update> for WS {
  type Result = ();

  fn handle(&mut self, _msg: Update, ctx: &mut ws::WebsocketContext<Self>) -> Self::Result {
    ctx.text("Update");
  }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WS {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match msg {
      Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
      _ => (),
    }
  }
}

pub async fn start_ws_connection(
  req: HttpRequest,
  stream: web::Payload,
) -> impl Responder {
  let ws = WS {};
  let res = ws::WsResponseBuilder::new(ws, &req, stream).start_with_addr();

  let (addr, resp) = match res {
    Ok(res) => res,
    Err(e) => return HttpResponse::from_error(e),
  };

  let uid = Uuid::new_v4();
  WS_ACTORS.lock().await.insert(uid, addr);

  resp
}

pub async fn send_update() -> Vec<Uuid> {
  let mut invalid_uids = vec![];
  let actors = WS_ACTORS.lock().await;

  for (uid, actor) in actors.iter() {
    if actor.connected() {
      let _r = actor.send(Update).await;
    } else {
      invalid_uids.push(uid.to_owned());
    }
  }

  invalid_uids
}

pub async fn remove_actors(uids: Vec<Uuid>) {
  let mut actors = WS_ACTORS.lock().await;

  for uid in uids {
    actors.remove(&uid);
  }
}

pub async fn notify_ws() {
  let uids = send_update().await;
  remove_actors(uids).await;
}