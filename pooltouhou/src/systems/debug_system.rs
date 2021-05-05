use amethyst::{
    core::timing::Time,
    derive::SystemDesc,
    ecs::{Entities, prelude::{ParallelIterator, ParJoin}, Read, ReadExpect, System, SystemData, WriteStorage},
    ui::UiText,
};

#[derive(SystemDesc, Default)]
pub struct DebugSystem {
    count: u32,
    delta: f32,
    fps: f32,
}

impl<'a> System<'a> for DebugSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, crate::ui::DebugText>,
        WriteStorage<'a, UiText>,
        Read<'a, Time>
    );
    fn run(&mut self, (entities, debug_text, mut ui_texts, time): Self::SystemData) {
        if let Some(text) = ui_texts.get_mut(debug_text.debug_text_entity) {
            self.delta += time.delta_real_time().as_secs_f32();
            self.count += 1;
            if self.delta >= 1.0 {
                self.fps = self.count as f32 / self.delta;
                self.delta = 0.0;
                self.count = 0;
            }
            if cfg!(feature = "debug-game") {
                text.text = format!("entities:{}\nfps:{:.2} ", (&entities).par_join().count(), self.fps);
            } else {
                text.text = format!("fps:{:.2}", self.fps);
            }
        }
    }
}