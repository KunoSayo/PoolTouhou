use pthapi::{CollideType, GAME_MAX_X, GAME_MAX_Y, GAME_MIN_X, GAME_MIN_Y, Player, PlayerBullet, PosType, Rotation, SimpleEnemyBullet, TexHandle};

use crate::LoopState;
use crate::render::texture2d::Texture2DObject;
use crate::script::{ON_DIE_FUNCTION, ScriptGameCommand, ScriptGameData, ScriptManager};
use crate::script::script_context::{ScriptContext, TempGameContext};
use crate::states::{GameState, StateData, Trans};

pub mod anime;

pub struct Enemy {
    pub pos: PosType,
    pub hp: f32,
    pub collide: CollideType,
    pub script: ScriptContext,
    pub tex: TexHandle,
}

pub struct EnemyBullet {
    pub pos: PosType,
    pub rot: Rotation,
    pub scale: f32,
    pub tex: TexHandle,
    pub collide: CollideType,
    pub script: ScriptContext,
}

impl Enemy {
    pub fn new(pos: PosType, hp: f32, collide: CollideType, script: ScriptContext, tex: TexHandle) -> Self {
        Self {
            pos,
            hp,
            collide,
            script,
            tex,
        }
    }
}


#[derive(Default)]
pub struct Gaming {
    player: Player,
    script_manager: ScriptManager,
    player_bullets: Vec<PlayerBullet>,
    enemies: Vec<Enemy>,
    enemy_bullets: Vec<EnemyBullet>,
    simple_bullets: Vec<SimpleEnemyBullet>,
    commands: Vec<ScriptGameCommand>,
    obj: Vec<Texture2DObject>,
    tick: u128,
}

impl GameState for Gaming {
    fn start(&mut self, data: &mut StateData) {
        log::info!("Gaming state starting");
        let mut game = ScriptGameData {
            player_tran: self.player.pos,
            submit_command: vec![],
            calc_stack: vec![],
        };
        self.player.pos.1 = -100.0;
        self.player.tex = data.global_state.handles.texture_map.read().unwrap()["sheep"];
        data.render.render2d.add_tex(data.global_state, self.player.tex);
        self.script_manager.load_scripts();
        log::info!("loaded all scripts");
        {
            let script = self.script_manager.get_script("main").unwrap();
            let mut context = ScriptContext::new(&script, vec![]);


            let mut temp = TempGameContext {
                tran: None,
            };
            context.execute_function("start", &mut game, &mut self.script_manager, &mut temp);
        }
        for x in game.submit_command {
            match x {
                crate::script::ScriptGameCommand::SummonEnemy(name, x, y, z, hp, collide, script_name, args) => {
                    let script = self.script_manager.get_script(&script_name).expect(&format!("Using unloaded script {}", name));
                    let tex = data.global_state.handles.texture_map.read().unwrap()[&name];
                    data.render.render2d.add_tex(data.global_state, tex);
                    self.enemies.push(Enemy {
                        pos: (x, y, z),
                        tex,
                        collide,
                        script: ScriptContext::new(script, args),
                        hp,
                    });
                }
                _ => panic!("没实现哪里来的命令（大声）")
            }
        }

        println!("Gaming state started.");
    }

    fn update(&mut self, _: &mut StateData) -> (Trans, LoopState) {
        (Trans::None, LoopState::POLL)
    }


    fn game_tick(&mut self, data: &mut StateData) -> Trans {
        log::trace!("gaming state ticking");
        self.tick += 1;

        let input = &data.inputs.cur_game_input;
        self.player.walking = input.slow > 0;
        let (mov_x, mov_y) = input.get_move(if self.player.walking {
            self.player.walk_speed
        } else {
            self.player.move_speed
        });
        self.player.pos.0 += mov_x;
        self.player.pos.1 += mov_y;
        self.player.pos.0 = self.player.pos.0.min(pthapi::GAME_MAX_X).max(pthapi::GAME_MIN_X);
        self.player.pos.1 = self.player.pos.1.min(pthapi::GAME_MAX_Y).max(pthapi::GAME_MIN_Y);

        let mut game_data = ScriptGameData {
            player_tran: self.player.pos,
            submit_command: Vec::with_capacity(4),
            calc_stack: Vec::with_capacity(4),
        };

        let mut idx = 0;
        'bl:
        loop {
            'bullet_for:
            for bullet in &mut self.player_bullets[idx..] {
                {
                    for (idx, enemy) in self.enemies.iter_mut().enumerate() {
                        if enemy.hp <= 0.0 {
                            continue;
                        }
                        if enemy.collide.is_collide_with_point(&enemy.pos, &bullet.pos) {
                            enemy.hp -= bullet.damage;
                            if enemy.hp <= 0.0 {
                                let mut temp = TempGameContext {
                                    tran: Some(&mut enemy.pos),
                                };
                                let result = enemy.script.exe_fn_if_present(&ON_DIE_FUNCTION.to_string(), &mut game_data, &mut self.script_manager, &mut temp)
                                    .unwrap_or(0.0);
                                self.enemies.swap_remove(idx);
                                if result == 9.0 {
                                    //anime here
                                }
                            }
                            continue 'bullet_for;
                        }
                    }
                }
                bullet.pos.1 += 30.0;
                if is_out_of_game(&bullet.pos) {
                    self.player_bullets.swap_remove(idx);
                    continue 'bl;
                }
                idx += 1;
            }
            break;
        }


        let commands = &mut self.commands;
        idx = 0;
        'el:
        loop {
            for enemy_bullet in &mut self.enemy_bullets[idx..] {
                let bullet_tran = &mut enemy_bullet.pos;
                if is_out_of_game(bullet_tran) {
                    self.enemy_bullets.swap_remove(idx);
                    continue 'el;
                }

                let mut temp = TempGameContext {
                    tran: Some(bullet_tran)
                };
                enemy_bullet.script.tick_function(&mut game_data, &mut self.script_manager, &mut temp);
                let mut killed = false;
                while let Some(x) = game_data.submit_command.pop() {
                    match x {
                        crate::script::ScriptGameCommand::Move(v) => {
                            bullet_tran.0 += enemy_bullet.rot.facing.0 * v;
                            bullet_tran.1 += enemy_bullet.rot.facing.1 * v;
                        }
                        crate::script::ScriptGameCommand::Kill => {
                            if !killed {
                                killed = true;
                            }
                        }
                        crate::script::ScriptGameCommand::SummonBullet(..) => {
                            commands.push(x);
                        }
                        _ => {
                            unimplemented!("Not ready")
                        }
                    }
                }
                if killed {
                    self.enemy_bullets.swap_remove(idx);
                    continue 'el;
                }
                idx += 1;
            }
            break;
        }


        for enemy in &mut self.enemies {
            let enemy_tran = &mut enemy.pos;
            let mut temp = TempGameContext {
                tran: Some(enemy_tran)
            };
            enemy.script.tick_function(&mut game_data, &mut self.script_manager, &mut temp);

            while let Some(x) = game_data.submit_command.pop() {
                match x {
                    ScriptGameCommand::SummonBullet(..) => {
                        commands.push(x);
                    }
                    ScriptGameCommand::SummonEnemy(..) => {
                        commands.push(x);
                    }
                    _ => {
                        unimplemented!("Not ready")
                    }
                }
            }
        }

        while let Some(x) = commands.pop() {
            match x {
                ScriptGameCommand::SummonBullet(name, x, y, z, scale, angle, collide, script, args) => {
                    let script_context;
                    if let Some(script) = self.script_manager.get_script(&script) {
                        script_context = ScriptContext::new(script, args);
                    } else {
                        let script = self.script_manager.load_script(&script).unwrap();
                        script_context = ScriptContext::new(script, args);
                    }
                    let tex = data.global_state.handles.texture_map.read().unwrap()[&name];
                    data.render.render2d.add_tex(data.global_state, tex);

                    self.enemy_bullets.push(EnemyBullet {
                        pos: (x, y, z),
                        rot: Rotation::new(angle),
                        scale,
                        tex,
                        collide,
                        script: script_context,
                    });
                }
                ScriptGameCommand::SummonEnemy(name, x, y, z, hp, collide, script, args) => {
                    let script_context;
                    if let Some(script) = self.script_manager.get_script(&script) {
                        script_context = ScriptContext::new(script, args);
                    } else {
                        let script = self.script_manager.load_script(&script).unwrap();
                        script_context = ScriptContext::new(script, args);
                    }
                }
                _ => {
                    unimplemented!("Not ready")
                }
            }
        }

        if game_data.calc_stack.len() != 0 {
            eprintln!("Not balance");
        }

        Trans::None
    }

    fn render(&mut self, data: &mut StateData) -> Trans {
        self.obj.clear();

        self.obj.push(Texture2DObject::with_game_pos(self.player.pos, 100.0, 100.0, self.player.tex));
        println!("{:?}", self.obj[0]);
        for x in &self.player_bullets {
            self.obj.push(Texture2DObject::with_game_pos(x.pos, 20.0, 20.0, x.tex));
        }
        for x in &self.enemy_bullets {
            self.obj.push(Texture2DObject::with_game_pos(x.pos, 100.0 * x.scale, 100.0 * x.scale, x.tex));
        }

        for x in &self.enemies {
            self.obj.push(Texture2DObject::with_game_pos(x.pos, 100.0, 100.0, x.tex));
        }
        self.obj.sort();
        data.render.render2d.render(&data.global_state, &data.render.views.get_screen().view, &self.obj);
        // self.player
        // data.render.render2d.render()
        Trans::None
    }

    fn stop(&mut self, _: &mut StateData) {
        //todo: clean up animations
    }
}

//
// fn process_player_move(data: &mut StateData) {
//     let player = data.players.get_mut(entity).unwrap();
//     let pos = data.transforms.get_mut(entity).unwrap();
//     let input = &data.core.cur_input;
//     let is_walk = input.pressing.contains(&VirtualKeyCode::LShift);
//     let (mov_x, mov_y) = input.get_move(if is_walk {
//         player.walk_speed
//     } else {
//         player.move_speed
//     });
//     let (raw_x, raw_y) = (pos.translation().x, pos.translation().y);
//     pos.set_translation_x((mov_x + raw_x).max(0.0 + 50.0).min(1600.0 - 50.0))
//         .set_translation_y((mov_y + raw_y).max(0.0 + 50.0).min(900.0 - 50.0));
//
//     if is_walk {
//         data.animations.0.insert(entity, InvertColorCircle {
//             pos: (*pos).clone(),
//             radius: player.radius,
//         }).expect("Insert error");
//     } else {
//         data.animations.0.remove(entity);
//     }
//
//     if player.shoot_cooldown == 0 {
//         if input.pressing.contains(&VirtualKeyCode::Z) {
//             player.shoot_cooldown = 2;
//             let mut pos = (*pos).clone();
//             pos.set_translation_z(1.0);
//             pos.set_scale(Vector3::new(0.5, 0.5, 1.0));
//             data.entities.build_entity()
//                 .with(pos, &mut data.transforms)
//                 .with(PlayerBullet { damage: 10.0 }, &mut data.player_bullets)
//                 .with(data.resources_handles.player_bullet.clone().unwrap(), &mut data.sprite_renders)
//                 .with(Transparent, &mut data.transparent)
//                 .build();
//         }
//     } else {
//         player.shoot_cooldown -= 1;
//     }
//     let pos = data.transforms.get(entity).unwrap();
//
//     let collide = CollideType::Circle(player.radius * player.radius);
//
//     let die = (&data.enemy_bullets, &data.entities).par_join().any(|(bullet, enemy_bullet_entity)| {
//         let enemy_tran = data.transforms.get(enemy_bullet_entity).unwrap();
//         if bullet.collide.is_collide_with(enemy_tran.translation(), &collide, pos.translation()) {
//             true
//         } else {
//             false
//         }
//     });
//     if die {
//         boss_die_anime(&mut data.entities, (&mut data.animations.0, &mut data.animations.1), pos.translation());
//         data.entities.delete(entity).expect("delete player entity failed");
//         data.core.player = None;
//     }
// }
/*
fn boss_die_anime<'a>(entities: &Entities<'a>,
                      mut animations: (&mut WriteStorage<'a, InvertColorCircle>, &mut WriteStorage<'a, InvertColorAnimation>),
                      enemy_pos: &Vector3<f32>) {
    let last_seconds = 7.0;
    let spread_per_second = 300.0;
    let delay_second = 0.0;
    let mut transform = Transform::default();
    transform.set_translation_x(enemy_pos.x);
    transform.set_translation_y(enemy_pos.y);
    transform.set_translation_z(enemy_pos.z);
    entities.build_entity()
        .with(InvertColorCircle {
            pos: Transform::from(transform.clone()),
            radius: 0.0,
        }, &mut animations.0)
        .with(InvertColorAnimation {
            last_seconds,
            spread_per_second,
            delay_second,
            transform: None,
        }, &mut animations.1)
        .build();
    let last_seconds = 6.75;
    let spread_per_second = 375.0;
    let delay_second = 0.25;
    transform.set_translation_x(enemy_pos.x - 50.0);
    transform.set_translation_y(enemy_pos.y + 50.0);
    entities.build_entity()
        .with(InvertColorAnimation {
            last_seconds,
            spread_per_second,
            delay_second,
            transform: Some(transform.clone()),
        }, &mut animations.1)
        .build();
    transform.set_translation_x(enemy_pos.x + 50.0);
    entities.build_entity()
        .with(InvertColorAnimation {
            last_seconds,
            spread_per_second,
            delay_second,
            transform: Some(transform.clone()),
        }, &mut animations.1)
        .build();
    transform.set_translation_y(enemy_pos.y - 50.0);
    entities.build_entity()
        .with(InvertColorAnimation {
            last_seconds,
            spread_per_second,
            delay_second,
            transform: Some(transform.clone()),
        }, &mut animations.1)
        .build();
    transform.set_translation_x(enemy_pos.x - 50.0);
    entities.build_entity()
        .with(InvertColorAnimation {
            last_seconds,
            spread_per_second,
            delay_second,
            transform: Some(transform.clone()),
        }, &mut animations.1)
        .build();

    let last_seconds = 6.0;
    let spread_per_second = 450.0;
    let delay_second = 1.0;
    transform.set_translation_x(enemy_pos.x);
    transform.set_translation_y(enemy_pos.y);
    entities.build_entity()
        .with(InvertColorAnimation {
            last_seconds,
            spread_per_second,
            delay_second,
            transform: Some(transform),
        }, &mut animations.1)
        .build();
}
 */
#[inline]
pub fn is_out_of_game(tran: &PosType) -> bool {
    tran.0 < GAME_MIN_X - 100.0 || tran.0 > GAME_MAX_X + 100.0 || tran.1 > GAME_MAX_Y + 100.0 || tran.1 < GAME_MIN_Y - 100.0
}
