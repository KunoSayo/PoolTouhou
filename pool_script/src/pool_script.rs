use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Read, Write};

use crate::context::Context;
use crate::expression::{ExpressionElement, try_parse_expression};
use crate::game_data::GameData;

pub trait Compile {
    fn flush(&self, binary: &mut Vec<u8>) -> Result<(), Error>;
}

#[derive(Debug, Copy, Clone)]
pub enum Loop {
    Start(usize),
    End(usize),
}

#[derive(Debug, Clone)]
pub struct FunctionDesc {
    pub code: Vec<u8>,
    pub loops: Vec<Loop>,
    pub max_stack: u16,
}

#[derive(Clone, Debug)]
pub struct PoolScriptBin {
    pub version: u32,
    pub data: HashMap<String, u8>,
    pub functions: HashMap<String, FunctionDesc>,
}

pub struct Parser {
    reader: BufReader<File>,
    line: usize,
}

impl Parser {
    pub fn new(file: std::fs::File) -> Self {
        let reader = std::io::BufReader::new(file);
        Self {
            reader,
            line: 0,
        }
    }

    pub fn read_line(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.line += 1;
        self.reader.read_line(buf).map_err(|e| {
            eprintln!("Read line {} failed", self.line);
            e
        })
    }

    pub fn try_parse(&mut self) -> Result<PoolScriptBin, Error> {
        PoolScriptBin::try_parse(self).map_err(|e| {
            eprintln!("In line {}", self.line);
            e
        })
    }
}

impl PoolScriptBin {
    fn try_parse(mut parser: &mut Parser) -> Result<Self, Error> {
        let mut data = HashMap::new();
        let mut functions = HashMap::new();
        loop {
            let mut line = String::new();
            let size = parser.read_line(&mut line)
                .unwrap();
            let line = line.trim();
            if line.starts_with("data") {
                parse_data(&mut parser, &mut data)?;
            }
            let mut context = Context::new(&data);
            if line.starts_with("function") {
                let function_with_name: Vec<&str> = line.split(" ").collect();
                if function_with_name.len() < 2 {
                    return Err(Error::new(ErrorKind::InvalidData, "[parse function]where is the function name"));
                }
                let result = parse_function(function_with_name[1], &mut parser, &mut context)?;
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

    pub fn try_parse_bin(mut reader: BufReader<File>) -> Result<Self, Error> {
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
            let mut max_stack = 0u16;
            let mut loop_vec = Vec::new();
            let function_name = read_str(&mut reader, &mut binary, false);
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
                        log::debug!("return");
                        if loops > 0 {
                            loops -= 1;
                            loop_vec.push(Loop::End(binary.len()));
                        } else {
                            break;
                        }
                    }
                    1 => {
                        log::debug!("loop");
                        loop_vec.push(Loop::Start(binary.len()));
                        loops += 1;
                    }
                    3 | 5 | 10 | 20 => {
                        log::debug!("{}", match buf[0] {
                                3 => "push stack",
                                5 => "break",
                                10 => "move_up",
                                20 => "store",
                                _ => "Unknown",
                            });
                        if let Some(s) = read_f32(&mut binary, &mut reader) {
                            max_stack = max_stack.max(s as _);
                        }
                    }
                    4 => {
                        log::debug!("allocated");
                        //allocate needn't execute
                        binary.pop().unwrap();
                    }
                    11 => {
                        log::debug!("summon_e");
                        //name
                        read_str(&mut reader, &mut binary, true);

                        //xyz hp
                        read_f32(&mut binary, &mut reader);
                        read_f32(&mut binary, &mut reader);
                        read_f32(&mut binary, &mut reader);
                        read_f32(&mut binary, &mut reader);
                        //collide & args
                        reader.read(&mut buf[0..1]).unwrap();
                        binary.push(buf[0]);

                        for _ in 0..GameData::try_from(buf[0]).unwrap().get_args_count() {
                            read_f32(&mut binary, &mut reader);
                        }
                        //ai & args
                        let _script_name = read_str(&mut reader, &mut binary, true);
                        while let Some(_) = read_f32(&mut binary, &mut reader) {}
                    }
                    12 => {
                        log::debug!("summon_b");
                        //name
                        read_str(&mut reader, &mut binary, true);

                        //xyz scale angle
                        read_f32(&mut binary, &mut reader);
                        read_f32(&mut binary, &mut reader);
                        read_f32(&mut binary, &mut reader);
                        read_f32(&mut binary, &mut reader);
                        read_f32(&mut binary, &mut reader);
                        //collide & args
                        reader.read(&mut buf[0..1]).unwrap();
                        binary.push(buf[0]);

                        for _ in 0..GameData::try_from(buf[0]).unwrap().get_args_count() {
                            read_f32(&mut binary, &mut reader);
                        }
                        //ai & args
                        let _script_name = read_str(&mut reader, &mut binary, true);
                        while let Some(_) = read_f32(&mut binary, &mut reader) {}
                    }
                    38 | 39 => {
                        log::debug!("sin/cos command{}", buf[0]);
                        if let Some(s) = read_f32(&mut binary, &mut reader) {
                            max_stack = max_stack.max(s as _);
                        }
                        if let Some(s) = read_f32(&mut binary, &mut reader) {
                            max_stack = max_stack.max(s as _);
                        }
                    }
                    _ => {
                        log::debug!("byte command{}", buf[0]);
                    }
                }
            }
            let function_desc = FunctionDesc {
                code: binary,
                loops: loop_vec,
                max_stack
            };
            functions.insert(function_name, function_desc);
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
            writer.write(&x.code)?;
        }
        writer.flush()?;
        Ok(())
    }
}

fn parse_data(parser: &mut Parser, data: &mut HashMap<String, u8>) -> Result<(), Error> {
    loop {
        let mut line = String::new();
        parser.read_line(&mut line).unwrap();
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

fn parse_function(name: &str, parser: &mut Parser, context: &mut Context) -> Result<FunctionDesc, Error> {
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
        let size = parser.read_line(&mut raw_line).unwrap();
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
        //fixme: offer max_stack && loop vec
        Ok(FunctionDesc {
            code: binary,
            loops: vec![],
            max_stack: 0
        })
    }
}

fn summon_e(raw_args: &str, context: &Context, binary: &mut Vec<u8>) -> Result<(), Error> {
    binary.push(11);
    let args: Vec<&str> = raw_args.split(" ").collect();
    if args.len() < 5 {
        return Err(Error::new(ErrorKind::InvalidData, "[parse function]command args is not good (require 5..): ".to_owned() + raw_args));
    }

    // name x y z hp ai_name
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
    context.parse_value(args[5])?.flush(binary)?;

    let collide_rule = GameData::try_from(args[6])?;
    let read = collide_rule.get_args(&args[7..], context, binary)?;
    let index = 7 + read;
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

fn read_f32(binary: &mut Vec<u8>, reader: &mut BufReader<File>) -> Option<u8> {
    let mut buf = [0; 4];
    reader.read(&mut buf[0..1]).unwrap();
    binary.push(buf[0]);
    match buf[0] {
        0 => {
            log::debug!("point const ({})", buf[0]);
            reader.read(&mut buf[0..4]).unwrap();
            binary.push(buf[0]);
            binary.push(buf[1]);
            binary.push(buf[2]);
            binary.push(buf[3]);
        }
        3 => {
            log::debug!("point stack value ({})", buf[0]);
            reader.read(&mut buf[0..1]).unwrap();
            binary.push(buf[0]);
            return Some(buf[0]);
        }
        4 => {
            log::debug!("point calc value ({})", buf[0]);
        }
        9 => {
            log::debug!("no data");
        }
        _ => {
            log::debug!("point value ({})", buf[0]);
            reader.read(&mut buf[0..1]).unwrap();
            binary.push(buf[0]);
        }
    }
    None
}

fn read_str(reader: &mut BufReader<File>, binary: &mut Vec<u8>, write: bool) -> String {
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
    log::debug!("str: {}", str);
    str
}
