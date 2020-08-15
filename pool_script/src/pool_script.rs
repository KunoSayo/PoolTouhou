use std::collections::HashMap;
use std::io::{BufRead, Error, ErrorKind};

use crate::context::Context;
use crate::expression::{Compile, ExpressionElement, try_parse_expression};

pub struct PoolScript {
    version: u32,
    data: HashMap<String, u8>,
    functions: HashMap<String, Vec<u8>>,
}

impl PoolScript {
    pub(crate) fn try_parse(mut reader: Box<dyn BufRead>) -> Result<Self, Error> {
        let mut data = HashMap::new();
        let mut functions = HashMap::new();
        loop {
            let mut line = String::new();
            let size = reader.read_line(&mut line)
                .unwrap();
            let line = line.trim();
            if line.starts_with("data") {
                parse_data(&mut reader, &mut data)?;
            }
            let mut context = Context::new(&data);
            if line.starts_with("function") {
                let function_with_name: Vec<&str> = line.split(" ").collect();
                if function_with_name.len() < 2 {
                    return Err(Error::new(ErrorKind::InvalidData, "[parse function]where is the function name"));
                }
                let result = parse_function(function_with_name[1], &mut reader, &mut context)?;
                functions.insert(function_with_name[1].to_string(), result);
            }
            if size == 0 {
                break;
            }
        }
        Ok(Self {
            version: 0,
            data,
            functions,
        })
    }
}

fn parse_data(reader: &mut Box<dyn BufRead>, data: &mut HashMap<String, u8>) -> Result<(), Error> {
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let line = line.trim();
        if line == "end" {
            return Ok(());
        }
        let type_with_name: Vec<&str> = line.split(" ").collect();
        if type_with_name.len() != 2 {
            return Err(Error::new(ErrorKind::InvalidData, "[parse data]excepted 2 but found ".to_owned() + &type_with_name.len().to_string()));
        }
        if type_with_name[0] != "f32" {
            return Err(Error::new(ErrorKind::InvalidData, "[parse data]only f32 is supported"));
        }
        data.insert(type_with_name[1].to_string(), data.len() as u8);
    }
}

fn parse_function(name: &str, reader: &mut Box<dyn BufRead>, context: &mut Context) -> Result<Vec<u8>, Error> {
    let name_bytes = name.bytes();
    let mut binary: Vec<u8> = Vec::with_capacity(name_bytes.len() + 3);
    for byte in (name_bytes.len() as u16).to_be_bytes().iter() {
        binary.push(*byte);
    }
    for byte in name_bytes {
        binary.push(byte);
    }

    loop {
        let mut raw_line = String::new();
        let size = reader.read_line(&mut raw_line).unwrap();
        if size == 0 {
            break;
        }
        let line: Vec<&str> = raw_line.trim().splitn(2, " ").collect();
        match line[0] {
            "end" => {
                binary.push(0);
                break;
            }
            "move_up" => {
                binary.push(10);
                let value = context.parse_value(line[1])?;
                value.flush(&mut binary)?;
            }
            "summon_e" => {
                binary.push(11);
            }
            "summon_b" => {
                binary.push(12);
            }
            "let" => {
                let expression: Vec<&str> = line[1].split("=").collect();
                let name = expression[0].trim();
                context.push_name(name);
                if expression.len() > 1 {
                    let exp = try_parse_expression(expression[1].trim(), context)?;
                    exp.flush(&mut binary)?;
                    binary.push(20);
                    binary.push(3);
                    if let ExpressionElement::STACK(idx) = context.find_index(name)? {
                        binary.push(idx);
                    } else {
                        return Err(Error::new(ErrorKind::InvalidData, "[parse function]unknown reason: ".to_owned() + &*raw_line));
                    }
                }
            }
            _ => {
                return Err(Error::new(ErrorKind::InvalidData, "[parse function]unknown command: ".to_owned() + &*raw_line));
            }
        }
    }
    Ok(binary)
}

fn summon_e(args: &str, binary: &mut Vec<u8>) {}

fn summon_b(args: &str, binary: &mut Vec<u8>) {}