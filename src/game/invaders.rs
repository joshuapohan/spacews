use std::{cmp::max, time::Duration};

use rusty_time::timer::Timer;

use crate::game::common::{NUM_COLS , NUM_ROWS};
use crate::game::frame::{Drawable, Frame};

pub struct Invader {
    x: usize,
    y: usize,
}

pub struct Invaders {
    pub army: Vec<Invader>,
    move_timer: Timer,
    direction: i32,
    stop: bool,
}

impl Invaders {
    pub fn new() -> Self {
        let mut army = Vec::new();
        for x in 0..NUM_COLS {
            for y in 0..NUM_ROWS {
                if(y > 1)
                    && (y < NUM_ROWS / 2)
                    && (x >  0)
                    && (x < NUM_COLS)
                    && (x % 2 == 0)
                    && (y % 2 == 0){
                        army.push(Invader{x, y});
                    }
            }
        }

        Self { 
            army,
            move_timer: Timer::from_millis(2500), 
            direction: 1, 
            stop: false
        }
    }


    pub fn update(&mut self, delta: Duration) -> bool {
        if self.stop {
            return true
        }
        self.move_timer.update(delta);
        if self.move_timer.ready {
            self.move_timer.reset();
            let mut downwards = false;
            if self.direction == -1 {
                let min_x = self.army.iter()
                    .map(|invader| invader.x)
                    .min().unwrap_or(0);
                if min_x == 0 {
                    self.direction = 1;
                    downwards = true;
                }
            } else {
                let max_x = self.army.iter()
                    .map(|invader| invader.x)
                    .max()
                    .unwrap_or(0);
                if max_x == NUM_COLS - 1{
                    self.direction  = -1;
                    downwards = true;
                }
            }
            if downwards {
                let new_duration = max(self.move_timer.duration.as_millis() - 250, 250);
                self.move_timer = Timer::from_millis(new_duration as u64);
                for invader in self.army.iter_mut(){
                    invader.y += 1;
                }
            } else {
                for invader in self.army.iter_mut(){
                    invader.x  =((invader.x as i32) + self.direction) as usize;
                }
            }
            return true
        }
        false
    }

    pub fn all_killed(&self) -> bool {
        self.army.is_empty()
    }

    pub fn reached_bottom(&mut self) -> bool {
        self.stop = self.army.iter().map(|invader| invader.y)
        .max().unwrap_or(0) >= NUM_ROWS - 2;
        self.stop
    }

    pub fn kill_invader_at(&mut self, x:usize, y:usize) -> bool{
        if let Some(idx) = self.army.iter()
            .position(
                |invader| (invader.x == x) && (invader.y == y)
            )
        {
            self.army.remove(idx);
            true
        } else {
            false
        }
    }
    
}

impl Drawable for Invaders {
    fn draw(&self, frame: &mut Frame) {
        for invader in self.army.iter(){
            frame[invader.y][invader.x] = if self.move_timer.time_left.as_secs_f32() / 
            self.move_timer.duration.as_secs_f32() > 0.5 {
                "x"
            } else {
                "+"
            }
        }
    }
}