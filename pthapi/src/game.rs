//! The center is (0, 0)
//!
//! Left top is (-800, 450)
//!
//! Right bottom is (800, -450)
//!
//! The right angle is zero and up is 90
//!

use std::convert::TryFrom;
use std::io::{Error, ErrorKind};

use crate::{PLAYER_Z, TexHandle};

pub const GAME_MAX_X: f32 = 800.0;
pub const GAME_MIN_X: f32 = -800.0;
pub const GAME_MAX_Y: f32 = 450.0;
pub const GAME_MIN_Y: f32 = -450.0;

#[repr(C)]
pub struct Player {
    pub pos: GamePos,
    pub move_speed: f32,
    pub walk_speed: f32,
    pub radius: f32,
    ///
    /// zero is no death
    /// &gt; 0 is dying
    /// &lt; 0 is died and ticks for death
    ///
    pub death: isize,
    pub tex: usize,
    pub shoot_cooldown: u8,
    pub walking: bool,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct GamePos {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Into<GamePos> for (f32, f32, f32) {
    fn into(self) -> GamePos {
        GamePos {
            x: self.0,
            y: self.1,
            z: self.2,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum CollideType {
    Circle {
        radius: f32,
        radius_2: f32,
    }
}

impl CollideType {
    pub extern "C" fn is_collide_with_point(self, me: &GamePos, other: &GamePos) -> bool {
        match self {
            Self::Circle {
                radius: _,
                radius_2: r_2
            } => {
                let x_distance = me.x - other.x;
                let y_distance = me.y - other.y;
                x_distance * x_distance + y_distance * y_distance < r_2
            }
        }
    }

    pub extern "C" fn is_collide_with_point_up(self, me: &GamePos, point: &GamePos, up: f32) -> bool {
        match self {
            Self::Circle {
                radius: r,
                radius_2: r_2
            } => {
                let left = point.x - r;
                let right = point.x + r;
                let top = up + point.y;
                if me.x > left && me.x < right && me.y < top && me.y > point.y {
                    true
                } else {
                    let x_distance = me.x - point.x;
                    let y_distance = me.y - point.y;
                    if x_distance * x_distance + y_distance * y_distance < r_2 {
                        true
                    } else {
                        let x_distance = me.x - point.x;
                        let y_distance = me.y - top;
                        x_distance * x_distance + y_distance * y_distance < r_2
                    }
                }
            }
        }
    }

    pub extern "C" fn is_collide_with(self, me: &GamePos, other_collide: &CollideType, other: &GamePos) -> bool {
        match self {
            Self::Circle {
                radius: _,
                radius_2: r_2
            } => {
                match other_collide {
                    Self::Circle {
                        radius: _,
                        radius_2: o_r_2
                    } => {
                        let center_x_distance = me.x - other.x;
                        let center_y_distance = me.y - other.y;
                        let center_distance = center_x_distance * center_x_distance + center_y_distance * center_y_distance;
                        center_distance < r_2 + o_r_2
                    }
                }
            }
        }
    }
}

impl TryFrom<(u8, Vec<f32>)> for CollideType {
    type Error = Error;

    fn try_from((value, args): (u8, Vec<f32>)) -> Result<Self, Self::Error> {
        match value {
            10 => Ok(CollideType::Circle { radius: args[0], radius_2: args[0] * args[0] }),
            _ => Err(Error::new(ErrorKind::InvalidData, "No such value for CollideType: ".to_owned() + &*value.to_string()))
        }
    }
}

impl CollideType {
    //noinspection RsSelfConvention
    pub extern "C" fn get_arg_count(byte: u8) -> usize {
        match byte {
            10 => 1,
            _ => panic!("Not collide byte: {}", byte)
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new(10.0, 4.5)
    }
}

impl Player {
    pub extern "C" fn new(move_speed: f32, walk_speed: f32) -> Self {
        Self {
            pos: (0.0, -400.0, PLAYER_Z).into(),
            move_speed,
            walk_speed,
            walking: false,
            radius: 5.0,
            shoot_cooldown: 0,
            death: 0,
            tex: 0,
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct PlayerBullet {
    pub pos: GamePos,
    pub tex: TexHandle,
    pub damage: f32,
}


#[repr(C)]
#[derive(Default)]
pub struct Rotation {
    pub facing_x: f32,
    pub facing_y: f32,
    pub angle: f32,
}

impl Rotation {
    pub extern "C" fn add_angle(&mut self, a: f32) {
        if a != 0.0 {
            self.angle += a;
            let (sin, cos) = (self.angle * std::f32::consts::PI / 180.0).sin_cos();
            self.facing_x = cos;
            self.facing_y = sin;
        }
    }

    pub extern "C" fn new(a: f32) -> Self {
        let (sin, cos) = (a * std::f32::consts::PI / 180.0).sin_cos();
        Self {
            facing_x: cos,
            facing_y: sin,
            angle: a,
        }
    }
}

#[repr(C)]
pub struct SimpleEnemyBullet {
    pub pos: GamePos,
    pub tex: TexHandle,
    pub collide: CollideType,
    pub speed: f32,
    pub rotation: Rotation,
    pub a: f32,
    pub a_delta: f32,
    pub w: f32,
    pub w_delta: f32,
}

impl SimpleEnemyBullet {
    pub extern "C" fn new(pos: GamePos, tex: TexHandle, collide: CollideType, speed: f32, angle: f32) -> Self {
        let (sin, cos) = (angle * std::f32::consts::PI / 180.0).sin_cos();
        Self {
            pos,
            tex,
            collide,
            speed,
            rotation: Rotation {
                facing_x: cos,
                facing_y: sin,
                angle,

            },
            a: 0.0,
            a_delta: 0.0,
            w: 0.0,
            w_delta: 0.0,
        }
    }

    pub extern "C" fn tick(&mut self) {
        self.pos.x += self.speed * self.rotation.facing_x;
        self.pos.y += self.speed * self.rotation.facing_y;
        self.speed += self.a;
        self.a += self.a_delta;
        self.rotation.add_angle(self.w);
        self.w += self.w_delta;
    }
}
