use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use amethyst::core::transform::Transform;

use crate::script::{FunctionDesc, Loop, ScriptDesc, ScriptGameCommand, ScriptGameData, ScriptManager};
use crate::systems::game_system::CollideType;

pub struct ScriptContext {
    pub(crate) desc_index: usize,
    pub(crate) data: Vec<f32>,
    function_context: HashMap<String, FunctionContext>,
    tick_function: Option<FunctionContext>,
}

#[derive(Debug)]
pub struct TempGameContext<'a> {
    pub(crate) tran: Option<&'a mut Transform>,
}

impl ScriptContext {
    pub fn new(desc: &ScriptDesc, args: Vec<f32>) -> Self {
        Self {
            desc_index: desc.index,
            data: args,
            function_context: HashMap::new(),
            tick_function: desc.tick_function.as_ref().map(|f| FunctionContext::new(f.max_stack.into())),
        }
    }
}

impl ScriptContext {
    pub fn execute_function(&mut self, name: &String, game_data: &mut ScriptGameData, script_manager: &mut ScriptManager, temp: &mut TempGameContext) -> Option<f32> {
        let function = script_manager.scripts.get(self.desc_index)
            .unwrap().functions.get(name).expect("no such function.");
        let function_context;
        if let Some(ctx) = self.function_context.get_mut(name) {
            function_context = ctx;
        } else {
            self.function_context.insert(name.clone(), FunctionContext::new(function.max_stack as usize));
            function_context = self.function_context.get_mut(name).unwrap();
        }
        let mut function_runner = FunctionRunner {
            data: &mut self.data,
            desc: function,
            game: game_data,
            context: function_context,
            temp,
        };

        function_runner.execute(script_manager)
    }

    pub fn tick_function(&mut self, game_data: &mut ScriptGameData, script_manager: &mut ScriptManager, temp: &mut TempGameContext) -> Option<f32> {
        let f_with_ctx = self.tick_function.as_mut().unwrap();
        let desc = script_manager.scripts.get(self.desc_index).unwrap().tick_function
            .as_ref().unwrap();
        let mut function_runner = FunctionRunner {
            data: &mut self.data,
            desc,
            game: game_data,
            context: f_with_ctx,
            temp,
        };

        function_runner.execute(script_manager)
    }
}

#[derive(Debug)]
struct FunctionContext {
    var_stack: Vec<f32>,
    loop_start: Vec<usize>,
    pointer: usize,
}

impl FunctionContext {
    fn new(max_stack: usize) -> Self {
        let mut stack_vec = Vec::new();
        stack_vec.resize_with(max_stack, || 0.0);
        Self {
            var_stack: stack_vec,
            loop_start: Vec::with_capacity(2),
            pointer: 0,
        }
    }
}


impl FunctionContext {
    #[inline]
    fn reset(&mut self) {
        self.loop_start.clear();
        self.pointer = 0;
    }
}

struct FunctionRunner<'a, 'c> {
    data: &'a mut Vec<f32>,
    desc: &'a FunctionDesc,
    game: &'a mut ScriptGameData,
    context: &'a mut FunctionContext,
    temp: &'a mut TempGameContext<'c>,
}

impl<'a, 'c> FunctionRunner<'a, 'c> {
    pub fn execute(&mut self, script_manager: &ScriptManager) -> Option<f32> {
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
                    } else {
                        self.context.reset();
                        break;
                    }
                }
                1 => {
                    self.context.loop_start.push(self.context.pointer);
                }
                2 => {
                    return None;
                }
                3 => {
                    let data = self.get_f32();
                    self.game.calc_stack.push(data);
                }
                5 => {
                    let times = self.get_f32();
                    if times >= 1.0 {
                        let times = times.floor() as i32;
                        for _ in 0..times {
                            if let Some(_) = self.context.loop_start.pop() {
                                let mut layer = 0;
                                for x in self.desc.loops.iter() {
                                    match x {
                                        Loop::Start(p) => {
                                            if self.context.pointer < *p {
                                                layer += 1;
                                            }
                                        }
                                        Loop::End(p) => {
                                            if self.context.pointer < *p {
                                                if layer == 0 {
                                                    self.context.pointer = *p;
                                                    break;
                                                }
                                                layer -= 1;
                                            }
                                        }
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
                    let arg_len = script_manager.get_script_data_count(&ai_name);
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
                    let arg_len = script_manager.get_script_data_count(&ai_name);
                    let mut args = Vec::with_capacity(arg_len as usize);
                    for _ in 0..arg_len {
                        args.push(self.get_f32());
                    }
                    self.game.submit_command.push(ScriptGameCommand::SummonBullet(name, x, y, z, angle, collide, ai_name, args));
                }
                20 => {
                    let value = self.game.calc_stack.pop().unwrap();
                    self.store_f32(value);
                }
                21 => {
                    let x = self.game.calc_stack.pop().unwrap();
                    let y = self.game.calc_stack.last_mut().unwrap();
                    *y = *y + x;
                }
                22 => {
                    let x = self.game.calc_stack.pop().unwrap();
                    let y = self.game.calc_stack.last_mut().unwrap();
                    *y = *y - x;
                }
                23 => {
                    let x = self.game.calc_stack.pop().unwrap();
                    let y = self.game.calc_stack.last_mut().unwrap();
                    *y = *y * x;
                }
                24 => {
                    let x = self.game.calc_stack.pop().unwrap();
                    let y = self.game.calc_stack.last_mut().unwrap();
                    *y = *y / x;
                }
                25 => {
                    let x = self.game.calc_stack.pop().unwrap();
                    let y = self.game.calc_stack.last_mut().unwrap();
                    *y = *y % x;
                }
                _ => panic!("Unknown byte command: {}", command)
            }
        }
        None
    }
    #[inline]
    fn get_str(&mut self) -> String {
        let count = &self.desc.code[self.context.pointer..self.context.pointer + 2 as usize];
        let count = u16::from_be_bytes(count.try_into().unwrap());
        self.context.pointer += 2;
        let bytes = &self.desc.code[self.context.pointer..self.context.pointer + count as usize];
        self.context.pointer += count as usize;
        unsafe {
            String::from_utf8_unchecked(bytes.try_into().unwrap())
        }
    }

    #[inline]
    fn store_f32(&mut self, value: f32) {
        let src = self.desc.code[self.context.pointer];
        let index = self.desc.code[self.context.pointer + 1];
        self.context.pointer += 2;
        match src {
            1 => {
                match index {
                    0 => {
                        let mut tran = self.temp.tran.as_mut().unwrap().clone();
                        tran.set_translation_x(value);
                    }
                    1 => {
                        let mut tran = self.temp.tran.as_mut().unwrap().clone();
                        tran.set_translation_y(value);
                    }
                    2 => {
                        let mut tran = self.temp.tran.as_mut().unwrap().clone();
                        tran.set_translation_z(value);
                    }
                    3 => {
                        let tran = &mut self.game.player_tran;
                        tran.set_translation_x(value);
                        // self.game.player_tran.replace(tran);
                    }
                    4 => {
                        let tran = &mut self.game.player_tran;
                        tran.set_translation_y(value);
                        // self.game.player_tran.replace(tran);
                    }
                    5 => {
                        let tran = &mut self.game.player_tran;
                        tran.set_translation_z(value);
                        // self.game.player_tran.replace(tran);
                    }
                    _ => panic!("Unknown game data byte: {}", index)
                };
            }
            2 => {
                self.data[index as usize] = value;
            }
            3 => {
                self.context.var_stack[index as usize] = value;
            }
            _ => panic!("Unknown data src: {}", src)
        }
    }
    #[inline]
    fn get_f32(&mut self) -> f32 {
        let src = self.desc.code[self.context.pointer];
        match src {
            0 => {
                let data = self.desc.code[self.context.pointer + 1..self.context.pointer + 5].try_into().unwrap();
                self.context.pointer += 5;
                f32::from_be_bytes(data)
            }
            1 => {
                let data = self.desc.code[self.context.pointer + 1];
                self.context.pointer += 2;
                match data {
                    0 => self.temp.tran.as_ref().unwrap().translation().x,
                    1 => self.temp.tran.as_ref().unwrap().translation().y,
                    2 => self.temp.tran.as_ref().unwrap().translation().z,
                    3 => self.game.player_tran.translation().x,
                    4 => self.game.player_tran.translation().y,
                    5 => self.game.player_tran.translation().z,
                    _ => panic!("Unknown game data byte: {}", data)
                }
            }
            2 => {
                let data = self.desc.code[self.context.pointer + 1];
                self.context.pointer += 2;
                self.data[data as usize]
            }
            3 => {
                let data = self.desc.code[self.context.pointer + 1];
                self.context.pointer += 2;
                self.context.var_stack[data as usize]
            }
            4 => {
                self.context.pointer += 1;
                self.game.calc_stack.pop().unwrap()
            }
            _ => panic!("Unknown data src: {}", src)
        }
    }
}