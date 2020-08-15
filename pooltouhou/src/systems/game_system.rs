use std::convert::TryFrom;
use std::io::{Error, ErrorKind};

use amethyst::{
    core::{components::Transform},
    derive::SystemDesc,
    ecs::{Entities, Read, RunningTime, System, SystemData, World, Write, WriteStorage},
    ecs::prelude::{Component, DenseVecStorage, Join},
    input::VirtualKeyCode,
    renderer::{SpriteRender, Transparent},
    shred::ResourceId,
};
use nalgebra::Vector3;

use crate::component::{EnemyBullet, InvertColorAnimation, PlayerBullet};
use crate::CoreStorage;
use crate::handles::TextureHandles;
use crate::render::InvertColorCircle;
use crate::script::{ScriptGameData, ScriptManager};

#[derive(Default)]
pub struct Player {
    move_speed: f32,
    walk_speed: f32,
    radius: f32,
    shoot_cooldown: u8,
}

#[derive(Debug)]
pub enum CollideType {
    Circle(f32)
}

impl TryFrom<(u8, Vec<f32>)> for CollideType {
    type Error = Error;

    fn try_from((value, args): (u8, Vec<f32>)) -> Result<Self, Self::Error> {
        match value {
            10 => Ok(CollideType::Circle(args[0])),
            _ => Err(Error::new(ErrorKind::InvalidData, "No such value for CollideType: ".to_owned() + &*value.to_string()))
        }
    }
}

impl CollideType {
    pub fn get_arg_count(byte: u8) -> usize {
        match byte {
            10 => 1,
            _ => panic!("Not collide byte: {}", byte)
        }
    }
}

impl Player {
    pub fn new(speed: f32) -> Self {
        Self {
            move_speed: speed,
            walk_speed: speed * 0.6,
            radius: 5.0,
            shoot_cooldown: 0,
        }
    }
}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}

#[derive(SystemData)]
pub struct GameSystemData<'a> {
    transforms: WriteStorage<'a, Transform>,
    player_bullets: WriteStorage<'a, PlayerBullet>,
    sprite_renders: WriteStorage<'a, SpriteRender>,
    transparent: WriteStorage<'a, Transparent>,
    players: WriteStorage<'a, Player>,
    texture_handles: Read<'a, TextureHandles>,
    core: Write<'a, CoreStorage>,
    entities: Entities<'a>,
    enemies: WriteStorage<'a, crate::component::Enemy>,
    enemy_bullets: WriteStorage<'a, EnemyBullet>,
    animations: (WriteStorage<'a, InvertColorCircle>, WriteStorage<'a, InvertColorAnimation>),
    script_manager: Write<'a, ScriptManager>,
}


#[derive(SystemDesc)]
pub struct GameSystem;

impl<'a> System<'a> for GameSystem {
    type SystemData = GameSystemData<'a>;


    fn run(&mut self, mut data: Self::SystemData) {
        if data.core.tick_sign {
            data.core.tick_sign = false;
            data.core.tick += 1;
            'bullet_for: for (bullet, bullet_entity) in (&data.player_bullets, &data.entities).join() {
                {
                    let bullet_pos = data.transforms.get(bullet_entity).unwrap().translation();
                    for (enemy, enemy_entity) in (&mut data.enemies, &data.entities).join() {
                        if enemy.hp <= 0.0 {
                            continue;
                        }

                        let enemy_pos = data.transforms.get(enemy_entity).unwrap().translation();
                        let x_distance = (enemy_pos.x - bullet_pos.x).abs();
                        let y_distance = enemy_pos.y - bullet_pos.y;
                        let distance_p2 = if y_distance >= 0.0 {
                            let y_distance = (y_distance - 30.0).max(0.0);
                            x_distance * x_distance + y_distance * y_distance
                        } else {
                            x_distance * x_distance + y_distance * y_distance
                        };
                        if distance_p2 <= enemy.rad_p2 {
                            enemy.hp -= bullet.damage;
                            if enemy.hp <= 0.0 {
                                println!("Anye hp left: 0.0");
                                data.entities.delete(enemy_entity).expect("delete enemy entity failed");
                                boss_die_anime(&data.entities, (&mut data.animations.0, &mut data.animations.1), enemy_pos);
                            } else {
                                println!("Anye hp left: {}", enemy.hp);
                            }
                            data.entities.delete(bullet_entity).expect("delete bullet entity failed");

                            continue 'bullet_for;
                        }
                    }
                }
                let pos = data.transforms.get_mut(bullet_entity).unwrap();
                pos.move_up(30.0);
                if pos.translation().y > 900.0 {
                    data.entities.delete(bullet_entity).expect("delete bullet entity failed");
                }
            }
            process_player(&mut data);
            //tick if end
        }
    }

    fn running_time(&self) -> RunningTime {
        RunningTime::Long
    }
}

fn process_player(data: &mut GameSystemData) {
    if let Some(entity) = data.core.player {
        let player = data.players.get_mut(entity).unwrap();
        let pos = data.transforms.get_mut(entity).unwrap();
        let input = data.core.cur_input.as_ref().unwrap();
        let is_walk = input.pressing.contains(&VirtualKeyCode::LShift);
        let (mov_x, mov_y) = input.get_move(if is_walk {
            player.walk_speed
        } else {
            player.move_speed
        });
        let (raw_x, raw_y) = (pos.translation().x, pos.translation().y);
        pos.set_translation_x((mov_x + raw_x).max(0.0).min(1600.0))
            .set_translation_y((mov_y + raw_y).max(0.0).min(900.0));

        if is_walk {
            data.animations.0.insert(entity, InvertColorCircle {
                pos: (*pos).clone(),
                radius: player.radius,
            }).expect("Insert error");
        } else {
            data.animations.0.remove(entity);
        }

        let mut game_data = ScriptGameData {
            tran: None,
            player_tran: Some((*pos).clone()),
            submit_command: vec![],
            script_manager: &mut data.script_manager,
        };

        if player.shoot_cooldown == 0 {
            if input.pressing.contains(&VirtualKeyCode::Z) {
                player.shoot_cooldown = 2;
                let mut pos = (*pos).clone();
                pos.prepend_translation_z(-1.0);
                pos.set_scale(Vector3::new(0.5, 0.5, 1.0));
                data.entities.build_entity()
                    .with(pos, &mut data.transforms)
                    .with(PlayerBullet { damage: 10.0 }, &mut data.player_bullets)
                    .with(data.texture_handles.player_bullet.clone().unwrap(), &mut data.sprite_renders)
                    .with(Transparent, &mut data.transparent)
                    .build();
            }
        } else {
            player.shoot_cooldown -= 1;
        }
    }
}

fn boss_die_anime<'a>(entities: &Entities<'a>,
                      mut animations: (&mut WriteStorage<'a, InvertColorCircle>, &mut WriteStorage<'a, InvertColorAnimation>),
                      enemy_pos: &Vector3<f32>) {
    let last_seconds = 5.0;
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
    let last_seconds = 4.75;
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

    let last_seconds = 4.0;
    let spread_per_second = 500.0;
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

pub fn is_out_of_game(tran: &Transform) -> bool {
    let tran = tran.translation();
    tran.x < 0.0 || tran.x > 1600.0 || tran.y > 900.0 || tran.y < 0.0
}