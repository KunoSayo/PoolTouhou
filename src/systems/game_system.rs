use amethyst::{
    core::components::Transform,
    derive::SystemDesc,
    ecs::{Entities, Read, System, SystemData, Write, WriteStorage},
    ecs::prelude::{Component, DenseVecStorage, Join},
    input::VirtualKeyCode,
    renderer::{SpriteRender, Transparent},
};
use nalgebra::Vector3;

use crate::CoreStorage;
use crate::entities::PlayerBullet;
use crate::handles::TextureHandles;

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

#[derive(SystemDesc)]
pub struct GameSystem;

impl<'a> System<'a> for GameSystem {
    type SystemData = (
        WriteStorage<'a, Transform>,
        WriteStorage<'a, PlayerBullet>,
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, Transparent>,
        WriteStorage<'a, Player>,
        Read<'a, TextureHandles>,
        Write<'a, CoreStorage>,
        Entities<'a>
    );
    fn run(&mut self, (mut transforms, mut player_bullets, mut sprite_renders, mut transparents, mut player, texture_handles, mut core, entities): Self::SystemData) {
        if core.tick_sign {
            core.tick_sign = false;
            core.tick += 1;
            let mut should_delete = vec![];
            for (pos, _, entity) in (&mut transforms, &player_bullets, &entities).join() {
                pos.prepend_translation_y(30.0);
                if pos.translation().y > 900.0 {
                    should_delete.push(entity);
                }
            }
            for entity in should_delete {
                entities.delete(entity).expect("Where is this sheep bullet?");
            }
            for (pos, p) in (&mut transforms, &mut player).join() {
                let cur_input = core.cur_input.as_ref().unwrap();
                let (mov_x, mov_y) = cur_input.get_move(p.move_speed);
                let (raw_x, raw_y) = (pos.translation().x, pos.translation().y);
                pos.set_translation_x((mov_x + raw_x).max(0.0).min(1600.0))
                    .set_translation_y((mov_y + raw_y).max(0.0).min(900.0));
                if p.shoot_cooldown == 0 {
                    p.shoot_cooldown = 2;
                    if cur_input.pressing.contains(&VirtualKeyCode::Z) {
                        let mut pos = (*pos).clone();
                        pos.prepend_translation_z(1.0);
                        pos.set_scale(Vector3::new(0.5, 0.5, 1.0));
                        entities.build_entity()
                            .with(pos, &mut transforms)
                            .with(PlayerBullet, &mut player_bullets)
                            .with(texture_handles.player_bullet.clone().unwrap(), &mut sprite_renders)
                            .with(Transparent, &mut transparents)
                            .build();
                        break;
                    }
                } else {
                    p.shoot_cooldown -= 1;
                }
            }
        }
    }
}