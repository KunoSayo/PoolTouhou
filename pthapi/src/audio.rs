use std::fmt::Formatter;
use std::sync::Arc;

use alto::{Alto, Buffer, Context, OutputDevice, Source, StaticSource};

pub struct OpenalData {
    alto: Alto,
    device: OutputDevice,
    pub ctx: Context,
    bgm_source: StaticSource,
}

impl std::fmt::Debug for OpenalData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenalData")
            .field("Not supported yet", &"Not supported yet")
            .finish()
    }
}

impl OpenalData {
    pub(crate) fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let alto = Alto::load_default()?;
        let device = alto.open(None)?;
        let ctx = device.new_context(None)?;
        let bgm_source = ctx.new_static_source()?;
        Ok(Self {
            alto,
            device,
            ctx,
            bgm_source,
        })
    }
}


impl OpenalData {
    pub fn play_bgm(&mut self, buf: Arc<Buffer>) {
        self.bgm_source.stop();
        self.bgm_source.set_looping(true);
        if let Err(e) = self.bgm_source.set_buffer(buf) {
            log::warn!("Play bgm failed for {}", e);
        } else {
            log::info!("To play new bgm");
            self.bgm_source.play();
        }
    }
}