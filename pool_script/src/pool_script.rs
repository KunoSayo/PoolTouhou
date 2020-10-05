use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io::{BufRead, BufWriter, Error, ErrorKind, Read, stdin, Write};

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

    pub(crate) fn try_parse_bin(mut reader: Box<dyn BufRead>, debug: bool) -> Result<Self, Error> {
        let mut buf = [0; 16];
        let size = reader.read(&mut buf[0..5]).expect("read file failed");
        if size < 5 {
            return Err(Error::new(std::io::ErrorKind::InvalidData, "No const file data"));
        }
        let version = u32::from_be_bytes(buf[0..4].try_into().unwrap());
        let data_count = buf[4];
        let mut functions = HashMap::new();
        loop {
            let mut binary = Vec::with_capacity(128);
            let mut max_stack: i16 = -1;
            let function_name = read_str(&mut reader, &mut binary, false, debug);
            if function_name.is_empty() {
                break;
            }

            binary.clear();

            let mut loops = 0;
            loop {
                let read = reader.read(&mut buf[0..1]).unwrap();
                if read == 0 {
                    break;
                }
                binary.push(buf[0]);
                match buf[0] {
                    0 => {
                        if debug {
                            println!("ret")
                        }
                        if loops > 0 {
                            loops -= 1;
                        } else {
                            break;
                        }
                    }
                    1 => {
                        if debug {
                            println!("loop")
                        }
                        loops += 1;
                    }
                    3 | 5 | 10 | 20 => {
                        if debug {
                            println!("{}", match buf[0] {
                                3 => "push stack",
                                5 => "break",
                                10 => "move_up",
                                20 => "store",
                                _ => "Unknown",
                            });
                        }
                        if let Some(s) = read_f32(&mut binary, &mut reader, debug) {
                            max_stack = max_stack.max(s as i16);
                        }
                    }
                    4 => {
                        if debug {
                            println!("allocated");
                        }
                        //allocate needn't execute
                        binary.pop().unwrap();
                    }
                    11 => {
                        if debug {
                            println!("summon_e")
                        }
                        //name
                        read_str(&mut reader, &mut binary, true, debug);

                        //xy hp
                        read_f32(&mut binary, &mut reader, debug);
                        read_f32(&mut binary, &mut reader, debug);
                        read_f32(&mut binary, &mut reader, debug);
                        //collide & args
                        reader.read(&mut buf[0..1]).unwrap();
                        binary.push(buf[0]);

                        for _ in 0..GameData::try_from(buf[0]).unwrap().get_args_count() {
                            read_f32(&mut binary, &mut reader, debug);
                        }
                        //ai & args
                        let script_name = read_str(&mut reader, &mut binary, true, debug);
                        //todo: there is no more data
                        println!("We need know script {} data count:", script_name);
                        let ai_args_count = read_stdin_i32();
                        for _ in 0..ai_args_count {
                            read_f32(&mut binary, &mut reader, debug);
                        }
                    }
                    12 => {
                        if debug {
                            println!("summon_b")
                        }
                        //name
                        read_str(&mut reader, &mut binary, true, debug);

                        //xyz angle
                        read_f32(&mut binary, &mut reader, debug);
                        read_f32(&mut binary, &mut reader, debug);
                        read_f32(&mut binary, &mut reader, debug);
                        read_f32(&mut binary, &mut reader, debug);
                        //collide & args
                        reader.read(&mut buf[0..1]).unwrap();
                        binary.push(buf[0]);

                        for _ in 0..GameData::try_from(buf[0]).unwrap().get_args_count() {
                            read_f32(&mut binary, &mut reader, debug);
                        }
                        //ai & args
                        let script_name = read_str(&mut reader, &mut binary, true, debug);
                        println!("We need know script {} data count:", script_name);
                        //todo: there is no more data
                        let ai_args_count = read_stdin_i32();
                        for _ in 0..ai_args_count {
                            read_f32(&mut binary, &mut reader, debug);
                        }
                    }
                    38 | 39 => {
                        if debug {
                            println!("sin/cos command{}", buf[0]);
                        }
                        if let Some(s) = read_f32(&mut binary, &mut reader, debug) {
                            max_stack = max_stack.max(s as i16);
                        }
                        if let Some(s) = read_f32(&mut binary, &mut reader, debug) {
                            max_stack = max_stack.max(s as i16);
                        }
                    }
                    _ => {
                        if debug {
                            println!("byte command{}", buf[0]);
                        }
                    }
                }
            }
            functions.insert(function_name, binary);
        }

        let mut data_map = HashMap::default();
        for x in 0..data_count {
            data_map.insert(format!("data{}", x), x);
        }
        Ok(Self {
            version,
            data: data_map,
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
        if line[0].is_empty() || line[0].starts_with("//") {
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
                if let Ok(value) = context.parse_value(line[1]) {
                    binary.push(10);
                    value.flush(&mut binary)?;
                } else {
                    let exp = try_parse_expression(line[1].trim(), context)?;
                    exp.flush(&mut binary)?;
                    binary.push(10);
                    binary.push(4);
                }
            }
            "break" => {
                if let Ok(value) = context.parse_value(line[1]) {
                    binary.push(5);
                    value.flush(&mut binary)?;
                } else {
                    let exp = try_parse_expression(line[1].trim(), context)?;
                    exp.flush(&mut binary)?;
                    binary.push(5);
                    binary.push(4);
                }
            }
            "wait" => {
                if let Ok(value) = context.parse_value(line[1]) {
                    binary.push(6);
                    value.flush(&mut binary)?;
                } else {
                    let exp = try_parse_expression(line[1].trim(), context)?;
                    exp.flush(&mut binary)?;
                    binary.push(6);
                    binary.push(4);
                }
            }
            "loop" => {
                loops += 1;
                context.push_stack();
                binary.push(1);
            }
            "summon_e" => summon_e(line[1], &context, &mut binary)?,
            "summon_b" => summon_b(line[1], &context, &mut binary)?,
            "kill" => {
                binary.push(16);
            }
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
            "sin" | "cos" => {
                let arg = line[1].trim().splitn(2, ",").collect::<Vec<&str>>();
                if let (Ok(src), Ok(dst)) = (context.parse_value(arg[0].trim()), context.find_index(arg[1].trim())) {
                    binary.push(if line[0] == "sin" { 38 } else { 39 });
                    src.flush(&mut binary)?;
                    dst.flush(&mut binary)?;
                } else {
                    return Err(Error::new(ErrorKind::InvalidData, "[parse function]unknown reason: ".to_owned() + &*raw_line));
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
    let collide_rule = GameData::try_from(args[4])?;
    let read = collide_rule.get_args(&args[5..], context, binary)?;
    let index = 5 + read;
    args[index].flush(binary)?;
    for x in args[index + 1..].iter() {
        if let Ok(value) = context.parse_value(x) {
            value.flush(binary)?;
        } else {
            x.flush(binary)?;
        }
    }
    binary.push(9);
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
    binary.push(9);
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

fn read_f32(binary: &mut Vec<u8>, reader: &mut Box<dyn BufRead>, debug: bool) -> Option<u8> {
    let mut buf = [0; 4];
    reader.read(&mut buf[0..1]).unwrap();
    binary.push(buf[0]);
    match buf[0] {
        0 => {
            if debug {
                println!("point const ({})", buf[0])
            }
            reader.read(&mut buf[0..4]).unwrap();
            binary.push(buf[0]);
            binary.push(buf[1]);
            binary.push(buf[2]);
            binary.push(buf[3]);
        }
        3 => {
            if debug {
                println!("point stack value ({})", buf[0])
            }
            reader.read(&mut buf[0..1]).unwrap();
            binary.push(buf[0]);
            return Some(buf[0]);
        }
        4 => {
            if debug {
                println!("point calc value ({})", buf[0])
            }
        }
        9 => {
            if debug {
                println!("no data");
            }
        }
        _ => {
            if debug {
                println!("point value ({})", buf[0])
            }
            reader.read(&mut buf[0..1]).unwrap();
            binary.push(buf[0]);
        }
    }
    None
}

fn read_str(reader: &mut Box<dyn BufRead>, binary: &mut Vec<u8>, write: bool, debug: bool) -> String {
    let mut buf = [0; 32];
    if reader.read(&mut buf[0..2]).unwrap() == 0 {
        return "".to_string();
    }
    if write {
        binary.push(buf[0]);
        binary.push(buf[1]);
    }
    let str_len = u16::from_be_bytes(buf[0..2].try_into().unwrap()) as usize;
    let mut vec = Vec::with_capacity(str_len);
    let mut len = 0;
    while len < str_len {
        let read = reader.read(&mut buf[0..(str_len - len).min(32)]).unwrap();
        for x in &buf[0..read] {
            vec.push(*x);
        }
        len += read;
    }

    if write {
        for x in &vec {
            binary.push(*x);
        }
    }

    let str = String::from_utf8(vec).expect("parse utf8 str failed");
    if debug {
        println!("str: {}", str);
    }
    str
}

fn read_stdin_i32() -> i32 {
    let mut line = String::default();
    stdin().read_line(&mut line).expect("read failed");
    line.trim().parse().unwrap()
}