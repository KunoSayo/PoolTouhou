use amethyst::{
    ecs::prelude::{Component, DenseVecStorage},
    core::components::Transform
};

#[derive(Default)]
pub struct PlayerBullet {
    pub damage: f32
}

pub struct EnemyBullet {
    pub tick: u32,
    pub v: f32,
    pub a: f32,
    pub ai: fn(&mut Self, transform: &mut Transform)
}

fn normal_ai(enemy_bullet: &mut EnemyBullet, transform: &mut Transform) {
    enemy_bullet.tick += 1;
    transform.move_up(enemy_bullet.v);
    enemy_bullet.v += enemy_bullet.a;
}

impl EnemyBullet {
    pub fn new(v: f32) -> Self {
        Self {
            tick: 0,
            v,
            a: 0.0,
            ai: normal_ai
        }
    }
}

impl Default for EnemyBullet {
    fn default() -> Self {
        Self {
            tick: 0,
            v: 0.0,
            a: 0.0,
            ai: normal_ai
        }
    }
}

impl Component for PlayerBullet {
    type Storage = DenseVecStorage<Self>;
}


impl Component for EnemyBullet {
    type Storage = DenseVecStorage<Self>;
}
