use std::time::Instant;
use actix::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use rand::Rng;

use crate::server;
use crate::session;

pub async fn chat_server(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
) -> Result<HttpResponse, Error> {
    let id = rand::thread_rng().gen_range(0..500);
    ws::start(
        session::WsChatSession {
            id: id,
            hb: Instant::now(),
            room: "main".to_string(),
            addr: srv.get_ref().clone(),
        },
        &req,
        stream
    )
}