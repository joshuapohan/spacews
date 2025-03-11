use std::{collections::{HashMap, HashSet}, ops::Deref, sync::{Arc, Mutex}};
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
#[rtype(result= "()")]
pub struct GameSessionMessage {
    pub room_id: String,
    pub frame: Arc<Mutex<Option<Frame>>>,
    pub state: GameStateType,
    pub player1_sessionid: usize,
    pub player2_sessionid: usize,
}

#[derive(Debug)]
pub struct ChatServer{
    sessions: HashMap<usize, Recipient<Message>>,
    rng:  ThreadRng,
    rooms: DashMap<String , HashSet<usize>>,
    game_rooms: DashMap<String , Room>,
    active_games: DashMap<String, bool>
}

impl ChatServer {
    pub fn new() -> ChatServer {
        let rooms = DashMap::new();
        let game_rooms = DashMap::new();
        rooms.insert("main".to_string(), HashSet::new());
        let active_games = DashMap::new();
        Self {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
            rooms,
            game_rooms,
            active_games,
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
                    if let Some(mut room) = self.game_rooms.get_mut(msg.room.as_str()){
                        room.handle_player_input(&msg.id, mov.as_str());
                    }
            },
            ClientMessageType::JOIN => {
                let mut room =  self.game_rooms.entry(msg.room.clone()).or_insert(Room::new(msg.room.clone(), ctx.address()));
                room.join(msg.id);
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
                    GameStateType::START => {
                        println!("[INFO] GAME STARTED Room [{}]", msg.room_id.as_str());
                        self.active_games.insert(msg.room_id.clone(), true);
                        println!("[INFO] Active games count : {}", self.active_games.len());
                        return ()
                    },
                    GameStateType::STOP => {
                        println!("[INFO] GAME STOPPED Room [{}]", msg.room_id.as_str());
                        self.active_games.remove(msg.room_id.as_str());
                        println!("[INFO] Active games count : {}", self.active_games.len());
                    },
                    GameStateType::WIN => {
                        println!("[INFO] GAME WON Room [{}]", msg.room_id.as_str());
                        room.stop_update_loop();
                        self.active_games.remove(msg.room_id.as_str());
                        println!("[INFO] Active games count : {}", self.active_games.len());
                    },
                    GameStateType::LOSE => {
                        println!("[INFO] GAME LOST Room [{}]", msg.room_id.as_str());
                        room.stop_update_loop();
                        self.active_games.remove(msg.room_id.as_str());
                        println!("[INFO] Active games count : {}", self.active_games.len());
                    },
                }

                match msg.frame.lock() {                    
                    Ok(frame) => {
                        let mut player1_connected = false;
                        let mut player2_connected = false;

                        let res  = serde_json::to_string(frame.deref()).unwrap();

                        match self.sessions.get(&msg.player1_sessionid) {
                            Some(session) => {
                                player1_connected = true;
                                session.do_send(Message{0:res.clone()})
                            },
                            None => {
                                if msg.player1_sessionid != 0 {
                                    println!("[INFO] Room [{}] Player 1 disconnected", room.name);
                                    room.disconnect_player(msg.player1_sessionid);

                                }
                            },
                        }

                        match self.sessions.get(&msg.player2_sessionid) {
                            Some(session) => {
                                player2_connected = true;                                
                                session.do_send(Message{0:res.clone()});
                            },
                            None => {
                                if msg.player2_sessionid != 0 {
                                    println!("[INFO] Room [{}] Player 2 disconnected", room.name);
                                    room.disconnect_player(msg.player2_sessionid);
                                    player2_connected = false;
                                }
                            },
                        }

                        if !(player1_connected || player2_connected){
                            println!("[INFO] GAME Room Empty [{}]", msg.room_id.as_str());
                            self.active_games.remove(msg.room_id.as_str());
                            println!("[INFO] Active games count : {}", self.active_games.len());
                        }


                    },
                    Err(_) => print!("[ERROR] No frame data for room [{}]", msg.room_id.clone()),
                }
            },
            None => println!("[ERROR] ChatServer : missing game room from message [{}]", msg.room_id.clone()),
        }
    }
}