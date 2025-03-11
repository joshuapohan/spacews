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
    pub player1_session_id: usize,
    pub player2_session_id: usize,
    pub ticker_handle: Option<JoinHandle<()>>,
    pub addr: Addr<crate::server::ChatServer>,
    pub game_session: Arc<Mutex<GameSession>>,
}

impl Room{
    pub fn new(name: String, server_addr: Addr<server::ChatServer>) -> Room {
        let room = Self {
            name:  name.clone(),
            player1: None,
            player2: None,
            player1_session_id: 0,
            player2_session_id: 0,
            ticker_handle: None,
            addr: server_addr.clone(),
            game_session: Arc::new(Mutex::new(GameSession::new(name.clone(), server_addr.clone())))
        };
        room
    }
    
    pub fn run_game_session_update_loop(&mut self){
        let invaders = crate::game::invaders::Invaders::new();
        let game_sesion_loop  = self.game_session.clone();
        self.game_session.lock().unwrap().invaders = Some(Arc::new(Mutex::new(invaders)));
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
                self.player1.take();
                let mut gs = self.game_session.lock().unwrap();
                gs.player1.take();
                gs.player1_sessionid = 0;
            }
        } 
        if  let Some(p2) = &self.player2 {
            if p2.lock().unwrap().id == player_id {
                println!("Player 2 {} disconnected from room {} ", player_id, &self.name);
                self.player2.take();
                let mut gs = self.game_session.lock().unwrap();
                gs.player2.take();
                gs.player2_sessionid = 0;
            }
        }

        if self.player1.is_none() && self.player2.is_none() {
            println!("Both player disconnected from room {} , stopping game loop", self.name);
            self.stop_update_loop();
        }
    }

    pub fn join2(&mut self, player: Arc<Mutex<Player>>){
        if self.player1.is_none() {
            self.player1 = Some(player.clone());
            self.game_session.lock().unwrap().player1.replace(player.clone());
            self.game_session.lock().unwrap().player1_sessionid = player.lock().unwrap().id;
            let mut player = player.lock().unwrap();
            player.room_id = Some(self.name.clone());
            println!("Player 1 {} joined room {} ", player.id, &self.name);
        } else if self.player2.is_none() {
            player.lock().unwrap().move_up(); // shift player 2 up
            self.player2 = Some(player.clone());
            self.game_session.lock().unwrap().player2.replace(player.clone());
            self.game_session.lock().unwrap().player2_sessionid = player.lock().unwrap().id;
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

    pub fn join(&mut self, session_id: usize){
        if self.player1.is_none() {
            let mut player = Player::new(session_id);
            player.room_id = Some(self.name.clone());
            let player_arc = Arc::new(Mutex::new(player)); 
            self.player1 = Some(player_arc.clone());
            self.player1_session_id = session_id;
            if let Ok(mut gs) = self.game_session.lock() {
                gs.player1.replace(player_arc.clone());
                gs.player1_sessionid = self.player1_session_id;
            }
            println!("Player 1 {} joined room {} ", session_id, &self.name);
        } else if self.player2.is_none() {
            let mut player = Player::new(session_id);
            player.y = player.y - 1;
            player.room_id = Some(self.name.clone());
            let player_arc = Arc::new(Mutex::new(player)); 
            self.player2 = Some(player_arc.clone());
            self.player2_session_id = session_id;
            if let Ok(mut gs) = self.game_session.lock() {
                gs.player2.replace(player_arc.clone());
                gs.player2_sessionid = self.player2_session_id;
            }
            println!("Player 2 {} joined room {} ", session_id, &self.name);
         }
        if self.ticker_handle.is_none() {
            self.run_game_session_update_loop();
        }
    }

    pub fn handle_player_input(&mut self, session_id: &usize, command: &str){
        if *session_id == self.player1_session_id {
            match &self.player1 {
                Some(player) => {
                    match player.lock(){
                        Ok(mut p1) => {
                            p1.handle_movement(command);
                            ()
                        },
                        Err(_) => println!("[ERROR] handle_player_input: player 1 failed to get mutex {}", session_id),
                    }
                },
                None => println!("[ERROR] handle_player_input: player 1 not found {}", session_id),
            }

        } else if *session_id == self.player2_session_id {
            match &self.player2 {
                Some(player) => {
                    match player.lock(){
                        Ok(mut p2) => {
                            p2.handle_movement(command);
                            ()
                        },
                        Err(_) => println!("[ERROR] handle_player_input: player 2 failed to get mutex {}", session_id),
                    }
                },
                None => println!("[ERROR] handle_player_input: player 2 not found {}", session_id),
            }
        }

    }

}