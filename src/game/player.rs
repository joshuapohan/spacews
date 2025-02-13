use std::{fmt, time::Duration};


use crate::game::{frame::{Drawable, Frame}, shot::Shot, common::NUM_COLS, common::NUM_ROWS};

pub struct Player {
    pub id: usize,
    pub x: usize,
    pub y:usize,
    pub shots: Vec<Shot>,
    pub room_id: Option<String>
}


impl fmt::Debug for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hi: Player {}", self.id)
    }
}


impl Player {
    pub fn new(id: usize) -> Self {
        Self {
            id: id,
            x: NUM_COLS / 2,
            y: NUM_ROWS -1,
            shots: Vec::new(),
            room_id: None,
        }
    }

    pub fn move_up(&mut self){
        if self.y > 0 {
            self.y -= 1;
        }
    }

    pub fn move_left(&mut self){
        if self.x > 0 {
            self.x -= 1;
        }
    }

    pub fn move_right(&mut self){
        if self.x < NUM_COLS - 1{
            self.x += 1;
        }
    }

    pub fn shoot(&mut self) -> bool {
        if self.shots.len() < 20 {
            self.shots.push(Shot::new(self.x, self.y - 1));
            true
        } else {
            false
        }
    }
    pub fn update(&mut self, delta: Duration) {
        for shot in self.shots.iter_mut(){
            shot.update(delta);
        }
        self.shots.retain(|shot| !shot.dead());
    }
}

impl Drawable for Player {
    fn draw(&self, frame: &mut Frame){
        frame[self.y][self.x] = "A";
        for shot in self.shots.iter() {
            shot.draw(frame);
        }
    }
}