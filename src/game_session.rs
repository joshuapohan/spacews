use std::sync::{Arc, Mutex, RwLock};
use std::fmt;
use actix::Addr;
use tokio::time::Duration;
use std::ops::{Deref, DerefMut};
use crate::game::common::NUM_COLS;
use crate::game::invaders::Invaders;
use crate::game::{frame::{Drawable, Frame}, player::Player};
use crate::server::GameSessionMessage;


#[derive(Clone)]
pub enum GameStateType {
    IDLE,
    START,
    STOP,
    WIN,
    LOSE
}

pub struct GameSession{
    pub room: String,
    pub last_frame: Arc<Mutex<Option<Frame>>>,
    pub player1: Option<Arc<Mutex<Player>>>,
    pub player2: Option<Arc<Mutex<Player>>>,
    pub invaders: Option<Arc<Mutex<Invaders>>>,
    pub state: Arc<RwLock<GameStateType>>,
    pub server_addr: Addr<crate::server::ChatServer>,
    pub player1_sessionid: usize,
    pub player2_sessionid: usize,
    pub score: usize,
}

impl fmt::Debug for GameSession {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Game session started")
    }
}

impl GameSession{
    pub fn render(&self){
        if let Some(frame) = self.last_frame.lock().unwrap().deref() {
            for row in frame{
                for col in row {
                    print!("{}", &col)
                }
                println!()
            }
        }
        println!();
        for _ in 0..NUM_COLS {
            print!("=")
        }
        println!();
    }

    pub fn new(room: String, server_addr: Addr<crate::server::ChatServer>) -> GameSession{

        let initial_frame = Arc::new(Mutex::new(Some(crate::game::frame::new_frame())));

        server_addr.do_send(GameSessionMessage{
            frame: initial_frame.clone(),
            room_id: room.clone(),
            state:GameStateType::START,
            player1_sessionid: 0,
            player2_sessionid: 0
        });


        let gs =  GameSession{
            server_addr: server_addr,
            room: room.clone(),
            last_frame: initial_frame.clone(),
            player1: None,
            player2: None,
            invaders: None,
            player1_sessionid: 0,
            player2_sessionid: 0,
            state: Arc::new(RwLock::new(GameStateType::IDLE)),
            score: 0,
        };

        gs
    }
    pub fn update_frame(&mut self, delta: Duration){
        let mut new_frame = crate::game::frame::new_frame();
        if let Some(p1) = &self.player1 {
            p1.lock().unwrap().update(delta);
            p1.lock().unwrap().draw(&mut new_frame);
        }

        if let Some(p2) = &self.player2 {
            p2.lock().unwrap().update(delta);
            p2.lock().unwrap().draw(&mut new_frame);
        }

        if let Some(invaders) = &self.invaders {
            invaders.lock().unwrap().update(delta);
            invaders.lock().unwrap().draw(&mut new_frame);
        }

        if let Some(p1) = &self.player1 {
            if let Some(invaders) = &self.invaders {
                self.score += p1.lock().unwrap().detect_hits(invaders.lock().unwrap().deref_mut()); 
            }
        }

        if let Some(p2) = &self.player2 {
            if let Some(invaders) = &self.invaders {
                self.score += p2.lock().unwrap().detect_hits(invaders.lock().unwrap().deref_mut()); 
            }
        }


        if let Some(invaders) = &self.invaders {
            if invaders.lock().unwrap().all_killed(){
                let mut state = self.state.write().unwrap();
                *state = GameStateType::WIN;
            }

            if invaders.lock().unwrap().reached_bottom(){
                let mut state = self.state.write().unwrap();
                *state = GameStateType::LOSE;
            }
        }
        
        {
            let mut binding = self.last_frame.lock().unwrap();    
            let last_frame = binding.deref_mut();    
            *last_frame = Some(new_frame);    
        }

        //let frame_json_binding = self.last_frame.lock().unwrap();
        //let frame_json = serde_json::to_string(frame_json_binding.deref()).unwrap();
        //self.render();
        
        self.server_addr.do_send(GameSessionMessage{
            frame: self.last_frame.clone(),
            room_id: self.room.clone(),
            state: self.state.read().unwrap().clone(),
            player1_sessionid: self.player1_sessionid,
            player2_sessionid: self.player2_sessionid
        });
    }
}