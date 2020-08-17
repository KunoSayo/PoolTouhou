use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::Arc;

use amethyst::core::components::Transform;

use crate::systems::game_system::CollideType;

pub mod script_context;

#[derive(Debug, Clone)]
pub struct FunctionDesc {
    code: Arc<Vec<u8>>,
    loop_exit: Arc<Vec<usize>>,
    max_stack: u16,
}

#[derive(Debug, Clone)]
pub struct ScriptDesc {
    version: u32,
    data_count: u8,
    pub(crate) functions: HashMap<String, FunctionDesc>,
}

#[derive(Debug, Default)]
pub struct ScriptManager {
    pub scripts: HashMap<String, ScriptDesc>
}

impl ScriptManager {
    pub fn get_script_data_count(&mut self, name: &String) -> u8 {
        if let Some(script) = self.scripts.get(name) {
            script.data_count
        } else if let Some(script) = self.load_script(name) {
            script.data_count
        } else {
            panic!("There is no script with name: ".to_owned() + name)
        }
    }

    pub fn get_script(&mut self, name: &String) -> Option<&ScriptDesc> {
        self.scripts.get(name)
    }

    pub fn load_script(&mut self, name: &String) -> Option<&ScriptDesc> {
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
                let mut loop_exit = Vec::new();
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
                                loop_exit.push(binary.len());
                            } else {
                                break;
                            }
                        }
                        1 => {
                            loops += 1;
                        }
                        3 | 5 | 10 | 20 => {
                            if let Some(s) = read_f32(&mut binary, &mut reader) {
                                max_stack = max_stack.max(s as i16);
                            }
                        }
                        11 => {
                            //name
                            read_str(&mut reader, &mut binary, true);

                            //xy hp
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
                            let script_name = read_str(&mut reader, &mut binary, true);
                            let ai_args_count;
                            if script_name == *name {
                                ai_args_count = data_count;
                            } else {
                                ai_args_count = self.get_script_data_count(&script_name);
                            }
                            for _ in 0..ai_args_count {
                                read_f32(&mut binary, &mut reader);
                            }
                        }
                        12 => {
                            //name
                            read_str(&mut reader, &mut binary, true);

                            //xyz angle
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
                            let script_name = read_str(&mut reader, &mut binary, true);
                            let ai_args_count;
                            if script_name == *name {
                                ai_args_count = data_count;
                            } else {
                                ai_args_count = self.get_script_data_count(&script_name);
                            }
                            for _ in 0..ai_args_count {
                                read_f32(&mut binary, &mut reader);
                            }
                        }
                        _ => {}
                    }
                }
                functions.insert(function_name, FunctionDesc {
                    code: Arc::new(binary),
                    loop_exit: Arc::new(loop_exit),
                    max_stack: (max_stack + 1) as u16,
                });
            }
            let script = ScriptDesc {
                version,
                data_count,
                functions,
            };
            self.scripts.insert(name.clone(), script);
            return self.scripts.get(name);
        } else {
            eprintln!("Script not found in {:?}", path);
        }
        None
    }
}

#[derive(Debug)]
pub enum ScriptGameCommand {
    MoveUp(f32),
    SummonEnemy(String, f32, f32, f32, CollideType, String, Vec<f32>),
    SummonBullet(String, f32, f32, f32, f32, CollideType, String, Vec<f32>),
}

#[derive(Debug)]
pub struct ScriptGameData<'a> {
    pub(crate) player_tran: Option<Transform>,
    pub(crate) submit_command: Vec<ScriptGameCommand>,
    pub(crate) script_manager: Option<&'a mut ScriptManager>,
    pub(crate) calc_stack: Vec<f32>,
}

fn read_f32(binary: &mut Vec<u8>, reader: &mut BufReader<File>) -> Option<u8> {
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
            return Some(buf[0]);
        }
        _ => {
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

    String::from_utf8(vec).expect("parse utf8 str failed")
}