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

#[derive(Default)]
pub struct Player {
    pub move_speed: f32,
    shoot_cooldown: u8,
}

impl Player {
    pub fn new(speed: f32) -> Self {
        Self {
            move_speed: speed,
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
            if let Some(entity) = data.core.player {
                let p = data.players.get_mut(entity).unwrap();
                let pos = data.transforms.get_mut(entity).unwrap();
                let cur_input = data.core.cur_input.as_ref().unwrap();
                let (mov_x, mov_y) = cur_input.get_move(p.move_speed);
                let (raw_x, raw_y) = (pos.translation().x, pos.translation().y);
                pos.set_translation_x((mov_x + raw_x).max(0.0).min(1600.0))
                    .set_translation_y((mov_y + raw_y).max(0.0).min(900.0));
                if p.shoot_cooldown == 0 {
                    if cur_input.pressing.contains(&VirtualKeyCode::Z) {
                        p.shoot_cooldown = 2;
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
                    p.shoot_cooldown -= 1;
                }

                for (bullet, bullet_pos) in (&mut data.enemy_bullets, &mut data.transforms).join() {
                    (bullet.ai)(bullet, bullet_pos);
                }
            }
        }
    }

    fn running_time(&self) -> RunningTime {
        RunningTime::Long
    }
}

#[inline]
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