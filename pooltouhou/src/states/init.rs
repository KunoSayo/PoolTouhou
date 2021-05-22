use crate::states::{Progress, GameState};

#[derive(Default)]
pub struct Loading {
    progress: Progress,
}


impl GameState for Loading {
    // fn on_start(&mut self) {
    //
    //     load_texture(world, "bullet".into(), "bullet.ron".into(), &mut self.progress);
    //     load_texture(world, "circle_red".into(), "circle.ron".into(), &mut self.progress);
    //     load_texture(world, "circle_blue".into(), "circle.ron".into(), &mut self.progress);
    //     load_texture(world, "circle_green".into(), "circle.ron".into(), &mut self.progress);
    //     load_texture(world, "circle_yellow".into(), "circle.ron".into(), &mut self.progress);
    //     load_texture(world, "circle_purple".into(), "circle.ron".into(), &mut self.progress);
    //     load_texture(world, "zzzz".into(), "zzzz.ron".to_string(), &mut self.progress);
    //     load_texture(world, "mainbg".into(), "mainbg.ron".to_string(), &mut self.progress);
    //
    //
    //     setup_camera(world);
    //
    //     crate::ui::debug::setup_debug_text(world, &mut self.progress);
    //
    //     let mut script_manager = ScriptManager::default();
    //     script_manager.load_scripts();
    //
    //
    //     world.insert(script_manager);
    //
    //     println!("Loading state started.");
    // }

    // fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
    //     if self.progress.num_loading() == 0 {
    //         println!("loaded {} resources.", self.progress.num_finished());
    //         match self.progress.complete() {
    //             Completion::Failed => {
    //                 for x in self.progress.errors() {
    //                     eprintln!("load {} failed for {}", x.asset_name, x.error);
    //                 }
    //             }
    //             _ => {}
    //         }
    //         Trans::Push(Box::new(Menu::default()))
    //     } else {
    //         Trans::None
    //     }
    // }

    // fn shadow_update(&mut self, data: StateData<'_, GameData<'_, '_>>) {
    //     if let Some(dispatcher) = self.input_dispatcher.as_mut() {
    //         dispatcher.dispatch(data.world);
    //     }
    //
    //     #[cfg(feature = "debug-game")]
    //         {
    //             let core_storage = data.world.read_resource::<GameCore>();
    //             if core_storage.is_pressed(&[VirtualKeyCode::F3, VirtualKeyCode::T]) {
    //                 println!("reloading...");
    //                 {
    //                     let mut manager = data.world.write_resource::<ScriptManager>();
    //                     manager.load_scripts();
    //                 }
    //             }
    //         }
    // }
}