use amethyst::{
    core::timing::Time,
    derive::SystemDesc,
    ecs::{Entities, prelude::{ParallelIterator, ParJoin}, Read, ReadExpect, System, SystemData, WriteStorage},
    ui::UiText,
};

#[derive(SystemDesc)]
pub struct DebugSystem;

impl<'a> System<'a> for DebugSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, crate::ui::DebugText>,
        WriteStorage<'a, UiText>,
        Read<'a, Time>
    );
    fn run(&mut self, (entities, debug_text, mut ui_texts, time): Self::SystemData) {
        if time.delta_time().as_secs_f32() > 1.0 / 50.0 {
            println!("lag! in {:?}", time.delta_time());
        }
        if let Some(text) = ui_texts.get_mut(debug_text.entity_count) {
            let fps = 1.0 / time.delta_real_time().as_secs_f32();
            text.text = "fps:".to_owned() + &*format!("{:.2}", fps) + "\nentities: " + &(&entities).par_join().count().to_string();
        }
    }
}