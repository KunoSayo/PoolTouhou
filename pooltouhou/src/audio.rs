use std::fmt::Formatter;
use std::sync::Arc;

use alto::{Alto, Buffer, Context, OutputDevice, Source, StaticSource};

use crate::Config;

pub struct OpenalData {
    pub alto: Alto,
    pub device: OutputDevice,
    pub ctx: Context,
    pub bgm_source: StaticSource,
}

impl std::fmt::Debug for OpenalData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenalData")
            .field("Not supported yet", &"Not supported yet")
            .finish()
    }
}

impl OpenalData {
    pub fn new(config: &mut Config) -> Result<Self, Box<dyn std::error::Error>> {
        let alto = Alto::load_default()?;
        let device = alto.open(None)?;
        let ctx = device.new_context(None)?;
        let mut bgm_source = ctx.new_static_source()?;
        if let Err(e) = bgm_source
            .set_gain(config.parse_or_default("bgm-gain",
                                              &bgm_source.gain().to_string())) {
            log::warn!("Change bgm gain failed for {:?}", e);
        }
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