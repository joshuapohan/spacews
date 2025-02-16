use std::sync::{Arc, Mutex};
use std::fmt;
use tokio::time::Duration;
use std::ops::{Deref, DerefMut};
use crate::game::common::NUM_COLS;
use crate::game::invaders::Invaders;
use crate::game::{frame::{Drawable, Frame}, player::Player};


pub enum GameStateType {
    IDLE,
    START,
    WIN,
    LOSE
}

pub struct GameSession{
    pub last_frame: Arc<Mutex<Option<Frame>>>,
    pub player1: Option<Arc<Mutex<Player>>>,
    pub player2: Option<Arc<Mutex<Player>>>,
    pub invaders: Option<Arc<Mutex<Invaders>>>,
    pub state: Arc<Mutex<GameStateType>>
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

    pub fn new() -> GameSession{
        return GameSession{
            last_frame: Arc::new(Mutex::new(Some(crate::game::frame::new_frame()))),
            player1: None,
            player2: None,
            invaders: None,
            state: Arc::new(Mutex::new(GameStateType::IDLE)),
        }
    }
    pub fn update_frame(&self, delta: Duration){
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
                p1.lock().unwrap().detect_hits(invaders.lock().unwrap().deref_mut()); 
            }
        }


        if let Some(invaders) = &self.invaders {
            if invaders.lock().unwrap().all_killed(){
                *self.state.lock().unwrap() = GameStateType::WIN;
            }

            if invaders.lock().unwrap().reached_bottom(){
                *self.state.lock().unwrap() = GameStateType::LOSE;
            }
        }
        
        {
            let mut binding = self.last_frame.lock().unwrap();    
            let last_frame = binding.deref_mut();    
            *last_frame = Some(new_frame);    
        }

        //let frame_json_binding = self.last_frame.lock().unwrap();
        //let frame_json = serde_json::to_string(frame_json_binding.deref()).unwrap();
        self.render();
    }
}