use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use amethyst::core::components::Transform;

use crate::systems::game_system::CollideType;

pub mod script_context;

pub const ON_DIE_FUNCTION: &str = "on_die";

#[derive(Debug, Copy, Clone)]
pub enum Loop {
    Start(usize),
    End(usize),
}

#[derive(Debug, Clone)]
pub struct FunctionDesc {
    code: Vec<u8>,
    loops: Vec<Loop>,
    max_stack: u16,
}

#[derive(Debug, Clone)]
pub struct ScriptDesc {
    version: u32,
    data_count: u8,
    index: usize,
    pub(crate) functions: HashMap<String, FunctionDesc>,
    pub(crate) tick_function: Option<FunctionDesc>,
}

#[derive(Debug, Default)]
pub struct ScriptManager {
    pub scripts: Vec<ScriptDesc>,
    pub script_map: HashMap<String, usize>,
}

impl ScriptManager {
    pub fn get_script_data_count(&self, name: &String) -> u8 {
        if let Some(index) = self.script_map.get(name) {
            self.scripts[*index].data_count
        } else {
            panic!("There is no script with name:  {}", name)
        }
    }

    pub fn load_script_data_count(&mut self, name: &String) -> u8 {
        if let Some(index) = self.script_map.get(name) {
            self.scripts[*index].data_count
        } else if let Some(script) = self.load_script(name) {
            script.data_count
        } else {
            panic!("There is no script with name:  {}", name)
        }
    }

    pub fn get_script(&mut self, name: &String) -> Option<&ScriptDesc> {
        if let Some(index) = self.script_map.get(name) {
            self.scripts.get(*index)
        } else {
            None
        }
    }

    pub(crate) fn load_script(&mut self, name: &String) -> Option<&ScriptDesc> {
        println!("loading script: {}", name);
        let path = PathBuf::from(std::env::current_dir().unwrap().to_str().unwrap().to_owned() + "/script/" + name + ".pthpsb");
        if let Ok(file) = File::open(&path) {
            let mut reader = BufReader::new(file);
            let mut buf = [0; 16];
            let size = reader.read(&mut buf[0..5]).expect("read file failed");
            if size < 5 {
                eprintln!("load script {} failed", name);
                return None;
            }
            let version = u32::from_be_bytes(buf[0..4].try_into().unwrap());
            let data_count = buf[4];
            let mut functions = HashMap::new();
            loop {
                let mut binary = Vec::with_capacity(128);
                let mut loop_vec = Vec::new();
                let mut max_stack: i16 = -1;
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
                            if loops > 0 {
                                loops -= 1;
                                loop_vec.push(Loop::End(binary.len()));
                            } else {
                                break;
                            }
                        }
                        1 => {
                            loop_vec.push(Loop::Start(binary.len()));
                            loops += 1;
                        }
                        3 | 5 | 6 | 10 | 20 => {
                            if let (Some(s), _) = read_f32(&mut binary, &mut reader) {
                                max_stack = max_stack.max(s as i16);
                            }
                        }
                        4 => {
                            //allocate needn't execute
                            binary.pop().unwrap();
                        }
                        11 => {
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
                            for _ in 0..CollideType::get_arg_count(buf[0]) {
                                read_f32(&mut binary, &mut reader);
                            }
                            //ai & args
                            let _script_name = read_str(&mut reader, &mut binary, true);
                            while read_f32(&mut binary, &mut reader).1 {}
                        }
                        12 => {
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
                            for _ in 0..CollideType::get_arg_count(buf[0]) {
                                read_f32(&mut binary, &mut reader);
                            }
                            //ai & args
                            let _script_name = read_str(&mut reader, &mut binary, true);

                            while read_f32(&mut binary, &mut reader).1 {}
                        }
                        38 | 39 => {
                            if let (Some(s), _) = read_f32(&mut binary, &mut reader) {
                                max_stack = max_stack.max(s as i16);
                            }
                            if let (Some(s), _) = read_f32(&mut binary, &mut reader) {
                                max_stack = max_stack.max(s as i16);
                            }
                        }
                        _ => {}
                    }
                }
                functions.insert(function_name, FunctionDesc {
                    code: binary,
                    loops: loop_vec,
                    max_stack: (max_stack + 1) as u16,
                });
            }
            let index = self.scripts.len();
            let tick_function = functions.remove("tick");
            let script = ScriptDesc {
                version,
                data_count,
                index,
                functions,
                tick_function,
            };
            self.scripts.push(script);
            self.script_map.insert(name.clone(), index);
            return self.scripts.get(index);
        } else {
            eprintln!("Script not found in {:?}", path);
        }
        None
    }

    pub fn load_scripts(&mut self) {
        self.scripts.clear();
        self.script_map.clear();
        let path = PathBuf::from(std::env::current_dir().unwrap().to_str().unwrap().to_owned() + "/script/");
        let dir = path.read_dir().unwrap();
        for file in dir {
            match file {
                Ok(entry) => {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() && entry.file_name().to_str().to_owned().unwrap().ends_with(".pthpsb") {
                            let name = &entry.file_name().into_string().unwrap().replace(".pthpsb", "");
                            if !self.script_map.contains_key(name) {
                                self.load_script(name);
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("read entry failed! {}", err);
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum ScriptGameCommand {
    MoveUp(f32),
    SummonEnemy(String, f32, f32, f32, f32, CollideType, String, Vec<f32>),
    SummonBullet(String, f32, f32, f32, f32, f32, CollideType, String, Vec<f32>),
    Kill,
}

#[derive(Debug)]
pub struct ScriptGameData {
    pub(crate) player_tran: Transform,
    pub(crate) submit_command: Vec<ScriptGameCommand>,
    pub(crate) calc_stack: Vec<f32>,
}

fn read_f32(binary: &mut Vec<u8>, reader: &mut BufReader<File>) -> (Option<u8>, bool) {
    let mut buf = [0; 4];
    reader.read(&mut buf[0..1]).unwrap();
    binary.push(buf[0]);
    match buf[0] {
        0 => {
            reader.read(&mut buf[0..4]).unwrap();
            binary.push(buf[0]);
            binary.push(buf[1]);
            binary.push(buf[2]);
            binary.push(buf[3]);
        }
        3 => {
            reader.read(&mut buf[0..1]).unwrap();
            binary.push(buf[0]);
            return (Some(buf[0]), true);
        }
        4 | 9 => {
            return (None, false);
        }
        _ => {
            reader.read(&mut buf[0..1]).unwrap();
            binary.push(buf[0]);
        }
    }
    (None, true)
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

    String::from_utf8(vec).expect("parse utf8 str failed")
}