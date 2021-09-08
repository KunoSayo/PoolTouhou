use nalgebra::Vector3;
use pool_script::pool_script::FunctionDesc;
use pool_script::PoolScriptBin;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use crate::systems::game_system::CollideType;

pub mod script_context;

pub const ON_DIE_FUNCTION: &str = "on_die";

#[derive(Debug, Clone)]
pub struct ScriptDesc {
    version: u32,
    data_count: u8,
    index: usize,
    pub functions: HashMap<String, FunctionDesc>,
    pub tick_function: Option<FunctionDesc>,
}

#[derive(Debug, Default)]
pub struct ScriptManager {
    pub scripts: Vec<ScriptDesc>,
    pub script_map: HashMap<String, usize>,
}

impl ScriptManager {
    pub fn get_script_data_count(&self, name: &str) -> u8 {
        if let Some(index) = self.script_map.get(name) {
            self.scripts[*index].data_count
        } else {
            panic!("There is no script with name:  {}", name)
        }
    }

    pub fn load_script_data_count(&mut self, name: &str) -> u8 {
        if let Some(index) = self.script_map.get(name) {
            self.scripts[*index].data_count
        } else if let Some(script) = self.load_script(name) {
            script.data_count
        } else {
            panic!("There is no script with name:  {}", name)
        }
    }

    pub fn get_script(&self, name: &str) -> Option<&ScriptDesc> {
        if let Some(index) = self.script_map.get(name) {
            self.scripts.get(*index)
        } else {
            None
        }
    }

    pub(crate) fn load_script(&mut self, name: &str) -> Option<&ScriptDesc> {
        println!("loading script: {}", name);
        let path = PathBuf::from(std::env::current_dir().unwrap().to_str().unwrap().to_owned() + "/script/" + name + ".pthpsb");
        if let Ok(file) = File::open(&path) {
            let mut bin = PoolScriptBin::try_parse_bin(BufReader::new(file)).ok()?;
            let index = self.scripts.len();
            let tick_function = bin.functions.remove("tick");
            let script = ScriptDesc {
                version: bin.version,
                index,
                functions: Default::default(),
                tick_function,
                data_count: 0,
            };
            self.scripts.push(script);
            self.script_map.insert(name.into(), index);
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
    pub(crate) player_tran: Vector3<f32>,
    pub(crate) submit_command: Vec<ScriptGameCommand>,
    pub(crate) calc_stack: Vec<f32>,
}