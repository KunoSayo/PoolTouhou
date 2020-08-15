use std::collections::{HashMap, LinkedList};
use std::io::{Error, ErrorKind};
use std::str::FromStr;

use crate::expression::ExpressionElement;

pub struct Context<'a> {
    heap: &'a HashMap<String, u8>,
    stack: LinkedList<Vec<String>>,
    stack_index: u8,
}

impl<'a> Context<'a> {
    pub fn new(heap: &'a HashMap<String, u8>) -> Self {
        let mut list = LinkedList::new();
        list.push_back(Vec::new());
        Self {
            heap,
            stack: list,
            stack_index: 0,
        }
    }

    pub fn parse_value(&self, string: &str) -> Result<ExpressionElement, Error> {
        if let Ok(value) = f32::from_str(string) {
            return Ok(ExpressionElement::CONST(value));
        }
        self.find_index(string)
    }

    pub fn find_index(&self, name: &str) -> Result<ExpressionElement, Error> {
        if let Some(value) = self.heap.get(name) {
            return Ok(ExpressionElement::DATA(*value));
        } else if self.stack_index > 0 {
            let mut index = self.stack_index - 1;
            for ss in self.stack.iter().rev() {
                for s in ss.iter().rev() {
                    if s == name {
                        return Ok(ExpressionElement::STACK(index));
                    }
                    index -= 1;
                }
            }
        }
        if name.starts_with("pos") {}

        return Err(Error::new(ErrorKind::InvalidData, "[find index]Unknown var name: ".to_owned() + name));
    }

    pub fn push_stack(&mut self) {
        self.stack.push_back(Vec::new());
    }

    pub fn pop_stack(&mut self) {
        let vec = self.stack.pop_back().unwrap();
        self.stack_index -= vec.len() as u8;
    }

    pub fn push_name(&mut self, name: &str) {
        let vec = self.stack.back_mut().unwrap();
        vec.push(name.to_string());
        self.stack_index += 1;
    }
}
