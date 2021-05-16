// use std::convert::TryFrom;
// use std::io::{Error, ErrorKind};
//
// use amethyst::{
//     core::{components::Transform},
//     derive::SystemDesc,
//     ecs::{Entities, Read, RunningTime, System, SystemData, World, Write, WriteStorage},
//     ecs::prelude::{Component, DenseVecStorage, Join, ParallelIterator, ParJoin},
//     input::VirtualKeyCode,
//     renderer::{SpriteRender, Transparent},
//     shred::ResourceId,
// };
// use failure::_core::f32::consts::PI;
// use nalgebra::Vector3;
//
// use crate::component::{Enemy, EnemyBullet, InvertColorAnimation, PlayerBullet};
// use crate::GameCore;
// use crate::handles::ResourcesHandles;
// use crate::render::InvertColorCircle;
// use crate::script::{ON_DIE_FUNCTION, ScriptGameCommand, ScriptGameData, ScriptManager};
// use crate::script::script_context::{ScriptContext, TempGameContext};
//
// #[derive(Default)]
// pub struct Player {
//     move_speed: f32,
//     walk_speed: f32,
//     radius: f32,
//     shoot_cooldown: u8,
// }
//
// #[derive(Debug)]
// pub enum CollideType {
//     Circle(f32)
// }
//
// impl CollideType {
//     pub fn is_collide_with_point(&self, me: &Vector3<f32>, other: &Vector3<f32>) -> bool {
//         match self {
//             Self::Circle(r_2) => {
//                 let x_distance = me.x - other.x;
//                 let y_distance = me.y - other.y;
//                 x_distance * x_distance + y_distance * y_distance < *r_2
//             }
//         }
//     }
//
//     pub fn is_collide_with(&self, me: &Vector3<f32>, other_collide: &CollideType, other: &Vector3<f32>) -> bool {
//         match self {
//             Self::Circle(r_2) => {
//                 match other_collide {
//                     Self::Circle(o_r_2) => {
//                         let center_x_distance = me.x - other.x;
//                         let center_y_distance = me.y - other.y;
//                         let center_distance = center_x_distance * center_x_distance + center_y_distance * center_y_distance;
//                         center_distance < r_2 + o_r_2
//                     }
//                 }
//             }
//         }
//     }
// }
//
// impl TryFrom<(u8, Vec<f32>)> for CollideType {
//     type Error = Error;
//
//     fn try_from((value, args): (u8, Vec<f32>)) -> Result<Self, Self::Error> {
//         match value {
//             10 => Ok(CollideType::Circle(args[0] * args[0])),
//             _ => Err(Error::new(ErrorKind::InvalidData, "No such value for CollideType: ".to_owned() + &*value.to_string()))
//         }
//     }
// }
//
// impl CollideType {
//     pub fn get_arg_count(byte: u8) -> usize {
//         match byte {
//             10 => 1,
//             _ => panic!("Not collide byte: {}", byte)
//         }
//     }
// }
//
// impl Player {
//     pub fn new(speed: f32) -> Self {
//         Self {
//             move_speed: speed,
//             walk_speed: speed * 0.6,
//             radius: 5.0,
//             shoot_cooldown: 0,
//         }
//     }
// }
//
// impl Component for Player {
//     type Storage = DenseVecStorage<Self>;
// }
//
// #[derive(SystemData)]
// pub struct GameSystemData<'a> {
//     transforms: WriteStorage<'a, Transform>,
//     player_bullets: WriteStorage<'a, PlayerBullet>,
//     sprite_renders: WriteStorage<'a, SpriteRender>,
//     transparent: WriteStorage<'a, Transparent>,
//     players: WriteStorage<'a, Player>,
//     resources_handles: Read<'a, ResourcesHandles>,
//     core: Write<'a, GameCore>,
//     entities: Entities<'a>,
//     enemies: WriteStorage<'a, crate::component::Enemy>,
//     enemy_bullets: WriteStorage<'a, EnemyBullet>,
//     animations: (WriteStorage<'a, InvertColorCircle>, WriteStorage<'a, InvertColorAnimation>),
//     script_manager: Write<'a, ScriptManager>,
// }
//
//
// #[derive(SystemDesc)]
// pub struct GameSystem;
//
// impl<'a> System<'a> for GameSystem {
//     type SystemData = GameSystemData<'a>;
//
//
//     fn run(&mut self, mut data: Self::SystemData) {
//         let player_tran = process_player(&mut data);
//         let mut game_data = ScriptGameData {
//             player_tran,
//             submit_command: Vec::with_capacity(4),
//             calc_stack: Vec::with_capacity(4),
//         };
//
//         data.core.tick += 1;
//         'bullet_for: for (bullet, bullet_entity) in (&data.player_bullets, &data.entities).join() {
//             {
//                 let bullet_pos = data.transforms.get(bullet_entity).unwrap().translation();
//                 for (enemy, enemy_entity) in (&mut data.enemies, &data.entities).join() {
//                     if enemy.hp <= 0.0 {
//                         continue;
//                     }
//                     let enemy_tran = data.transforms.get(enemy_entity).unwrap();
//                     let enemy_pos = enemy_tran.translation();
//                     if enemy.collide.is_collide_with_point(enemy_pos, bullet_pos) {
//                         enemy.hp -= bullet.damage;
//                         if enemy.hp <= 0.0 {
//                             data.entities.delete(enemy_entity).expect("delete enemy entity failed");
//                             let mut enemy_tran = data.transforms.get_mut(enemy_entity).unwrap();
//                             let mut temp = TempGameContext {
//                                 tran: Some(&mut enemy_tran),
//                             };
//                             let result = enemy.script.exe_fn_if_present(&ON_DIE_FUNCTION.to_string(), &mut game_data, &mut data.script_manager, &mut temp)
//                                 .unwrap_or(0.0);
//                             if result == 9.0 {
//                                 boss_die_anime(&data.entities, (&mut data.animations.0, &mut data.animations.1), enemy_tran.translation());
//                             }
//                         }
//                         data.entities.delete(bullet_entity).expect("delete bullet entity failed");
//
//                         continue 'bullet_for;
//                     }
//                 }
//             }
//             let pos = data.transforms.get_mut(bullet_entity).unwrap();
//             pos.move_up(30.0);
//             if is_out_of_game(pos) {
//                 data.entities.delete(bullet_entity).expect("delete bullet entity failed");
//             }
//         }
//
//
//         for (enemy_bullet, bullet_entity)
//         in (&mut data.enemy_bullets, &data.entities).join() {
//             let mut bullet_tran = data.transforms.get_mut(bullet_entity).unwrap();
//             if is_out_of_game(bullet_tran) {
//                 data.entities.delete(bullet_entity).expect("delete enemy bullet entity failed");
//                 continue;
//             }
//
//             let mut temp = TempGameContext {
//                 tran: Some(&mut bullet_tran),
//             };
//             enemy_bullet.script.tick_function(&mut game_data, &mut data.script_manager, &mut temp);
//             while let Some(x) = game_data.submit_command.pop() {
//                 match x {
//                     crate::script::ScriptGameCommand::MoveUp(v) => {
//                         bullet_tran.move_up(v);
//                     }
//                     crate::script::ScriptGameCommand::Kill => {
//                         data.entities.delete(bullet_entity).expect("delete the entity failed");
//                     }
//                     crate::script::ScriptGameCommand::SummonBullet(..) => {
//                         data.core.commands.push(x);
//                     }
//                     _ => {
//                         unimplemented!("Not ready")
//                     }
//                 }
//             }
//         }
//
//
//         for (enemy, enemy_entity) in (&mut data.enemies, &data.entities).join() {
//             let mut enemy_tran = data.transforms.get_mut(enemy_entity).unwrap();
//             let mut temp = TempGameContext {
//                 tran: Some(&mut enemy_tran),
//             };
//             enemy.script.tick_function(&mut game_data, &mut data.script_manager, &mut temp);
//
//             while let Some(x) = game_data.submit_command.pop() {
//                 match x {
//                     ScriptGameCommand::SummonBullet(..) => {
//                         data.core.commands.push(x);
//                     }
//                     ScriptGameCommand::SummonEnemy(..) => {
//                         data.core.commands.push(x);
//                     }
//                     _ => {
//                         unimplemented!("Not ready")
//                     }
//                 }
//             }
//         }
//
//         while let Some(x) = data.core.commands.pop() {
//             match x {
//                 ScriptGameCommand::SummonBullet(name, x, y, z, scale, angle, collide, script, args) => {
//                     let script_context;
//                     if let Some(script) = data.script_manager.get_script(&script) {
//                         script_context = ScriptContext::new(script, args);
//                     } else {
//                         let script = data.script_manager.load_script(&script).unwrap();
//                         script_context = ScriptContext::new(script, args);
//                     }
//                     let mut pos = Transform::default();
//                     pos.set_translation_xyz(x, y, z);
//                     pos.set_rotation_z_axis(angle / 180.0 * PI);
//                     pos.set_scale(Vector3::new(scale, scale, 1.0));
//                     data.entities.build_entity()
//                         .with(pos, &mut data.transforms)
//                         .with(EnemyBullet { collide, script: script_context }, &mut data.enemy_bullets)
//                         .with(data.resources_handles.sprites.get(&name).unwrap().clone(), &mut data.sprite_renders)
//                         .with(Transparent, &mut data.transparent)
//                         .build();
//                 }
//                 ScriptGameCommand::SummonEnemy(name, x, y, z, hp, collide, script, args) => {
//                     let script_context;
//                     if let Some(script) = data.script_manager.get_script(&script) {
//                         script_context = ScriptContext::new(script, args);
//                     } else {
//                         let script = data.script_manager.load_script(&script).unwrap();
//                         script_context = ScriptContext::new(script, args);
//                     }
//                     let mut pos = Transform::default();
//                     pos.set_translation_xyz(x, y, z);
//                     data.entities.build_entity()
//                         .with(pos, &mut data.transforms)
//                         .with(Enemy::new(hp, collide, script_context), &mut data.enemies)
//                         .with(data.resources_handles.sprites.get(&name).unwrap().clone(), &mut data.sprite_renders)
//                         .with(Transparent, &mut data.transparent)
//                         .build();
//                 }
//                 _ => {
//                     unimplemented!("Not ready")
//                 }
//             }
//         }
//
//         //tick if end
//
//         if game_data.calc_stack.len() != 0 {
//             eprintln!("Not balance");
//         }
//     }
//
//
//     fn running_time(&self) -> RunningTime {
//         RunningTime::Long
//     }
// }
//
//
// fn process_player(data: &mut GameSystemData) -> Transform {
//     if let Some(entity) = data.core.player {
//         let player = data.players.get_mut(entity).unwrap();
//         let pos = data.transforms.get_mut(entity).unwrap();
//         let input = &data.core.cur_input;
//         let is_walk = input.pressing.contains(&VirtualKeyCode::LShift);
//         let (mov_x, mov_y) = input.get_move(if is_walk {
//             player.walk_speed
//         } else {
//             player.move_speed
//         });
//         let (raw_x, raw_y) = (pos.translation().x, pos.translation().y);
//         pos.set_translation_x((mov_x + raw_x).max(0.0 + 50.0).min(1600.0 - 50.0))
//             .set_translation_y((mov_y + raw_y).max(0.0 + 50.0).min(900.0 - 50.0));
//
//         if is_walk {
//             data.animations.0.insert(entity, InvertColorCircle {
//                 pos: (*pos).clone(),
//                 radius: player.radius,
//             }).expect("Insert error");
//         } else {
//             data.animations.0.remove(entity);
//         }
//
//         if player.shoot_cooldown == 0 {
//             if input.pressing.contains(&VirtualKeyCode::Z) {
//                 player.shoot_cooldown = 2;
//                 let mut pos = (*pos).clone();
//                 pos.set_translation_z(1.0);
//                 pos.set_scale(Vector3::new(0.5, 0.5, 1.0));
//                 data.entities.build_entity()
//                     .with(pos, &mut data.transforms)
//                     .with(PlayerBullet { damage: 10.0 }, &mut data.player_bullets)
//                     .with(data.resources_handles.player_bullet.clone().unwrap(), &mut data.sprite_renders)
//                     .with(Transparent, &mut data.transparent)
//                     .build();
//             }
//         } else {
//             player.shoot_cooldown -= 1;
//         }
//         let pos = data.transforms.get(entity).unwrap();
//
//         let collide = CollideType::Circle(player.radius * player.radius);
//
//         let die = (&data.enemy_bullets, &data.entities).par_join().any(|(bullet, enemy_bullet_entity)| {
//             let enemy_tran = data.transforms.get(enemy_bullet_entity).unwrap();
//             if bullet.collide.is_collide_with(enemy_tran.translation(), &collide, pos.translation()) {
//                 true
//             } else {
//                 false
//             }
//         });
//         if die {
//             boss_die_anime(&mut data.entities, (&mut data.animations.0, &mut data.animations.1), pos.translation());
//             data.entities.delete(entity).expect("delete player entity failed");
//             data.core.player = None;
//         }
//         pos.clone()
//     } else {
//         Transform::default()
//         //fixme: not delete the entity for player while dying
//     }
// }
//
// fn boss_die_anime<'a>(entities: &Entities<'a>,
//                       mut animations: (&mut WriteStorage<'a, InvertColorCircle>, &mut WriteStorage<'a, InvertColorAnimation>),
//                       enemy_pos: &Vector3<f32>) {
//     let last_seconds = 7.0;
//     let spread_per_second = 300.0;
//     let delay_second = 0.0;
//     let mut transform = Transform::default();
//     transform.set_translation_x(enemy_pos.x);
//     transform.set_translation_y(enemy_pos.y);
//     transform.set_translation_z(enemy_pos.z);
//     entities.build_entity()
//         .with(InvertColorCircle {
//             pos: Transform::from(transform.clone()),
//             radius: 0.0,
//         }, &mut animations.0)
//         .with(InvertColorAnimation {
//             last_seconds,
//             spread_per_second,
//             delay_second,
//             transform: None,
//         }, &mut animations.1)
//         .build();
//     let last_seconds = 6.75;
//     let spread_per_second = 375.0;
//     let delay_second = 0.25;
//     transform.set_translation_x(enemy_pos.x - 50.0);
//     transform.set_translation_y(enemy_pos.y + 50.0);
//     entities.build_entity()
//         .with(InvertColorAnimation {
//             last_seconds,
//             spread_per_second,
//             delay_second,
//             transform: Some(transform.clone()),
//         }, &mut animations.1)
//         .build();
//     transform.set_translation_x(enemy_pos.x + 50.0);
//     entities.build_entity()
//         .with(InvertColorAnimation {
//             last_seconds,
//             spread_per_second,
//             delay_second,
//             transform: Some(transform.clone()),
//         }, &mut animations.1)
//         .build();
//     transform.set_translation_y(enemy_pos.y - 50.0);
//     entities.build_entity()
//         .with(InvertColorAnimation {
//             last_seconds,
//             spread_per_second,
//             delay_second,
//             transform: Some(transform.clone()),
//         }, &mut animations.1)
//         .build();
//     transform.set_translation_x(enemy_pos.x - 50.0);
//     entities.build_entity()
//         .with(InvertColorAnimation {
//             last_seconds,
//             spread_per_second,
//             delay_second,
//             transform: Some(transform.clone()),
//         }, &mut animations.1)
//         .build();
//
//     let last_seconds = 6.0;
//     let spread_per_second = 450.0;
//     let delay_second = 1.0;
//     transform.set_translation_x(enemy_pos.x);
//     transform.set_translation_y(enemy_pos.y);
//     entities.build_entity()
//         .with(InvertColorAnimation {
//             last_seconds,
//             spread_per_second,
//             delay_second,
//             transform: Some(transform),
//         }, &mut animations.1)
//         .build();
// }
//
// #[inline]
// pub fn is_out_of_game(tran: &Transform) -> bool {
//     let tran = tran.translation();
//     tran.x < -100.0 || tran.x > 1700.0 || tran.y > 1000.0 || tran.y < -100.0
// }