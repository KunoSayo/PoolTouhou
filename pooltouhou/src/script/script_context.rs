use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use crate::script::{FunctionDesc, ScriptDesc, ScriptGameCommand, ScriptGameData};
use crate::systems::game_system::CollideType;

pub struct ScriptContext {
    pub(crate) desc: ScriptDesc,
    pub(crate) data: Vec<f32>,
    function_context: HashMap<String, FunctionContext>,
}

impl ScriptContext {
    pub fn new(desc: &ScriptDesc, args: Vec<f32>) -> Self {
        Self {
            desc: desc.clone(),
            data: args,
            function_context: HashMap::new(),
        }
    }
}

impl ScriptContext {
    pub fn execute_function(&mut self, name: &String, game_data: &mut ScriptGameData) {
        let function = self.desc.functions.get(name).expect(&*("No function ".to_owned() + name));
        let function_context;
        if let Some(ctx) = self.function_context.get_mut(name) {
            function_context = ctx;
        } else {
            self.function_context.insert(name.clone(), FunctionContext::default());
            function_context = self.function_context.get_mut(name).unwrap();
        }
        let mut function_runner = FunctionRunner {
            data: &mut self.data,
            desc: function,
            game: game_data,
            context: function_context,
        };

        function_runner.execute();
    }
}

#[derive(Debug)]
struct FunctionContext {
    var_stack: Vec<f32>,
    var_per_stack: Vec<u8>,
    calc_stack: Vec<f32>,
    loop_start: Vec<usize>,
    pointer: usize,
}

impl Default for FunctionContext {
    fn default() -> Self {
        Self {
            var_stack: Vec::with_capacity(4),
            var_per_stack: vec![0],
            calc_stack: Vec::with_capacity(4),
            loop_start: Vec::with_capacity(2),
            pointer: 0,
        }
    }
}

impl FunctionContext {
    fn reset(&mut self) {
        self.var_stack.clear();
        self.var_per_stack.clear();
        self.var_per_stack.push(0);
        self.calc_stack.clear();
        self.loop_start.clear();
        self.pointer = 0;
    }
}

struct FunctionRunner<'a, 'b> {
    data: &'a mut Vec<f32>,
    desc: &'a FunctionDesc,
    game: &'a mut ScriptGameData<'b>,
    context: &'a mut FunctionContext,
}

impl<'a, 'b> FunctionRunner<'a, 'b> {
    pub fn execute(&mut self) -> Option<f32> {
        loop {
            if self.context.pointer >= self.desc.code.len() {
                self.context.reset();
                break;
            }
            let command = self.desc.code[self.context.pointer];
            self.context.pointer += 1;
            match command {
                0 => {
                    if self.context.loop_start.len() > 0 {
                        self.context.pointer = *self.context.loop_start.last().unwrap();
                        *self.context.var_per_stack.last_mut().unwrap() = 0;
                    } else {
                        self.context.reset();
                        break;
                    }
                }
                1 => {
                    self.context.loop_start.push(self.context.pointer);
                    self.context.var_per_stack.push(0);
                }
                2 => {
                    return None;
                }
                3 => {
                    let data = self.get_f32();
                    self.context.calc_stack.push(data);
                }
                4 => {
                    self.context.var_stack.push(0.0);
                    *self.context.var_per_stack.last_mut().unwrap() += 1;
                }
                5 => {
                    let times = self.get_f32();
                    let times = times.floor() as i32;
                    if times > 0 {
                        for _ in 0..times {
                            if let Some(_) = self.context.loop_start.pop() {
                                for x in self.desc.loop_exit.to_vec() {
                                    if x > self.context.pointer {
                                        self.context.pointer = x;
                                        for _ in 0..self.context.var_per_stack.pop().unwrap() {
                                            self.context.var_stack.pop().unwrap();
                                        }
                                        break;
                                    }
                                }
                            } else {
                                break;
                            }
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

                    let collide_byte = self.desc.code[self.context.pointer];
                    self.context.pointer += 1;
                    let collide_arg_len = CollideType::get_arg_count(collide_byte);
                    let mut collide_args = Vec::with_capacity(collide_arg_len as usize);
                    for _ in 0..collide_arg_len {
                        collide_args.push(self.get_f32());
                    }
                    let collide = CollideType::try_from((collide_byte, collide_args))
                        .unwrap();

                    let ai_name = self.get_str();
                    let arg_len = self.game.script_manager.as_mut().unwrap().get_script_data_count(&ai_name);
                    let mut args = Vec::with_capacity(arg_len as usize);
                    for _ in 0..arg_len {
                        args.push(self.get_f32());
                    }
                    self.game.submit_command.push(ScriptGameCommand::SummonEnemy(name, x, y, hp, collide, ai_name, args));
                }
                12 => {
                    let name = self.get_str();
                    let x = self.get_f32();
                    let y = self.get_f32();
                    let z = self.get_f32();
                    let angle = self.get_f32();
                    let collide_byte = self.desc.code[self.context.pointer];
                    self.context.pointer += 1;
                    let collide_arg_len = CollideType::get_arg_count(collide_byte);
                    let mut collide_args = Vec::with_capacity(collide_arg_len as usize);
                    for _ in 0..collide_arg_len {
                        collide_args.push(self.get_f32());
                    }
                    let collide = CollideType::try_from((collide_byte, collide_args))
                        .unwrap();
                    let ai_name = self.get_str();
                    let arg_len = self.game.script_manager.as_mut().unwrap().get_script_data_count(&ai_name);
                    let mut args = Vec::with_capacity(arg_len as usize);
                    for _ in 0..arg_len {
                        args.push(self.get_f32());
                    }
                    self.game.submit_command.push(ScriptGameCommand::SummonBullet(name, x, y, z, angle, collide, ai_name, args));
                }
                20 => {
                    let value = self.context.calc_stack.pop().unwrap();
                    self.store_f32(value);
                }
                21 => {
                    let x = self.context.calc_stack.pop().unwrap();
                    let y = self.context.calc_stack.pop().unwrap();
                    self.context.calc_stack.push(x + y);
                }
                22 => {
                    let x = self.context.calc_stack.pop().unwrap();
                    let y = self.context.calc_stack.pop().unwrap();
                    self.context.calc_stack.push(y - x);
                }
                23 => {
                    let x = self.context.calc_stack.pop().unwrap();
                    let y = self.context.calc_stack.pop().unwrap();
                    self.context.calc_stack.push(x * y);
                }
                _ => panic!("Unknown byte command: {}", command)
            }
        }
        None
    }

    fn get_str(&mut self) -> String {
        let count = &self.desc.code[self.context.pointer..self.context.pointer + 2 as usize];
        let count = u16::from_be_bytes(count.try_into().unwrap());
        self.context.pointer += 2;
        let bytes = &self.desc.code[self.context.pointer..self.context.pointer + count as usize];
        self.context.pointer += count as usize;
        String::from_utf8(bytes.try_into().unwrap()).unwrap()
    }

    fn store_f32(&mut self, value: f32) {
        let src = self.desc.code[self.context.pointer];
        self.context.pointer += 1;
        match src {
            1 => {
                let data = self.desc.code[self.context.pointer];
                self.context.pointer += 1;
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
                let data = self.desc.code[self.context.pointer];
                self.context.pointer += 1;
                self.data[data as usize] = value;
            }
            3 => {
                let data = self.desc.code[self.context.pointer];
                self.context.pointer += 1;
                self.context.var_stack[data as usize] = value;
            }
            _ => panic!("Unknown data src: {}", src)
        }
    }
    fn get_f32(&mut self) -> f32 {
        let src = self.desc.code[self.context.pointer];
        self.context.pointer += 1;
        match src {
            0 => {
                let data = self.desc.code[self.context.pointer..self.context.pointer + 4].try_into().unwrap();
                self.context.pointer += 4;
                f32::from_be_bytes(data)
            }
            1 => {
                let data = self.desc.code[self.context.pointer];
                self.context.pointer += 1;
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
                let data = self.desc.code[self.context.pointer];
                self.context.pointer += 1;
                self.data[data as usize]
            }
            3 => {
                let data = self.desc.code[self.context.pointer];
                self.context.pointer += 1;
                self.context.var_stack[data as usize]
            }
            _ => panic!("Unknown data src: {}", src)
        }
    }
}