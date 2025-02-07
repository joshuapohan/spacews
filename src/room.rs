use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use actix::Addr;
use tokio::task::{self, JoinHandle};
use tokio::time::{self, Duration, Instant};
use crate::server::{self};
use crate::{frame::{Drawable, Frame}, player::Player};

#[derive(Debug)]
pub struct Room{
    pub name: String,
    pub last_frame: Arc<Mutex<Option<Frame>>>,
    pub player1: Option<Arc<Mutex<Player>>>,
    pub player2: Option<Arc<Mutex<Player>>>,
    pub ticker_handle: Option<JoinHandle<()>>,
    pub addr: Addr<crate::server::ChatServer>,
}

impl Room{
    pub fn new(name: String, sever_addr: Addr<server::ChatServer>) -> Arc<Mutex<Self>> {
        let room = Self {
            name:  name,
            last_frame: Arc::new(Mutex::new(Some(crate::frame::new_frame()))),
            player1: None,
            player2: None,
            ticker_handle: None,
            addr: sever_addr,
        };
        let room = Arc::new(Mutex::new(room));
        let room_loop = room.clone();
        Room::run_update_loop(room_loop);
        /* 
        let repeating_task = task::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(100));
            let mut instant = Instant::now();
            loop {
                let delta = instant.elapsed();
                instant = Instant::now();
                room_loop.lock().unwrap().update_frame(delta);
                interval.tick().await;
            }
        });
        room.clone().lock().unwrap().ticker_handle = Some(repeating_task);
        */
        room.clone()
    }

    pub fn run_update_loop(a: Arc<Mutex<Self>>){
        let room_loop_arc  = a.clone();
        let repeating_task = task::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(100));
            let mut instant = Instant::now();
            loop {
                let delta = instant.elapsed();
                instant = Instant::now();
                room_loop_arc.lock().unwrap().update_frame(delta);
                interval.tick().await;
            }
        });
        a.clone().lock().unwrap().ticker_handle = Some(repeating_task);
    }

    /*
    pub fn run_update_loop_2(&mut self){

        let repeating_task = task::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(100));
            let mut instant = Instant::now();
            loop {
                let delta = instant.elapsed();
                instant = Instant::now();
                room.lock().unwrap().update_frame(delta);
                interval.tick().await;
            }
        });
        self.ticker_handle = Some(repeating_task);
    }
    */

    /*/
    pub fn stop_update_loop(a: Arc<Mutex<Self>>){
        if let Some(ticker_handle) = &a.lock().unwrap().ticker_handle {
            ticker_handle.abort();
        }
    }
    */    


    pub fn stop_update_loop(&mut self){
        if let Some(ticker_handle) = &self.ticker_handle {
            ticker_handle.abort();
            self.ticker_handle.take();
        }
    }

    pub fn update_frame(&self, delta: Duration){
        let mut new_frame = crate::frame::new_frame();
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

        self.render();
        let frame_json_binding = self.last_frame.lock().unwrap();
        let frame_json = serde_json::to_string(frame_json_binding.deref()).unwrap();
        self.addr.do_send(server::RoomMessage{room_id: self.name.clone(), msg:frame_json});
    }

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
            let mut player = player.lock().unwrap();
            player.room_id = Some(self.name.clone());
            println!("Player 1 {} joined room {} ", player.id, &self.name);
        } else if self.player2.is_none() {
            self.player2 = Some(player.clone());
            let mut player = player.lock().unwrap();
            player.room_id = Some(self.name.clone());            
            println!("Player 2 {} joined room {} ", player.id, &self.name);
        } else {
            println!("Unable to join room {} , already full", self.name);
        }

    }
}