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

pub struct Player {
    pub pos: PosType,
    pub move_speed: f32,
    pub walk_speed: f32,
    pub walking: bool,
    pub radius: f32,
    pub shoot_cooldown: u8,
    ///
    /// zero is no death
    /// &gt; 0 is dying
    /// &lt; 0 is died and ticks for death
    ///
    pub death: isize,
    pub tex: usize,
}

pub type PosType = (f32, f32, f32);

#[derive(Debug, Clone, Copy)]
pub enum CollideType {
    Circle {
        radius: f32,
        radius_2: f32,
    }
}

impl CollideType {
    pub fn is_collide_with_point(self, me: &PosType, other: &PosType) -> bool {
        match self {
            Self::Circle {
                radius: _,
                radius_2: r_2
            } => {
                let x_distance = me.0 - other.0;
                let y_distance = me.1 - other.1;
                x_distance * x_distance + y_distance * y_distance < r_2
            }
        }
    }

    pub fn is_collide_with_point_up(self, me: &PosType, point: &PosType, up: f32) -> bool {
        match self {
            Self::Circle {
                radius: r,
                radius_2: r_2
            } => {
                let left = point.0 - r;
                let right = point.0 + r;
                let top = up + point.1;
                if me.0 > left && me.0 < right && me.1 < top && me.1 > point.1 {
                    true
                } else {
                    let x_distance = me.0 - point.0;
                    let y_distance = me.1 - point.1;
                    if x_distance * x_distance + y_distance * y_distance < r_2 {
                        true
                    } else {
                        let x_distance = me.0 - point.0;
                        let y_distance = me.1 - top;
                        x_distance * x_distance + y_distance * y_distance < r_2
                    }
                }
            }
        }
    }

    pub fn is_collide_with(self, me: &PosType, other_collide: &CollideType, other: &PosType) -> bool {
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
                        let center_x_distance = me.0 - other.0;
                        let center_y_distance = me.1 - other.1;
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
    pub fn get_arg_count(byte: u8) -> usize {
        match byte {
            10 => 1,
            _ => panic!("Not collide byte: {}", byte)
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new(5.0)
    }
}

impl Player {
    pub fn new(speed: f32) -> Self {
        Self {
            pos: (0.0, -400.0, PLAYER_Z),
            move_speed: speed,
            walk_speed: speed * 0.6,
            walking: false,
            radius: 5.0,
            shoot_cooldown: 0,
            death: 0,
            tex: 0,
        }
    }
}

#[derive(Default)]
pub struct PlayerBullet {
    pub pos: PosType,
    pub tex: TexHandle,
    pub damage: f32,
}

#[derive(Default)]
pub struct Rotation {
    pub facing: (f32, f32),
    pub angle: f32,
}

impl Rotation {
    #[inline]
    pub fn add_angle(&mut self, a: f32) {
        if a != 0.0 {
            self.angle += a;
            let (sin, cos) = (self.angle * std::f32::consts::PI / 180.0).sin_cos();
            self.facing = (cos, sin);
        }
    }

    pub fn new(a: f32) -> Self {
        let a = a * std::f32::consts::PI / 180.0;
        let (sin, cos) = a.sin_cos();
        Self {
            facing: (cos, sin),
            angle: a,
        }
    }
}

pub struct SimpleEnemyBullet {
    pub pos: PosType,
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
    pub fn new(pos: PosType, tex: TexHandle, collide: CollideType, speed: f32, angle: f32) -> Self {
        let (sin, cos) = (angle * std::f32::consts::PI / 180.0).sin_cos();
        Self {
            pos,
            tex,
            collide,
            speed,
            rotation: Rotation {
                facing: (cos, sin),
                angle,
            },
            a: 0.0,
            a_delta: 0.0,
            w: 0.0,
            w_delta: 0.0,
        }
    }

    pub fn tick(&mut self) {
        self.pos.0 += self.speed * self.rotation.facing.0;
        self.pos.1 += self.speed * self.rotation.facing.1;
        self.speed += self.a;
        self.a += self.a_delta;
        self.rotation.add_angle(self.w);
        self.w += self.w_delta;
    }
}
