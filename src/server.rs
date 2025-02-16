use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}};
use dashmap::DashMap;
use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use crate::{game::{frame::Frame, player::Player}, game_session::GameStateType};
use crate::room::Room;


#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

pub enum ClientMessageType{
    MESSAGE(String),
    MOVEMENT(String),
    JOIN,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub id: usize,
    pub room: String,
    pub msg_type: ClientMessageType,
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct RoomMessage {
    pub room_id: String,
    pub msg: String,
}


#[derive(Message)]
#[rtype(result= "()")]
pub struct GameSessionMessage {
    pub room_id: String,
    pub frame: Arc<Mutex<Option<Frame>>>,
    pub state: GameStateType,
    pub player1_sessionid: usize,
    pub player2_sessionid: usize
}

#[derive(Debug)]
pub struct ChatServer{
    sessions: HashMap<usize, Recipient<Message>>,
    rng:  ThreadRng,
    rooms: DashMap<String , HashSet<usize>>,
    game_rooms: DashMap<String , Room>,
    players: DashMap<usize, Arc<Mutex<Player>>>,
}

impl ChatServer {
    pub fn new() -> ChatServer {
        let rooms = DashMap::new();
        let game_rooms = DashMap::new();
        let players = DashMap::new();
        rooms.insert("main".to_string(), HashSet::new());
        Self {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
            rooms,
            game_rooms,
            players,
        }
    }

    fn send_message(&self, room: &str, message: &str) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions.iter() {
                if let Some(addr) = self.sessions.get(&id) {
                    addr.do_send(Message(message.to_owned()))
                }
            }
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = usize;
    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);
        self.players.insert(id, Arc::new(Mutex::new(Player::new(id))));
        id
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        if self.sessions.remove(&msg.id).is_some() {
            println!("Player disconnected 1");
            for mut v in self.rooms.iter_mut() {
                let ( _name,  sessions) =  v.pair_mut();
                if sessions.remove(&msg.id) {

                }
            }
        }
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: ClientMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg.msg_type {
            ClientMessageType::MESSAGE(text_message) => self.send_message(&msg.room, &text_message),
            ClientMessageType::MOVEMENT(mov)=> {
                if let Some(player) = self.players.get(&msg.id){
                    if let Some(_) = self.game_rooms.get_mut(msg.room.as_str()){
                        match mov.as_str() {
                            "-1" => {
                                player.lock().unwrap().move_left();
                            }
                            "1" => {
                                player.lock().unwrap().move_right();
                            }
                            "-" => {
                                player.lock().unwrap().shoot();
                            }
                            _ => {}
                        }
                    }
                }
            },
            ClientMessageType::JOIN => {
                if let Some(player) = self.players.get(&msg.id){
                    let mut room =  self.game_rooms.entry(msg.room.clone()).or_insert(Room::new(msg.room.clone(), ctx.address()));
                    room.join(player.clone());
                }
            },
        }
    }
}

impl Handler<GameSessionMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: GameSessionMessage, _: &mut Self::Context) -> Self::Result {

        match self.game_rooms.get_mut(msg.room_id.as_str()){
            Some(mut room) => {
                match msg.state {
                    GameStateType::IDLE => (),
                    GameStateType::START => (),
                    GameStateType::WIN => {
                        println!("[INFO] GAME WON Room [{}]", msg.room_id.as_str());
                        room.stop_update_loop()
                    },
                    GameStateType::LOSE => {
                        println!("[INFO] GAME LOST Room [{}]", msg.room_id.as_str());                        
                        room.stop_update_loop()
                    },
                }

                match msg.frame.lock() {
                    Ok(frame) => {
                        let res  = serde_json::to_string(&frame.as_deref()).unwrap();

                        match self.sessions.get(&msg.player1_sessionid) {
                            Some(session) => session.do_send(Message{0:res.clone()}),
                            None => {
                                if msg.player1_sessionid != 0 {
                                    println!("[INFO] Room [{}] Player 1 disconnected", room.name);
                                    room.disconnect_player(msg.player1_sessionid);
                                }
                            },
                        }

                        match self.sessions.get(&msg.player2_sessionid) {
                            Some(session) => session.do_send(Message{0:res.clone()}),
                            None => {
                                if msg.player2_sessionid != 0 {
                                    println!("[INFO] Room [{}] Player 2 disconnected", room.name);
                                    room.disconnect_player(msg.player2_sessionid);                                    
                                }
                            },
                        }

                    },
                    Err(_) => print!("[ERROR] No frame data for room [{}]", msg.room_id.clone()),
                }
            },
            None => println!("[ERROR] ChatServer : missing game room from message [{}]", msg.room_id.clone()),
        }
    }
}