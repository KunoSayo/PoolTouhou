use amethyst::{
    assets::Loader,
    ecs::{Entity, prelude::Join, World},
    prelude::{Builder, WorldExt},
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};
use amethyst::ui::LineMode;

pub struct DebugText {
    pub entity_count: Entity
}

pub fn setup_debug_text(world: &mut World) {
    let font = world.read_resource::<Loader>().load(
        "font/simhei.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    );

    let entity_count_transform = UiTransform::new(
        "entity_count".to_string(), Anchor::BottomRight, Anchor::BottomLeft,
        -200., 0., 1., 200., 40.,
    );

    let entities_now = (&world.entities()).join().count().to_string();
    let text = UiText::new(
        font.clone(),
        "entities: ".to_owned() + &entities_now,
        [1., 1., 1., 1.],
        20.,
        LineMode::Wrap,
        Anchor::MiddleRight,
    );
    let entity_count = world
        .create_entity()
        .with(entity_count_transform)
        .with(text)
        .build();


    world.insert(DebugText { entity_count });
}