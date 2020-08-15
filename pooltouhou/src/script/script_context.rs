use std::convert::{TryFrom, TryInto};

use crate::script::{FunctionDesc, ScriptDesc, ScriptGameCommand, ScriptGameData};
use crate::systems::game_system::CollideType;

pub struct ScriptContext {
    pub(crate) desc: ScriptDesc,
    pub(crate) data: Vec<f32>,
}

impl ScriptContext {
    pub fn execute_function(&mut self, name: &String, game_data: &mut ScriptGameData) {
        let function = self.desc.functions.get(name).expect(&*("No function ".to_owned() + name));
        let mut function_context = FunctionContext {
            data: &mut self.data,
            desc: function,
            var_stack: vec![],
            calc_stack: vec![],
            loop_start: vec![],
            game: game_data,
            pointer: 0,
        };
        function_context.execute();
    }
}

struct FunctionContext<'a, 'b> {
    data: &'a mut Vec<f32>,
    desc: &'a FunctionDesc,
    var_stack: Vec<f32>,
    calc_stack: Vec<f32>,
    loop_start: Vec<usize>,
    game: &'a mut ScriptGameData<'b>,
    pointer: usize,
}

impl<'a, 'b> FunctionContext<'a, 'b> {
    pub fn execute(&mut self) -> Option<f32> {
        self.pointer = 0;
        self.var_stack.clear();
        self.loop_start.clear();
        loop {
            if self.pointer >= self.desc.code.len() {
                break;
            }
            let command = self.desc.code[self.pointer];
            self.pointer += 1;
            match command {
                0 => {
                    if self.loop_start.len() > 0 {
                        self.pointer = *self.loop_start.last().unwrap();
                    } else {
                        break;
                    }
                }
                1 => {
                    self.loop_start.push(self.pointer);
                }
                2 => {
                    return None;
                }
                3 => {
                    let data = self.get_f32();
                    self.calc_stack.push(data);
                }
                4 => {
                    self.var_stack.push(0.0);
                }
                5 => {
                    let times = self.get_f32();
                    let times = times.floor() as i32;
                    for _ in 0..times {
                        if let Some(_) = self.loop_start.pop() {
                            for x in self.desc.loop_exit.to_vec() {
                                if x > self.pointer {
                                    self.pointer = x;
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }
                10 => {
                    let v = self.get_f32();
                    self.game.submit_command.push(ScriptGameCommand::MoveUp(v));
                }
                11 => {
                    let name = self.get_str();
                    let x = self.get_f32();
                    let y = self.get_f32();
                    let hp = self.get_f32();
                    let ai_name = self.get_str();
                    let arg_len = self.game.script_manager.get_script_data_count(&ai_name);
                    let mut args = Vec::with_capacity(arg_len as usize);
                    for _ in 0..arg_len {
                        args.push(self.get_f32());
                    }
                    self.game.submit_command.push(ScriptGameCommand::SummonEnemy(name, x, y, hp, ai_name, args));
                }
                12 => {
                    let name = self.get_str();
                    let x = self.get_f32();
                    let y = self.get_f32();
                    let z = self.get_f32();
                    let angle = self.get_f32();
                    let collide_byte = self.desc.code[self.pointer];
                    self.pointer += 1;
                    let collide_arg_len = CollideType::get_arg_count(collide_byte);
                    let mut collide_args = Vec::with_capacity(collide_arg_len as usize);
                    for _ in 0..collide_arg_len {
                        collide_args.push(self.get_f32());
                    }
                    let collide = CollideType::try_from((collide_byte, collide_args))
                        .unwrap();
                    let ai_name = self.get_str();
                    let arg_len = self.game.script_manager.get_script_data_count(&ai_name);
                    let mut args = Vec::with_capacity(arg_len as usize);
                    for _ in 0..arg_len {
                        args.push(self.get_f32());
                    }
                    self.game.submit_command.push(ScriptGameCommand::SummonBullet(name, x, y, z, angle, collide, ai_name, args));
                }
                20 => {
                    let value = self.calc_stack.pop().unwrap();
                    self.store_f32(value);
                }
                21 => {
                    let x = self.calc_stack.pop().unwrap();
                    let y = self.calc_stack.pop().unwrap();
                    self.calc_stack.push(x + y);
                }
                22 => {
                    let x = self.calc_stack.pop().unwrap();
                    let y = self.calc_stack.pop().unwrap();
                    self.calc_stack.push(y - x);
                }
                23 => {
                    let x = self.calc_stack.pop().unwrap();
                    let y = self.calc_stack.pop().unwrap();
                    self.calc_stack.push(x * y);
                }
                _ => panic!("Unknown byte command: {}", command)
            }
        }
        None
    }

    fn get_str(&mut self) -> String {
        let count = &self.desc.code[self.pointer..self.pointer + 2 as usize];
        let count = u16::from_be_bytes(count.try_into().unwrap());
        self.pointer += 2;
        let bytes = &self.desc.code[self.pointer..self.pointer + count as usize];
        self.pointer += count as usize;
        String::from_utf8(bytes.try_into().unwrap()).unwrap()
    }

    fn store_f32(&mut self, value: f32) {
        let src = self.desc.code[self.pointer];
        self.pointer += 1;
        match src {
            1 => {
                let data = self.desc.code[self.pointer];
                self.pointer += 1;
                match data {
                    0 => {
                        let mut tran = self.game.tran.as_ref().unwrap().clone();
                        tran.set_translation_x(value);
                        self.game.tran.replace(tran);
                    }
                    1 => {
                        let mut tran = self.game.tran.as_ref().unwrap().clone();
                        tran.set_translation_y(value);
                        self.game.tran.replace(tran);
                    }
                    2 => {
                        let mut tran = self.game.tran.as_ref().unwrap().clone();
                        tran.set_translation_z(value);
                        self.game.tran.replace(tran);
                    }
                    3 => {
                        let mut tran = self.game.player_tran.as_ref().unwrap().clone();
                        tran.set_translation_x(value);
                        self.game.player_tran.replace(tran);
                    }
                    4 => {
                        let mut tran = self.game.player_tran.as_ref().unwrap().clone();
                        tran.set_translation_y(value);
                        self.game.player_tran.replace(tran);
                    }
                    5 => {
                        let mut tran = self.game.player_tran.as_ref().unwrap().clone();
                        tran.set_translation_z(value);
                        self.game.player_tran.replace(tran);
                    }
                    _ => panic!("Unknown game data byte: {}", data)
                };
            }
            2 => {
                let data = self.desc.code[self.pointer];
                self.pointer += 1;
                self.data[data as usize] = value;
            }
            3 => {
                let data = self.desc.code[self.pointer];
                self.pointer += 1;
                self.var_stack[data as usize] = value;
            }
            _ => panic!("Unknown data src: {}", src)
        }
    }
    fn get_f32(&mut self) -> f32 {
        let src = self.desc.code[self.pointer];
        self.pointer += 1;
        match src {
            0 => {
                let data = self.desc.code[self.pointer..self.pointer + 4].try_into().unwrap();
                self.pointer += 4;
                f32::from_be_bytes(data)
            }
            1 => {
                let data = self.desc.code[self.pointer];
                self.pointer += 1;
                match data {
                    0 => self.game.tran.as_ref().unwrap().translation().x,
                    1 => self.game.tran.as_ref().unwrap().translation().y,
                    2 => self.game.tran.as_ref().unwrap().translation().z,
                    3 => self.game.player_tran.as_ref().unwrap().translation().x,
                    4 => self.game.player_tran.as_ref().unwrap().translation().y,
                    5 => self.game.player_tran.as_ref().unwrap().translation().z,
                    _ => panic!("Unknown game data byte: {}", data)
                }
            }
            2 => {
                let data = self.desc.code[self.pointer];
                self.pointer += 1;
                self.data[data as usize]
            }
            3 => {
                let data = self.desc.code[self.pointer];
                self.pointer += 1;
                self.var_stack[data as usize]
            }
            _ => panic!("Unknown data src: {}", src)
        }
    }
}