use std::time::{Duration, Instant};
use actix::prelude::*;
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
const HEARTBEAT: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

use crate::server;

#[derive(Debug)]
pub struct WsChatSession {
    pub id: usize,
    pub hb: Instant,
    pub room: String,
    pub addr: Addr<server::ChatServer>,
}
#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub enum ChatType {
    TYPING,
    JOIN,
    TEXT,
    CONNECT,
    DISCONNECT,
    MOVEMENT,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatMessage {
    pub chat_type: ChatType,
    pub value: String,
}


impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);


        let addr = ctx.address();

        self.addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res: Result<usize, MailboxError>, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);

    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

impl Handler<server::Message> for WsChatSession {
    type Result = ();
    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match item {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let data_json = serde_json::from_str::<ChatMessage>(&text.to_string());
                if let Err(err) = data_json {
                    println!("{err}");
                    println!("Failed to parse message: {text}");
                    return;
                }

                let input = data_json.as_ref().unwrap();
                match &input.chat_type {
                    ChatType::JOIN => {
                        self.room = input.value.clone();
                        self.addr.do_send(server::ClientMessage {
                            id: self.id,
                            room: self.room.clone(),
                            msg_type: server::ClientMessageType::JOIN,
                        })
                    }
                    ChatType::MOVEMENT => {
                        self.addr.do_send(server::ClientMessage {
                            id: self.id,
                            room: self.room.clone(),
                            msg_type: server::ClientMessageType::MOVEMENT(input.value.clone()),
                        })
                    }
                    ChatType::TYPING => {
                        let chat_msg = ChatMessage {
                            chat_type: ChatType::TYPING,
                            value: input.value.clone(),
                        };
                        let msg = serde_json::to_string(&chat_msg).unwrap();
                        self.addr.do_send(server::ClientMessage {
                            id: self.id,
                            room: self.room.clone(),
                            msg_type: server::ClientMessageType::MESSAGE(msg),
                        })
                    }
                    ChatType::TEXT => {
                        let input = data_json.as_ref().unwrap();
                        let chat_msg = ChatMessage {
                            chat_type: ChatType::TEXT,
                            value: input.value.clone(),
                        };

                        let msg = serde_json::to_string(&chat_msg).unwrap();
                        self.addr.do_send(server::ClientMessage {
                            id: self.id,
                            room: self.room.clone(),
                            msg_type: server::ClientMessageType::MESSAGE(msg),                            
                        })
                    }
                    _ => {}
                }
            }
            ws::Message::Binary(_) => println!("Unsupported binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

impl WsChatSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                act.addr.do_send(server::Disconnect { id: act.id });
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}