use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufRead, BufWriter, Error, ErrorKind, Write};

use crate::context::Context;
use crate::expression::{ExpressionElement, try_parse_expression};
use crate::game_data::GameData;

pub trait Compile {
    fn flush(&self, binary: &mut Vec<u8>) -> Result<(), Error>;
}

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

    pub fn save(&self, writer: &mut BufWriter<File>) -> Result<(), Error> {
        writer.write(&self.version.to_be_bytes())?;
        writer.write(&[self.data.len() as u8])?;
        for x in self.functions.values() {
            writer.write(x)?;
        }
        writer.flush()?;
        Ok(())
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

    let mut loops = 0;
    loop {
        let mut raw_line = String::new();
        let size = reader.read_line(&mut raw_line).unwrap();
        if size == 0 {
            break;
        }
        let line: Vec<&str> = raw_line.trim().splitn(2, " ").collect();
        if line[0].starts_with("//") {
            continue;
        }
        match line[0] {
            "end" => {
                binary.push(0);
                if loops > 0 {
                    loops -= 1;
                    context.pop_stack();
                } else {
                    break;
                }
            }
            "move_up" => {
                binary.push(10);
                let value = context.parse_value(line[1])?;
                value.flush(&mut binary)?;
            }
            "break" => {
                binary.push(5);
                let value = context.parse_value(line[1])?;
                value.flush(&mut binary)?;
            }
            "loop" => {
                loops += 1;
                context.push_stack();
                binary.push(1);
            }
            "summon_e" => summon_e(line[1], &context, &mut binary)?,
            "summon_b" => summon_b(line[1], &context, &mut binary)?,
            "let" => {
                let expression: Vec<&str> = line[1].split("=").collect();
                let name = expression[0].trim();
                if let Ok(index) = context.find_index(name) {
                    if expression.len() < 2 {
                        return Err(Error::new(ErrorKind::InvalidData, "[parse function]where is the expression? : ".to_owned() + &*raw_line));
                    }
                    let exp = try_parse_expression(expression[1].trim(), context)?;
                    exp.flush(&mut binary)?;
                    binary.push(20);
                    index.flush(&mut binary)?;
                } else {
                    binary.push(4);
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
            }
            _ => {
                return Err(Error::new(ErrorKind::InvalidData, "[parse function]unknown command: ".to_owned() + &*raw_line));
            }
        }
    }
    if loops > 0 {
        Err(Error::new(ErrorKind::InvalidData, "[loops not end!] ".to_owned() + &*loops.to_string()))
    } else {
        Ok(binary)
    }
}

fn summon_e(raw_args: &str, context: &Context, binary: &mut Vec<u8>) -> Result<(), Error> {
    binary.push(11);
    let args: Vec<&str> = raw_args.split(" ").collect();
    if args.len() < 5 {
        return Err(Error::new(ErrorKind::InvalidData, "[parse function]command args is not good (require 5..): ".to_owned() + raw_args));
    }

    // name x y hp ai_name
    args[0].flush(binary)?;
    context.parse_value(args[1])?.flush(binary)?;
    context.parse_value(args[2])?.flush(binary)?;
    context.parse_value(args[3])?.flush(binary)?;
    args[4].flush(binary)?;
    for x in args[5..].iter() {
        if let Ok(value) = context.parse_value(x) {
            value.flush(binary)?;
        } else {
            x.flush(binary)?;
        }
    }
    Ok(())
}

fn summon_b(raw_args: &str, context: &Context, binary: &mut Vec<u8>) -> Result<(), Error> {
    binary.push(12);
    let args: Vec<&str> = raw_args.split(" ").collect();
    if args.len() < 7 {
        return Err(Error::new(ErrorKind::InvalidData, "[parse function]command args is not good (require 7..): ".to_owned() + raw_args));
    }

    args[0].flush(binary)?;
    context.parse_value(args[1])?.flush(binary)?;
    context.parse_value(args[2])?.flush(binary)?;
    context.parse_value(args[3])?.flush(binary)?;
    context.parse_value(args[4])?.flush(binary)?;

    let collide_rule = GameData::try_from(args[5])?;
    let read = collide_rule.get_args(&args[6..], context, binary)?;
    let index = 6 + read;
    args[index].flush(binary)?;
    for x in args[index + 1..].iter() {
        if let Ok(value) = context.parse_value(x) {
            value.flush(binary)?;
        } else {
            x.flush(binary)?;
        }
    }
    Ok(())
}

impl Compile for &str {
    fn flush(&self, binary: &mut Vec<u8>) -> Result<(), Error> {
        let bytes = self.bytes();
        let len = bytes.len() as u16;
        for byte in len.to_be_bytes().iter() {
            binary.push(*byte);
        }
        for byte in bytes.into_iter() {
            binary.push(byte);
        }
        Ok(())
    }
}