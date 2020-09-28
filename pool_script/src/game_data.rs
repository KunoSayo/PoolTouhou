use std::convert::TryFrom;
use std::io::{Error, ErrorKind};

use crate::context::Context;
use crate::pool_script::Compile;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum GameData {
    PosX = 0,
    PosY = 1,
    PosZ = 2,
    PlayerX = 3,
    PlayerY = 4,
    PlayerZ = 5,
    CircleCollide = 10,
}

impl GameData {
    pub fn get_args(&self, args: &[&str], context: &Context, binary: &mut Vec<u8>) -> Result<usize, Error> {
        match self {
            Self::CircleCollide => {
                if args.len() < 1 {
                    eprintln!("There is no more args {:?}", self);
                    Err(Error::new(ErrorKind::InvalidData, "[parse game data]args is not enough"))
                } else {
                    binary.push(*self as u8);
                    context.parse_value(args[0])?.flush(binary)?;
                    Ok(1)
                }
            }
            _ => {
                eprintln!("There is no args about {:?}", self);
                Err(Error::new(ErrorKind::InvalidData, "[parse game data]get args failed"))
            }
        }
    }

    pub fn get_args_count(&self) -> usize {
        match self {
            GameData::CircleCollide => 1,
            _ => panic!("no such arg")
        }
    }
}

impl TryFrom<u8> for GameData {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            10 => Ok(GameData::CircleCollide),
            _ => {
                eprintln!("There is unknown binary value {}", value);
                Err(Error::new(ErrorKind::InvalidData, "[parse game data]no such game value"))
            }
        }
    }
}

impl TryFrom<&str> for GameData {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pos_x" => Ok(GameData::PosX),
            "pos_y" => Ok(GameData::PosY),
            "pos_z" => Ok(GameData::PosZ),
            "player_x" => Ok(GameData::PlayerX),
            "player_y" => Ok(GameData::PlayerY),
            "player_z" => Ok(GameData::PlayerZ),
            "circle" => Ok(GameData::CircleCollide),
            _ => Err(Error::new(ErrorKind::InvalidData, "[parse game data]expected game data but found : ".to_owned() + value))
        }
    }
}