use std::collections::{HashMap, LinkedList};
use std::convert::TryFrom;
use std::io::{Error, ErrorKind};
use std::str::FromStr;

use crate::expression::ExpressionElement;
use crate::game_data::GameData;

pub struct Context<'a> {
    heap: &'a HashMap<String, u8>,
    stack: LinkedList<Vec<String>>,
    stack_count: u8,
}

impl<'a> Context<'a> {
    pub fn new(heap: &'a HashMap<String, u8>) -> Self {
        let mut list = LinkedList::new();
        list.push_back(Vec::new());
        Self {
            heap,
            stack: list,
            stack_count: 0,
        }
    }

    pub fn parse_value(&self, string: &str) -> Result<ExpressionElement, Error> {
        if let Ok(value) = f32::from_str(string) {
            return Ok(ExpressionElement::CONST(value));
        }
        self.find_index(string)
    }

    pub fn find_index(&self, name: &str) -> Result<ExpressionElement, Error> {
        if let Ok(value) = GameData::try_from(name) {
            return Ok(ExpressionElement::GAME(value as u8));
        } else if let Some(value) = self.heap.get(name) {
            return Ok(ExpressionElement::DATA(*value));
        } else if self.stack_count > 0 {
            let mut count = self.stack_count;
            for ss in self.stack.iter().rev() {
                for s in ss.iter().rev() {
                    if s == name {
                        return Ok(ExpressionElement::STACK(count - 1));
                    }
                    count -= 1;
                }
            }
        }

        return Err(Error::new(ErrorKind::InvalidData, "[find index]Unknown var name: ".to_owned() + name));
    }

    pub fn push_stack(&mut self) {
        self.stack.push_back(Vec::new());
    }

    pub fn pop_stack(&mut self) {
        let vec = self.stack.pop_back().unwrap();
        self.stack_count -= vec.len() as u8;
    }

    pub fn push_name(&mut self, name: &str) {
        let vec = self.stack.back_mut().unwrap();
        vec.push(name.to_string());
        self.stack_count += 1;
    }
}
