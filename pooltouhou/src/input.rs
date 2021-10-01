use std::collections::HashSet;
use std::mem::swap;

use winit::event::VirtualKeyCode;

use crate::input;

#[derive(Debug, Default)]
pub struct RawInputData {
    pub x: f32,
    pub y: f32,
    pub pressing: Box<HashSet<VirtualKeyCode>>,
}


#[derive(Debug, Default)]
pub struct GameInputData {
    pub shoot: u32,
    pub slow: u32,
    pub bomb: u32,
    pub sp: u32,
    pub up: u32,
    pub down: u32,
    pub left: u32,
    pub right: u32,
    pub direction: (i32, i32),
    pub enter: u32,
    pub esc: u32,
}

#[derive(Default)]
pub struct BakedInputs {
    pub cur_temp_input: input::RawInputData,

    pub last_frame_input: input::RawInputData,
    pub cur_frame_input: input::RawInputData,
    pub cur_frame_game_input: input::GameInputData,

    /// only swap in states.game tick
    pub cur_temp_game_input: input::RawInputData,
    /// only swap in states.game tick
    pub last_temp_game_input: input::RawInputData,
    /// only swap in states.game tick
    pub cur_game_input: input::GameInputData,
}


impl BakedInputs {
    pub fn process(&mut self, pressed: &HashSet<VirtualKeyCode>, released: &HashSet<VirtualKeyCode>) {
        for key in released.iter() {
            if self.cur_frame_input.pressing.contains(key) {
                self.cur_temp_input.pressing.remove(key);
            }
        }
        for key in pressed.iter() {
            self.cur_temp_input.pressing.insert(*key);
        }

        for key in released.iter() {
            if self.last_temp_game_input.pressing.contains(key) {
                self.cur_temp_game_input.pressing.remove(key);
            }
        }

        for key in pressed.iter() {
            self.cur_temp_game_input.pressing.insert(*key);
        }
    }
    /// save current input to last
    /// make current temp input to current frame input
    pub fn swap_frame(&mut self) {
        //save current to last
        swap(&mut self.cur_frame_input, &mut self.last_frame_input);
        //clone for not lose temp info
        self.cur_frame_input = self.cur_temp_input.clone();
        self.cur_frame_game_input.tick_mut(&self.cur_frame_input);
    }

    /// save current states.game tick input to last
    pub fn tick(&mut self) {
        self.last_temp_game_input = self.cur_temp_game_input.clone();
        self.cur_game_input.tick_mut(&self.cur_temp_game_input);
    }

    pub fn is_pressed(&self, keys: &[VirtualKeyCode]) -> bool {
        keys.iter().any(|k| !self.last_frame_input.pressing.contains(k))
            && keys.iter().all(|k| self.cur_frame_input.pressing.contains(k))
    }
}

fn get_direction(up: u32, down: u32, left: u32, right: u32) -> (i32, i32) {
    let x = if left == right {
        0
    } else if left == 0 {
        1
    } else if right == 0 || left < right {
        -1
    } else {
        1
    };
    let y = if up == down {
        0
    } else if up == 0 {
        -1
    } else if down == 0 || up < down {
        1
    } else {
        -1
    };
    (x, y)
}

impl From<&RawInputData> for GameInputData {
    fn from(r: &RawInputData) -> Self {
        let up = r.pressing.contains(&VirtualKeyCode::Up) as u32;
        let down = r.pressing.contains(&VirtualKeyCode::Down) as u32;
        let left = r.pressing.contains(&VirtualKeyCode::Left) as u32;
        let right = r.pressing.contains(&VirtualKeyCode::Right) as u32;
        let direction = get_direction(up, down, left, right);
        Self {
            shoot: r.pressing.contains(&VirtualKeyCode::Z) as u32,
            slow: r.pressing.contains(&VirtualKeyCode::LShift) as u32,
            bomb: r.pressing.contains(&VirtualKeyCode::X) as u32,
            sp: r.pressing.contains(&VirtualKeyCode::C) as u32,
            up,
            down,
            left,
            right,
            direction,
            enter: (r.pressing.contains(&VirtualKeyCode::Return) || r.pressing.contains(&VirtualKeyCode::NumpadEnter)) as u32,
            esc: r.pressing.contains(&VirtualKeyCode::Escape) as u32,
        }
    }
}

macro_rules! inc_or_zero {
    ($e: expr, $b: expr) => {
        if $b {
            $e += 1;
        } else {
            $e = 0;
        }
    };
}

impl GameInputData {
    pub fn tick_mut(&mut self, r: &RawInputData) {
        inc_or_zero!(self.shoot, r.pressing.contains(&VirtualKeyCode::Z));
        inc_or_zero!(self.slow, r.pressing.contains(&VirtualKeyCode::LShift));
        inc_or_zero!(self.bomb, r.pressing.contains(&VirtualKeyCode::X));
        inc_or_zero!(self.sp, r.pressing.contains(&VirtualKeyCode::C));
        inc_or_zero!(self.up, r.pressing.contains(&VirtualKeyCode::Up));
        inc_or_zero!(self.down, r.pressing.contains(&VirtualKeyCode::Down));
        inc_or_zero!(self.left, r.pressing.contains(&VirtualKeyCode::Left));
        inc_or_zero!(self.right, r.pressing.contains(&VirtualKeyCode::Right));
        inc_or_zero!(self.enter, r.pressing.contains(&VirtualKeyCode::Return) || r.pressing.contains(&VirtualKeyCode::NumpadEnter));
        inc_or_zero!(self.esc, r.pressing.contains(&VirtualKeyCode::Escape));
        self.direction = get_direction(self.up, self.down, self.left, self.right);
    }

    pub fn clear(&mut self) {
        *self = Default::default();
    }
}

impl RawInputData {
    pub fn empty() -> Self {
        Self::default()
    }
}

impl GameInputData {
    pub fn get_move(&self, base_speed: f32) -> (f32, f32) {
        let x = if self.left == self.right {
            0
        } else if self.left == 0 {
            1
        } else if self.right == 0 || self.left < self.right {
            -1
        } else {
            1
        };
        let y = if self.up == self.down {
            0
        } else if self.up == 0 {
            -1
        } else if self.down == 0 || self.up < self.down {
            1
        } else {
            -1
        };
        if x == 0 || y == 0 {
            (x as f32 * base_speed, y as f32 * base_speed)
        } else {
            (x as f32 * std::f32::consts::FRAC_1_SQRT_2 * base_speed, y as f32 * base_speed * std::f32::consts::FRAC_1_SQRT_2)
        }
    }
}

impl Clone for RawInputData {
    fn clone(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            pressing: self.pressing.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.x = source.x;
        self.y = source.y;
        self.pressing = source.pressing.clone();
    }
}

mod test {
    #[test]
    fn test_direction() {
        use crate::input::get_direction;
        assert_eq!(get_direction(0, 0, 0, 5), (1, 0));
        assert_eq!(get_direction(3, 3, 10, 5), (1, 0));
        assert_eq!(get_direction(3, 3, 4, 4), (0, 0));
        assert_eq!(get_direction(7, 3, 4, 4), (0, -1));
        assert_eq!(get_direction(1, 3, 4, 4), (0, 1));
        assert_eq!(get_direction(1, 3, 4, 4), (0, 1));
        assert_eq!(get_direction(1, 3, 4, 5), (-1, 1));
        assert_eq!(get_direction(4, 3, 4, 5), (-1, -1));
        assert_eq!(get_direction(4, 3, 6, 5), (1, -1));
        //zero region
        assert_eq!(get_direction(0, 0, 0, 0), (0, 0));
        assert_eq!(get_direction(1, 0, 0, 0), (0, 1));
        assert_eq!(get_direction(0, 1, 0, 0), (0, -1));
        assert_eq!(get_direction(0, 0, 1, 0), (-1, 0));
        assert_eq!(get_direction(0, 0, 0, 1), (1, 0));
        //end zero region
    }
}