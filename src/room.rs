use std::sync::{Arc, Mutex};
use actix::Addr;
use tokio::task::{self, JoinHandle};
use tokio::time::{self, Duration, Instant};
use crate::game_session::GameSession;
use crate::server::{self};
use crate::game::player::Player;

#[derive(Debug)]
pub struct Room{
    pub name: String,
    pub player1: Option<Arc<Mutex<Player>>>,
    pub player2: Option<Arc<Mutex<Player>>>,
    pub ticker_handle: Option<JoinHandle<()>>,
    pub addr: Addr<crate::server::ChatServer>,
    pub game_session: Arc<Mutex<GameSession>>,
}

impl Room{
    pub fn new(name: String, sever_addr: Addr<server::ChatServer>) -> Room {
        let room = Self {
            name:  name,
            player1: None,
            player2: None,
            ticker_handle: None,
            addr: sever_addr,
            game_session: Arc::new(Mutex::new(GameSession::new()))
        };
        room
    }
    
    pub fn run_game_session_update_loop(&mut self){
        let game_sesion_loop  = self.game_session.clone();
        let repeating_task = task::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(100));
            let mut instant = Instant::now();
            loop {
                let delta = instant.elapsed();
                instant = Instant::now();
                game_sesion_loop.lock().unwrap().update_frame(delta);
                interval.tick().await;
            }
        });
        self.ticker_handle = Some(repeating_task);
    }    

    pub fn stop_update_loop(&mut self){
        if let Some(ticker_handle) = &self.ticker_handle {
            ticker_handle.abort();
            self.ticker_handle.take();
        }
    }

    pub fn disconnect_player(&mut self, player_id: usize){
        if let Some(p1) = &self.player1 {
            if p1.lock().unwrap().id == player_id {
                println!("Player 1 {} disconnected from  {} ", player_id, &self.name);
            }
            self.player1.take();
        } 
        if  let Some(p2) = &self.player2 {
            if p2.lock().unwrap().id == player_id {
                println!("Player 2 {} disconnected from room {} ", player_id, &self.name);
            }
            self.player2.take();
        }

        if self.player1.is_none() && self.player2.is_none() {
            println!("Both player disconnected from room {} , stopping game loop", self.name);
            self.stop_update_loop();
        }
    }

    pub fn join(&mut self, player: Arc<Mutex<Player>>){
        if self.player1.is_none() {
            self.player1 = Some(player.clone());
            self.game_session.lock().unwrap().player1.replace(player.clone());
            let mut player = player.lock().unwrap();
            player.room_id = Some(self.name.clone());
            println!("Player 1 {} joined room {} ", player.id, &self.name);
        } else if self.player2.is_none() {
            player.lock().unwrap().move_up(); // shift player 2 up
            self.player2 = Some(player.clone());
            self.game_session.lock().unwrap().player2.replace(player.clone());
            let mut player = player.lock().unwrap();
            player.room_id = Some(self.name.clone());            
            println!("Player 2 {} joined room {} ", player.id, &self.name);
        } else {
            println!("Unable to join room {} , already full", self.name);
        }
        if self.ticker_handle.is_none() {
            self.run_game_session_update_loop()
        }
    }
}