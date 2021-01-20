use amethyst::{
    assets::Loader,
    ecs::{Entity, World},
    prelude::{Builder, WorldExt},
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};
use amethyst::ui::LineMode;

pub struct DebugText {
    pub debug_text_entity: Entity
}

pub fn setup_debug_text(world: &mut World) {
    let font = world.read_resource::<Loader>().load(
        "font/simhei.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    );

    let entity_count_transform = UiTransform::new(
        "debug_text_trans".into(), Anchor::BottomRight, Anchor::BottomLeft,
        -200., 0., 1., 200., 40.,
    );

    let text = UiText::new(
        font.clone(),
        "".into(),
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


    world.insert(DebugText { debug_text_entity: entity_count });
}