use std::sync::{Arc, Mutex};
use std::fmt;
use tokio::time::Duration;
use std::ops::{Deref, DerefMut};
use crate::game::{frame::{Drawable, Frame}, player::Player};

pub struct GameSession{
    pub last_frame: Arc<Mutex<Option<Frame>>>,
    pub player1: Option<Arc<Mutex<Player>>>,
    pub player2: Option<Arc<Mutex<Player>>>,
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
    }

    pub fn new() -> GameSession{
        return GameSession{
            last_frame: Arc::new(Mutex::new(Some(crate::game::frame::new_frame()))),
            player1: None,
            player2: None,
        }
    }
    pub fn update_frame(&self, delta: Duration){
        let mut new_frame = crate::game::frame::new_frame();
        if let Some(p1) = &self.player1 {
            p1.lock().unwrap().update(delta);
        }

        if let Some(p2) = &self.player2 {
            p2.lock().unwrap().update(delta);
        }

        if let Some(p1) = &self.player1 {
            p1.lock().unwrap().draw(&mut new_frame);
        }

        if let Some(p2) = &self.player2 {
            p2.lock().unwrap().draw(&mut new_frame);
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