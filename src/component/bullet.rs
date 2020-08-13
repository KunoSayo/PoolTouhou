use amethyst::{
    core::components::Transform,
    ecs::prelude::{Component, DenseVecStorage},
};

pub trait BulletAi: Send + Sync {
    fn tick(&mut self, transform: &mut Transform) -> bool;
}


#[derive(Default)]
pub struct PlayerBullet {
    pub damage: f32
}

pub struct EnemyBullet {
    pub ai: Box<dyn BulletAi>
}

pub struct NormalBulletAi {
    v: f32,
    a: f32,
}

impl BulletAi for NormalBulletAi {
    fn tick(&mut self, transform: &mut Transform) -> bool {
        transform.move_up(self.v);
        self.v += self.a;
        crate::systems::game_system::is_out_of_game(transform)
    }
}


impl EnemyBullet {
    pub fn new(v: f32) -> Self {
        Self {
            ai: Box::new(NormalBulletAi {
                v,
                a: 0.0,
            })
        }
    }
}


impl Component for PlayerBullet {
    type Storage = DenseVecStorage<Self>;
}


impl Component for EnemyBullet {
    type Storage = DenseVecStorage<Self>;
}
